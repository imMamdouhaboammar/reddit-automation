use crate::accounts::AccountContext;
use crate::auth::AuthManager;
use crate::drivers::{Capability, Driver, DriverError, HealthIssue, HealthReport};
use crate::reddit::{
    KarmaResponse, PostRequirements, RedditIdentity, RedditPreferences, SubredditRulesResponse,
};
use crate::storage::Storage;
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, warn};

pub struct RedditApiDriver {
    client: reqwest::Client,
    auth: Arc<AuthManager>,
    storage: Arc<Storage>,
    client_id: String,
    client_secret: String,
    base_url: String,
}

impl RedditApiDriver {
    pub fn new(
        client: reqwest::Client,
        auth: Arc<AuthManager>,
        storage: Arc<Storage>,
        client_id: String,
        client_secret: String,
    ) -> Self {
        Self {
            client,
            auth,
            storage,
            client_id,
            client_secret,
            base_url: "https://oauth.reddit.com".to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    fn build_user_agent(&self, reddit_username: Option<&str>) -> String {
        let user = reddit_username.unwrap_or("reddit-agent-user");
        format!("macos:reddit-agent:v0.1.0 (by /u/{})", user)
    }

    async fn send_authed(
        &self,
        account_ctx: &AccountContext,
        method: reqwest::Method,
        path: &str,
        query: Option<&[(&str, &str)]>,
        body: Option<Value>,
        form_params: Option<&[(&str, &str)]>,
    ) -> Result<reqwest::Response, DriverError> {
        let alias = &account_ctx.account.alias;
        let token = self
            .auth
            .get_valid_access_token(alias, &self.client_id, &self.client_secret)
            .await
            .map_err(|e| {
                DriverError::Unauthorized(format!("Failed to obtain access token: {}", e))
            })?;

        let url = if path.starts_with("http://") || path.starts_with("https://") {
            path.to_string()
        } else {
            format!("{}{}", self.base_url, path)
        };

        let mut headers = HeaderMap::new();
        let ua = self.build_user_agent(account_ctx.account.reddit_username.as_deref());
        headers.insert(USER_AGENT, HeaderValue::from_str(&ua).unwrap());
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token.expose_secret())).unwrap(),
        );

        let mut req = self.client.request(method.clone(), &url).headers(headers);

        if let Some(q) = query {
            req = req.query(q);
        }
        if let Some(f) = form_params {
            req = req.form(f);
        } else if let Some(b) = body {
            req = req.json(&b);
        }

        let resp = req.send().await.map_err(|e| DriverError::Api {
            status: 0,
            message: format!("Network request failed: {}", e),
        })?;

        // Inspect and record rate limit headers
        let remaining: Option<f64> = resp
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());

        let reset: Option<u64> = resp
            .headers()
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());

        if let (Some(rem), Some(res)) = (remaining, reset) {
            let _ = self
                .storage
                .update_rate_limit(&account_ctx.account.id, rem, res);
            debug!(
                "Reddit rate-limit for {}: remaining={}, reset={}s",
                alias, rem, res
            );
        }

        let status = resp.status();
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = reset.unwrap_or(60);
            warn!(
                "Reddit rate limit reached for {}. Retry after {}s",
                alias, retry_after
            );
            return Err(DriverError::RateLimited(retry_after));
        }

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(DriverError::Unauthorized(format!(
                "Token rejected or expired for account {}",
                alias
            )));
        }

        if status == reqwest::StatusCode::FORBIDDEN {
            return Err(DriverError::MissingScope(format!(
                "Access forbidden (403) on {}. Check OAuth scopes.",
                path
            )));
        }

        if !status.is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(DriverError::Api {
                status: status.as_u16(),
                message: err_body,
            });
        }

        Ok(resp)
    }
}

#[async_trait]
impl Driver for RedditApiDriver {
    fn name(&self) -> &'static str {
        "reddit_api"
    }

    fn supports(&self, capability: &Capability) -> bool {
        match capability {
            Capability::GetIdentity
            | Capability::GetPreferences
            | Capability::SetPreferences
            | Capability::GetSubreddit
            | Capability::GetSubredditRules
            | Capability::GetPostRequirements
            | Capability::Search
            | Capability::GetPost
            | Capability::GetComments
            | Capability::CreatePost
            | Capability::EditPost
            | Capability::DeletePost
            | Capability::CreateComment
            | Capability::EditComment
            | Capability::DeleteComment
            | Capability::GetInbox
            | Capability::ManageSubscription
            | Capability::ManageSaved
            | Capability::ManageHidden
            | Capability::InspectAccountHealth => true,
            Capability::BrowserSettings => false,
        }
    }

    async fn execute(
        &self,
        capability: &Capability,
        account_ctx: &AccountContext,
        input: Value,
    ) -> Result<Value, DriverError> {
        match capability {
            Capability::GetIdentity => {
                let resp = self
                    .send_authed(
                        account_ctx,
                        reqwest::Method::GET,
                        "/api/v1/me",
                        None,
                        None,
                        None,
                    )
                    .await?;

                let identity: RedditIdentity = resp.json().await.map_err(|e| DriverError::Api {
                    status: 200,
                    message: format!("Malformed identity payload: {}", e),
                })?;

                // Optionally fetch karma
                let karma_resp = self
                    .send_authed(
                        account_ctx,
                        reqwest::Method::GET,
                        "/api/v1/me/karma",
                        None,
                        None,
                        None,
                    )
                    .await;

                let karma_data = match karma_resp {
                    Ok(r) => r.json::<KarmaResponse>().await.ok().map(|k| k.data),
                    Err(_) => None,
                };

                Ok(json!({
                    "identity": identity,
                    "karma_breakdown": karma_data,
                }))
            }

            Capability::GetPreferences => {
                let resp = self
                    .send_authed(
                        account_ctx,
                        reqwest::Method::GET,
                        "/api/v1/me/prefs",
                        None,
                        None,
                        None,
                    )
                    .await?;

                let prefs: RedditPreferences = resp.json().await.map_err(|e| DriverError::Api {
                    status: 200,
                    message: format!("Malformed preferences payload: {}", e),
                })?;

                Ok(json!(prefs))
            }

            Capability::SetPreferences => {
                let resp = self
                    .send_authed(
                        account_ctx,
                        reqwest::Method::PATCH,
                        "/api/v1/me/prefs",
                        None,
                        Some(input),
                        None,
                    )
                    .await?;

                let updated: Value = resp.json().await.map_err(|e| DriverError::Api {
                    status: 200,
                    message: format!("Failed to read patched preferences: {}", e),
                })?;

                Ok(updated)
            }

            Capability::GetSubreddit => {
                let sr = input["subreddit"].as_str().ok_or_else(|| {
                    DriverError::Internal("Missing 'subreddit' parameter".to_string())
                })?;
                let path = format!("/r/{}/about.json", sr);

                let resp = self
                    .send_authed(account_ctx, reqwest::Method::GET, &path, None, None, None)
                    .await?;

                let data: Value = resp.json().await.map_err(|e| DriverError::Api {
                    status: 200,
                    message: format!("Failed to read subreddit about: {}", e),
                })?;

                Ok(data["data"].clone())
            }

            Capability::GetSubredditRules => {
                let sr = input["subreddit"].as_str().ok_or_else(|| {
                    DriverError::Internal("Missing 'subreddit' parameter".to_string())
                })?;
                let path = format!("/r/{}/about/rules.json", sr);

                let resp = self
                    .send_authed(account_ctx, reqwest::Method::GET, &path, None, None, None)
                    .await?;

                let rules_resp: SubredditRulesResponse =
                    resp.json().await.map_err(|e| DriverError::Api {
                        status: 200,
                        message: format!("Failed to read subreddit rules: {}", e),
                    })?;

                Ok(json!(rules_resp.rules))
            }

            Capability::GetPostRequirements => {
                let sr = input["subreddit"].as_str().ok_or_else(|| {
                    DriverError::Internal("Missing 'subreddit' parameter".to_string())
                })?;
                let path = format!("/api/v1/{}/post_requirements", sr);

                let resp = self
                    .send_authed(account_ctx, reqwest::Method::GET, &path, None, None, None)
                    .await?;

                let reqs: PostRequirements = resp.json().await.map_err(|e| DriverError::Api {
                    status: 200,
                    message: format!("Failed to read post requirements: {}", e),
                })?;

                Ok(json!(reqs))
            }

            Capability::Search => {
                let sr = input["subreddit"].as_str().unwrap_or("all");
                let query = input["query"].as_str().ok_or_else(|| {
                    DriverError::Internal("Missing 'query' parameter".to_string())
                })?;
                let limit = input["limit"].as_u64().unwrap_or(25).to_string();
                let path = format!("/r/{}/search.json", sr);

                let query_params = [
                    ("q", query),
                    ("limit", &limit),
                    ("restrict_sr", if sr == "all" { "0" } else { "1" }),
                ];

                let resp = self
                    .send_authed(
                        account_ctx,
                        reqwest::Method::GET,
                        &path,
                        Some(&query_params),
                        None,
                        None,
                    )
                    .await?;

                let body: Value = resp.json().await.map_err(|e| DriverError::Api {
                    status: 200,
                    message: format!("Failed to read search response: {}", e),
                })?;

                Ok(body["data"]["children"].clone())
            }

            Capability::GetPost | Capability::GetComments => {
                let sr = input["subreddit"].as_str().unwrap_or("all");
                let post_id = input["post_id"].as_str().ok_or_else(|| {
                    DriverError::Internal("Missing 'post_id' parameter".to_string())
                })?;
                // Strip "t3_" if present
                let clean_id = post_id.trim_start_matches("t3_");
                let path = format!("/r/{}/comments/{}.json", sr, clean_id);

                let resp = self
                    .send_authed(account_ctx, reqwest::Method::GET, &path, None, None, None)
                    .await?;

                let data: Value = resp.json().await.map_err(|e| DriverError::Api {
                    status: 200,
                    message: format!("Failed to read comments/post data: {}", e),
                })?;

                Ok(data)
            }

            Capability::CreatePost => {
                let sr = input["subreddit"].as_str().ok_or_else(|| {
                    DriverError::Internal("Missing 'subreddit' parameter".to_string())
                })?;
                let title = input["title"].as_str().ok_or_else(|| {
                    DriverError::Internal("Missing 'title' parameter".to_string())
                })?;
                let kind = input["kind"].as_str().unwrap_or("self");
                let text = input["text"].as_str().unwrap_or("");
                let url = input["url"].as_str().unwrap_or("");
                let nsfw = if input["nsfw"].as_bool().unwrap_or(false) {
                    "true"
                } else {
                    "false"
                };
                let spoiler = if input["spoiler"].as_bool().unwrap_or(false) {
                    "true"
                } else {
                    "false"
                };

                let params = [
                    ("api_type", "json"),
                    ("sr", sr),
                    ("kind", kind),
                    ("title", title),
                    ("text", text),
                    ("url", url),
                    ("nsfw", nsfw),
                    ("spoiler", spoiler),
                    ("resubmit", "true"),
                ];

                let resp = self
                    .send_authed(
                        account_ctx,
                        reqwest::Method::POST,
                        "/api/submit",
                        None,
                        None,
                        Some(&params),
                    )
                    .await?;

                let body: Value = resp.json().await.map_err(|e| DriverError::Api {
                    status: 200,
                    message: format!("Failed to parse submit response: {}", e),
                })?;

                // Check for Reddit inline errors
                if let Some(errors) = body["json"]["errors"].as_array() {
                    if !errors.is_empty() {
                        return Err(DriverError::Api {
                            status: 400,
                            message: format!("Reddit rejected submission: {:?}", errors),
                        });
                    }
                }

                Ok(body["json"]["data"].clone())
            }

            Capability::CreateComment => {
                let parent_id = input["thing_id"].as_str().ok_or_else(|| {
                    DriverError::Internal("Missing 'thing_id' (parent ID) parameter".to_string())
                })?;
                let text = input["text"]
                    .as_str()
                    .ok_or_else(|| DriverError::Internal("Missing 'text' parameter".to_string()))?;

                let params = [
                    ("api_type", "json"),
                    ("thing_id", parent_id),
                    ("text", text),
                ];

                let resp = self
                    .send_authed(
                        account_ctx,
                        reqwest::Method::POST,
                        "/api/comment",
                        None,
                        None,
                        Some(&params),
                    )
                    .await?;

                let body: Value = resp.json().await.map_err(|e| DriverError::Api {
                    status: 200,
                    message: format!("Failed to parse comment response: {}", e),
                })?;

                if let Some(errors) = body["json"]["errors"].as_array() {
                    if !errors.is_empty() {
                        return Err(DriverError::Api {
                            status: 400,
                            message: format!("Reddit rejected comment: {:?}", errors),
                        });
                    }
                }

                Ok(body["json"]["data"].clone())
            }

            Capability::EditPost | Capability::EditComment => {
                let thing_id = input["thing_id"].as_str().ok_or_else(|| {
                    DriverError::Internal("Missing 'thing_id' parameter".to_string())
                })?;
                let text = input["text"]
                    .as_str()
                    .ok_or_else(|| DriverError::Internal("Missing 'text' parameter".to_string()))?;

                let params = [("api_type", "json"), ("thing_id", thing_id), ("text", text)];

                let resp = self
                    .send_authed(
                        account_ctx,
                        reqwest::Method::POST,
                        "/api/editusertext",
                        None,
                        None,
                        Some(&params),
                    )
                    .await?;

                let body: Value = resp.json().await.map_err(|e| DriverError::Api {
                    status: 200,
                    message: format!("Failed to parse edit response: {}", e),
                })?;

                Ok(body)
            }

            Capability::DeletePost | Capability::DeleteComment => {
                let id = input["id"]
                    .as_str()
                    .ok_or_else(|| DriverError::Internal("Missing 'id' parameter".to_string()))?;

                let params = [("id", id)];

                let resp = self
                    .send_authed(
                        account_ctx,
                        reqwest::Method::POST,
                        "/api/del",
                        None,
                        None,
                        Some(&params),
                    )
                    .await?;

                Ok(json!({"deleted": id, "success": resp.status().is_success()}))
            }

            Capability::GetInbox => {
                let filter = input["filter"].as_str().unwrap_or("unread");
                let path = format!("/message/{}.json", filter);

                let resp = self
                    .send_authed(account_ctx, reqwest::Method::GET, &path, None, None, None)
                    .await?;

                let body: Value = resp.json().await.map_err(|e| DriverError::Api {
                    status: 200,
                    message: format!("Failed to read inbox: {}", e),
                })?;

                Ok(body["data"]["children"].clone())
            }

            Capability::ManageSubscription => {
                let sr_name = input["subreddit"].as_str().ok_or_else(|| {
                    DriverError::Internal("Missing 'subreddit' parameter".to_string())
                })?;
                let action = input["action"].as_str().unwrap_or("sub"); // "sub" or "unsub"

                let params = [
                    ("action", action),
                    ("sr_name", sr_name),
                    ("skip_initial_defaults", "true"),
                ];

                let resp = self
                    .send_authed(
                        account_ctx,
                        reqwest::Method::POST,
                        "/api/subscribe",
                        None,
                        None,
                        Some(&params),
                    )
                    .await?;

                Ok(
                    json!({"subreddit": sr_name, "action": action, "success": resp.status().is_success()}),
                )
            }

            Capability::ManageSaved => {
                let id = input["id"]
                    .as_str()
                    .ok_or_else(|| DriverError::Internal("Missing 'id' parameter".to_string()))?;
                let unsave = input["unsave"].as_bool().unwrap_or(false);
                let path = if unsave { "/api/unsave" } else { "/api/save" };

                let params = [("id", id)];

                let resp = self
                    .send_authed(
                        account_ctx,
                        reqwest::Method::POST,
                        path,
                        None,
                        None,
                        Some(&params),
                    )
                    .await?;

                Ok(
                    json!({"id": id, "action": if unsave { "unsave" } else { "save" }, "success": resp.status().is_success()}),
                )
            }

            Capability::ManageHidden => {
                let id = input["id"]
                    .as_str()
                    .ok_or_else(|| DriverError::Internal("Missing 'id' parameter".to_string()))?;
                let unhide = input["unhide"].as_bool().unwrap_or(false);
                let path = if unhide { "/api/unhide" } else { "/api/hide" };

                let params = [("id", id)];

                let resp = self
                    .send_authed(
                        account_ctx,
                        reqwest::Method::POST,
                        path,
                        None,
                        None,
                        Some(&params),
                    )
                    .await?;

                Ok(
                    json!({"id": id, "action": if unhide { "unhide" } else { "hide" }, "success": resp.status().is_success()}),
                )
            }

            Capability::InspectAccountHealth => {
                let report = self.health_check(account_ctx).await?;
                Ok(json!(report))
            }

            Capability::BrowserSettings => {
                Err(DriverError::Unsupported(Capability::BrowserSettings))
            }
        }
    }

    async fn health_check(
        &self,
        account_ctx: &AccountContext,
    ) -> Result<HealthReport, DriverError> {
        let alias = &account_ctx.account.alias;
        let mut issues = Vec::new();
        let mut status = "healthy".to_string();

        // Check 1: Keyring refresh token presence
        match self.auth.get_refresh_token(alias) {
            Ok(_) => {}
            Err(_) => {
                status = "unhealthy".to_string();
                issues.push(HealthIssue {
                    code: "NO_REFRESH_TOKEN".to_string(),
                    capability: "auth".to_string(),
                    message: format!(
                        "No refresh token stored in OS keychain for account '{}'",
                        alias
                    ),
                    remediation: format!(
                        "Run `reddit-agent account oauth {}` to authorize.",
                        alias
                    ),
                });
            }
        }

        // Check 2: API connectivity & identity verification
        if status != "unhealthy" {
            match self
                .send_authed(
                    account_ctx,
                    reqwest::Method::GET,
                    "/api/v1/me",
                    None,
                    None,
                    None,
                )
                .await
            {
                Ok(resp) => {
                    if let Ok(identity) = resp.json::<RedditIdentity>().await {
                        if let Some(expected_user) = &account_ctx.account.reddit_username {
                            if !identity.name.eq_ignore_ascii_case(expected_user) {
                                status = "degraded".to_string();
                                issues.push(HealthIssue {
                                    code: "USERNAME_MISMATCH".to_string(),
                                    capability: "identity".to_string(),
                                    message: format!("Authenticated username 'u/{}' does not match registered 'u/{}'", identity.name, expected_user),
                                    remediation: "Re-run identity verification to update local account registry.".to_string(),
                                });
                            }
                        }
                    }
                }
                Err(DriverError::MissingScope(msg)) => {
                    status = "degraded".to_string();
                    issues.push(HealthIssue {
                        code: "MISSING_SCOPE".to_string(),
                        capability: "identity".to_string(),
                        message: msg,
                        remediation: "Reauthorize account with required identity scope."
                            .to_string(),
                    });
                }
                Err(DriverError::Unauthorized(msg)) => {
                    status = "unhealthy".to_string();
                    issues.push(HealthIssue {
                        code: "OAUTH_REVOKED".to_string(),
                        capability: "auth".to_string(),
                        message: msg,
                        remediation: "OAuth access token was revoked. Run reauthorization."
                            .to_string(),
                    });
                }
                Err(DriverError::RateLimited(sec)) => {
                    status = "degraded".to_string();
                    issues.push(HealthIssue {
                        code: "RATE_LIMITED".to_string(),
                        capability: "api".to_string(),
                        message: format!("Currently rate-limited. Retry after {} seconds.", sec),
                        remediation: "Wait for rate-limit reset before executing further requests."
                            .to_string(),
                    });
                }
                Err(e) => {
                    status = "degraded".to_string();
                    issues.push(HealthIssue {
                        code: "API_UNREACHABLE".to_string(),
                        capability: "network".to_string(),
                        message: e.to_string(),
                        remediation: "Check network connection or Reddit platform status."
                            .to_string(),
                    });
                }
            }
        }

        // Check 3: Browser profile state if enabled
        if account_ctx.account.browser_enabled {
            if let Some(path) = &account_ctx.browser_profile_path {
                if !std::path::Path::new(path).exists() {
                    if status == "healthy" {
                        status = "degraded".to_string();
                    }
                    issues.push(HealthIssue {
                        code: "BROWSER_PROFILE_MISSING".to_string(),
                        capability: "browser".to_string(),
                        message: format!("Browser profile directory does not exist: {}", path),
                        remediation: format!(
                            "Run `reddit-agent account login {}` to initialize browser profile.",
                            alias
                        ),
                    });
                }
            }
        }

        Ok(HealthReport {
            account: alias.clone(),
            reddit_username: account_ctx.account.reddit_username.clone(),
            status,
            issues,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::MemoryKeyring;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_reddit_api_driver_identity_mock() {
        let mock_server = MockServer::start().await;

        let identity_json = json!({
            "name": "rust_tester",
            "id": "abc1234",
            "created_utc": 1600000000.0,
            "link_karma": 150,
            "comment_karma": 300,
            "is_mod": false,
            "is_gold": false,
            "over_18": false,
            "has_verified_email": true
        });

        Mock::given(method("GET"))
            .and(path("/api/v1/me"))
            .and(header("authorization", "Bearer test_access_token_123"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(&identity_json)
                    .insert_header("x-ratelimit-remaining", "98.0")
                    .insert_header("x-ratelimit-reset", "45"),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v1/me/karma"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
            .mount(&mock_server)
            .await;

        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let keyring = Arc::new(MemoryKeyring::new());
        let client = reqwest::Client::new();
        let auth = Arc::new(AuthManager::new(keyring.clone(), client.clone()));

        // Add account and set pre-cached token to avoid real OAuth refresh network calls
        let acc = storage.add_account("tester").unwrap();
        auth.set_cached_access_token("tester", "test_access_token_123", 3600)
            .await;
        let ctx = AccountContext {
            account: acc,
            scopes: vec!["identity".to_string()],
            browser_profile_path: None,
        };

        let driver = RedditApiDriver::new(
            client,
            auth,
            storage.clone(),
            "dummy_id".to_string(),
            "dummy_secret".to_string(),
        )
        .with_base_url(mock_server.uri());

        let res = driver
            .execute(&Capability::GetIdentity, &ctx, json!({}))
            .await
            .unwrap();
        assert_eq!(res["identity"]["name"], "rust_tester");
        assert_eq!(res["identity"]["link_karma"], 150);

        // Verify rate limit was updated in SQLite
        let rl = storage.get_rate_limit(&ctx.account.id).unwrap().unwrap();
        assert_eq!(rl.remaining, 98.0);
        assert_eq!(rl.reset_seconds, 45);
    }

    #[tokio::test]
    async fn test_reddit_api_driver_preferences_patch_mock() {
        let mock_server = MockServer::start().await;

        let patch_body = json!({
            "default_comment_sort": "new",
            "show_presence": false
        });

        Mock::given(method("PATCH"))
            .and(path("/api/v1/me/prefs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&patch_body))
            .mount(&mock_server)
            .await;

        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let keyring = Arc::new(MemoryKeyring::new());
        let client = reqwest::Client::new();
        let auth = Arc::new(AuthManager::new(keyring, client.clone()));

        let acc = storage.add_account("prefs_tester").unwrap();
        auth.set_cached_access_token("prefs_tester", "test_access_token_123", 3600)
            .await;
        let ctx = AccountContext {
            account: acc,
            scopes: vec!["account".to_string()],
            browser_profile_path: None,
        };

        let driver = RedditApiDriver::new(
            client,
            auth,
            storage,
            "dummy_id".to_string(),
            "dummy_secret".to_string(),
        )
        .with_base_url(mock_server.uri());

        let res = driver
            .execute(&Capability::SetPreferences, &ctx, patch_body)
            .await
            .unwrap();
        assert_eq!(res["default_comment_sort"], "new");
        assert_eq!(res["show_presence"], false);
    }
}
