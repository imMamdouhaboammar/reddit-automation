pub mod secret_manager;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn};
use url::Url;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Keyring error: {0}")]
    Keyring(String),
    #[error("OAuth error: {0}")]
    OAuth(String),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("State mismatch in OAuth callback")]
    StateMismatch,
    #[error("Callback listener timed out")]
    Timeout,
    #[error("No refresh token found for account: {0}")]
    NoRefreshToken(String),
    #[error("Invalid token response from Reddit: {0}")]
    InvalidTokenResponse(String),
    #[error("I/O error during callback: {0}")]
    Io(#[from] std::io::Error),
}

/// A wrapper around sensitive tokens that redacts its content on Display and Debug
#[derive(Clone, PartialEq, Eq)]
pub struct SecretToken(String);

impl SecretToken {
    pub fn new(secret: impl Into<String>) -> Self {
        Self(secret.into())
    }

    /// Explicit accessor to retrieve the raw string value strictly for HTTP Authorization headers
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl fmt::Display for SecretToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl Serialize for SecretToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("[REDACTED]")
    }
}

/// Abstract storage for secrets (OS Keyring vs In-Memory for testing)
pub trait CredentialStore: Send + Sync {
    fn get_secret(&self, service: &str, account: &str) -> Result<SecretToken, AuthError>;
    fn set_secret(&self, service: &str, account: &str, secret: &str) -> Result<(), AuthError>;
    fn delete_secret(&self, service: &str, account: &str) -> Result<(), AuthError>;
}

pub struct SystemKeyring;

impl CredentialStore for SystemKeyring {
    fn get_secret(&self, service: &str, account: &str) -> Result<SecretToken, AuthError> {
        let entry =
            keyring::Entry::new(service, account).map_err(|e| AuthError::Keyring(e.to_string()))?;
        let pass = entry
            .get_password()
            .map_err(|e| AuthError::Keyring(e.to_string()))?;
        Ok(SecretToken::new(pass))
    }

    fn set_secret(&self, service: &str, account: &str, secret: &str) -> Result<(), AuthError> {
        let entry =
            keyring::Entry::new(service, account).map_err(|e| AuthError::Keyring(e.to_string()))?;
        entry
            .set_password(secret)
            .map_err(|e| AuthError::Keyring(e.to_string()))?;
        Ok(())
    }

    fn delete_secret(&self, service: &str, account: &str) -> Result<(), AuthError> {
        let entry =
            keyring::Entry::new(service, account).map_err(|e| AuthError::Keyring(e.to_string()))?;
        entry
            .delete_credential()
            .map_err(|e| AuthError::Keyring(e.to_string()))?;
        Ok(())
    }
}

/// In-memory credential store for fast isolated unit testing
#[derive(Default, Clone)]
pub struct MemoryKeyring {
    store: Arc<std::sync::RwLock<HashMap<String, String>>>,
}

impl MemoryKeyring {
    pub fn new() -> Self {
        Self {
            store: Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }
}

impl CredentialStore for MemoryKeyring {
    fn get_secret(&self, service: &str, account: &str) -> Result<SecretToken, AuthError> {
        let key = format!("{}:{}", service, account);
        let store = self
            .store
            .read()
            .map_err(|e| AuthError::Keyring(e.to_string()))?;
        match store.get(&key) {
            Some(v) => Ok(SecretToken::new(v.clone())),
            None => Err(AuthError::Keyring(format!(
                "Secret not found for key: {}",
                key
            ))),
        }
    }

    fn set_secret(&self, service: &str, account: &str, secret: &str) -> Result<(), AuthError> {
        let key = format!("{}:{}", service, account);
        let mut store = self
            .store
            .write()
            .map_err(|e| AuthError::Keyring(e.to_string()))?;
        store.insert(key, secret.to_string());
        Ok(())
    }

    fn delete_secret(&self, service: &str, account: &str) -> Result<(), AuthError> {
        let key = format!("{}:{}", service, account);
        let mut store = self
            .store
            .write()
            .map_err(|e| AuthError::Keyring(e.to_string()))?;
        store.remove(&key);
        Ok(())
    }
}

#[derive(Clone)]
struct CachedToken {
    access_token: SecretToken,
    expires_at: Instant,
}

#[derive(Deserialize, Debug)]
pub struct RedditTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: String,
    pub refresh_token: Option<String>,
}

pub struct AuthManager {
    credential_store: Arc<dyn CredentialStore>,
    client: reqwest::Client,
    token_cache: Arc<RwLock<HashMap<String, CachedToken>>>,
    refresh_locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
}

impl AuthManager {
    pub fn new(credential_store: Arc<dyn CredentialStore>, client: reqwest::Client) -> Self {
        Self {
            credential_store,
            client,
            token_cache: Arc::new(RwLock::new(HashMap::new())),
            refresh_locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn new_system(client: reqwest::Client) -> Self {
        Self::new(Arc::new(SystemKeyring), client)
    }

    pub fn credential_store(&self) -> Arc<dyn CredentialStore> {
        self.credential_store.clone()
    }

    pub fn keyring_service() -> &'static str {
        "reddit-agent"
    }

    pub fn keyring_account(alias: &str) -> String {
        format!("oauth_refresh:{}", alias)
    }

    pub fn keyring_cookies_account(alias: &str) -> String {
        format!("browser_cookies:{}", alias)
    }

    pub fn store_refresh_token(&self, alias: &str, refresh_token: &str) -> Result<(), AuthError> {
        self.credential_store.set_secret(
            Self::keyring_service(),
            &Self::keyring_account(alias),
            refresh_token,
        )
    }

    pub fn get_refresh_token(&self, alias: &str) -> Result<SecretToken, AuthError> {
        self.credential_store
            .get_secret(Self::keyring_service(), &Self::keyring_account(alias))
            .map_err(|_| AuthError::NoRefreshToken(alias.to_string()))
    }

    pub fn store_cookies(&self, alias: &str, cookies_json: &str) -> Result<(), AuthError> {
        self.credential_store.set_secret(
            Self::keyring_service(),
            &Self::keyring_cookies_account(alias),
            cookies_json,
        )
    }

    pub fn get_cookies(&self, alias: &str) -> Result<SecretToken, AuthError> {
        self.credential_store
            .get_secret(
                Self::keyring_service(),
                &Self::keyring_cookies_account(alias),
            )
            .map_err(|e| {
                AuthError::Keyring(format!("No cookies found for account '{}': {}", alias, e))
            })
    }

    pub fn delete_cookies(&self, alias: &str) -> Result<(), AuthError> {
        self.credential_store.delete_secret(
            Self::keyring_service(),
            &Self::keyring_cookies_account(alias),
        )
    }

    pub fn generate_authorize_url(
        client_id: &str,
        redirect_uri: &str,
        state: &str,
        scopes: &[&str],
    ) -> String {
        let scope_str = scopes.join(" ");
        format!(
            "https://www.reddit.com/api/v1/authorize?client_id={}&response_type=code&state={}&redirect_uri={}&duration=permanent&scope={}",
            urlencoding::encode(client_id),
            urlencoding::encode(state),
            urlencoding::encode(redirect_uri),
            urlencoding::encode(&scope_str)
        )
    }

    /// Spin up localhost callback server and wait for OAuth response
    pub async fn wait_for_callback(
        port: u16,
        expected_state: &str,
        timeout_duration: Duration,
    ) -> Result<String, AuthError> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        info!("OAuth callback listener started on 127.0.0.1:{}", port);

        let accept_future = async {
            let (mut socket, _) = listener.accept().await?;
            let mut buf = [0u8; 2048];
            let n = socket.read(&mut buf).await?;
            let req = String::from_utf8_lossy(&buf[..n]);

            // Parse GET /callback?...
            let first_line = req.lines().next().unwrap_or("");
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.len() < 2 {
                return Err(AuthError::OAuth("Malformed HTTP request".to_string()));
            }

            let path = parts[1];
            let dummy_url = format!("http://127.0.0.1:{}", port) + path;
            let parsed_url = Url::parse(&dummy_url).map_err(|e| AuthError::OAuth(e.to_string()))?;

            let mut code = None;
            let mut state = None;
            let mut error = None;

            for (k, v) in parsed_url.query_pairs() {
                if k == "code" {
                    code = Some(v.to_string());
                } else if k == "state" {
                    state = Some(v.to_string());
                } else if k == "error" {
                    error = Some(v.to_string());
                }
            }

            if let Some(err) = error {
                let html = format!("HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\n\r\n<h1>Reddit OAuth Authorization Denied: {}</h1>", err);
                socket.write_all(html.as_bytes()).await?;
                return Err(AuthError::OAuth(format!(
                    "Reddit denied authorization: {}",
                    err
                )));
            }

            if state.as_deref() != Some(expected_state) {
                let html = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\n\r\n<h1>State verification failed. CSRF detected.</h1>";
                socket.write_all(html.as_bytes()).await?;
                return Err(AuthError::StateMismatch);
            }

            let code = match code {
                Some(c) => c,
                None => {
                    let html = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\n\r\n<h1>Missing code parameter</h1>";
                    socket.write_all(html.as_bytes()).await?;
                    return Err(AuthError::OAuth("Missing code parameter".to_string()));
                }
            };

            let response_body = r#"<!DOCTYPE html>
<html>
<head><title>Authorization Successful</title><style>body{font-family:sans-serif;text-align:center;padding-top:50px;background:#f8f9fa}h1{color:#ff4500}</style></head>
<body>
<h1>Reddit Authorization Successful!</h1>
<p>You can close this browser window and return to your terminal.</p>
</body>
</html>"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            socket.write_all(resp.as_bytes()).await?;
            socket.flush().await?;

            Ok(code)
        };

        tokio::time::timeout(timeout_duration, accept_future)
            .await
            .map_err(|_| AuthError::Timeout)?
    }

    /// Exchange authorization code for initial access and refresh tokens
    pub async fn exchange_code(
        &self,
        client_id: &str,
        client_secret: &str,
        code: &str,
        redirect_uri: &str,
    ) -> Result<RedditTokenResponse, AuthError> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ];

        let req = self
            .client
            .post("https://www.reddit.com/api/v1/access_token")
            .basic_auth(client_id, Some(client_secret))
            .header("User-Agent", "reddit-agent:v0.1.0 (local-operator)")
            .form(&params);

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AuthError::OAuth(format!(
                "Token exchange failed ({}): {}",
                status, body
            )));
        }

        let token_resp: RedditTokenResponse = resp.json().await?;
        Ok(token_resp)
    }

    /// Refresh token with per-account async mutex lock and memory cache
    pub async fn get_valid_access_token(
        &self,
        alias: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Result<SecretToken, AuthError> {
        // Fast path: check in-memory cache
        {
            let cache = self.token_cache.read().await;
            if let Some(cached) = cache.get(alias) {
                // Ensure at least 60 seconds buffer before expiration
                if cached.expires_at > Instant::now() + Duration::from_secs(60) {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        // Get or create per-account lock
        let account_lock = {
            let mut locks = self.refresh_locks.lock().await;
            locks
                .entry(alias.to_string())
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };

        let _guard = account_lock.lock().await;

        // Double check cache under lock
        {
            let cache = self.token_cache.read().await;
            if let Some(cached) = cache.get(alias) {
                if cached.expires_at > Instant::now() + Duration::from_secs(60) {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        let refresh_token = self.get_refresh_token(alias)?;

        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.expose_secret()),
        ];

        let req = self
            .client
            .post("https://www.reddit.com/api/v1/access_token")
            .basic_auth(client_id, Some(client_secret))
            .header("User-Agent", "reddit-agent:v0.1.0 (local-operator)")
            .form(&params);

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            warn!(
                "Reddit token refresh failed for account {}: {}",
                alias, status
            );
            return Err(AuthError::OAuth(format!(
                "Token refresh failed ({}): {}",
                status, body
            )));
        }

        let token_resp: RedditTokenResponse = resp.json().await?;
        let secret = SecretToken::new(token_resp.access_token);
        let expires_in_sec = token_resp.expires_in.max(300);
        let expires_at = Instant::now() + Duration::from_secs(expires_in_sec);

        {
            let mut cache = self.token_cache.write().await;
            cache.insert(
                alias.to_string(),
                CachedToken {
                    access_token: secret.clone(),
                    expires_at,
                },
            );
        }

        // If Reddit rotated refresh token, save the new one
        if let Some(new_refresh) = token_resp.refresh_token {
            let _ = self.store_refresh_token(alias, &new_refresh);
        }

        Ok(secret)
    }

    pub async fn set_cached_access_token(&self, alias: &str, token: &str, expires_in_secs: u64) {
        let mut cache = self.token_cache.write().await;
        cache.insert(
            alias.to_string(),
            CachedToken {
                access_token: SecretToken::new(token),
                expires_at: Instant::now() + Duration::from_secs(expires_in_secs),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_token_redaction() {
        let token = SecretToken::new("super_secret_reddit_token_12345");
        let display = format!("{}", token);
        let debug = format!("{:?}", token);
        let json = serde_json::to_string(&token).unwrap();

        assert_eq!(display, "[REDACTED]");
        assert_eq!(debug, "[REDACTED]");
        assert_eq!(json, "\"[REDACTED]\"");
        assert_eq!(token.expose_secret(), "super_secret_reddit_token_12345");
    }

    #[test]
    fn test_memory_keyring_isolation() {
        let keyring = MemoryKeyring::new();
        keyring
            .set_secret("reddit-agent", "oauth_refresh:personal", "refresh_tok_1")
            .unwrap();
        keyring
            .set_secret("reddit-agent", "oauth_refresh:company", "refresh_tok_2")
            .unwrap();

        let tok1 = keyring
            .get_secret("reddit-agent", "oauth_refresh:personal")
            .unwrap();
        let tok2 = keyring
            .get_secret("reddit-agent", "oauth_refresh:company")
            .unwrap();

        assert_eq!(tok1.expose_secret(), "refresh_tok_1");
        assert_eq!(tok2.expose_secret(), "refresh_tok_2");
        assert_ne!(tok1.expose_secret(), tok2.expose_secret());
    }

    #[test]
    fn test_generate_authorize_url() {
        let url = AuthManager::generate_authorize_url(
            "client123",
            "http://127.0.0.1:8080/callback",
            "state_abc",
            &["identity", "read", "submit"],
        );

        assert!(url.contains("client_id=client123"));
        assert!(url.contains("state=state_abc"));
        assert!(url.contains("duration=permanent"));
        assert!(url.contains("identity"));
        assert!(url.contains("submit"));
    }
}
