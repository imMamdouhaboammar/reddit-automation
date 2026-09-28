use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

use crate::accounts::{AccountContext, AccountManager};
use crate::actions::ActionManager;
use crate::drivers::{Capability, DriverError};
use crate::reddit::PostRequirements;
use crate::router::CapabilityRouter;
use crate::storage::Storage;

#[derive(Error, Debug)]
pub enum IssueError {
    #[error("Driver error: {0}")]
    Driver(#[from] DriverError),
    #[error("Storage error: {0}")]
    Storage(#[from] crate::storage::StorageError),
    #[error("Invalid remediation: {0}")]
    InvalidRemediation(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IssueCategory {
    AuthRevoked,
    MissingScope,
    BrowserSessionExpired,
    PreferenceDrift,
    PostRejected,
    PostRemoved,
    RateLimited,
    SecurityChallenge,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisReport {
    pub account: String,
    pub category: IssueCategory,
    pub code: String,
    pub summary: String,
    pub evidence: Value,
    pub human_action_mandatory: bool,
    pub remediation_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairPlan {
    pub id: String,
    pub account: String,
    pub issue_code: String,
    pub remediation: String,
    pub is_safe_automatic: bool,
    pub requires_human: bool,
    pub repair_action: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairResult {
    pub repair_id: String,
    pub status: String, // "applied", "human_action_required", "failed"
    pub details: String,
    pub data: Option<Value>,
}

pub struct IssueEngine {
    router: Arc<CapabilityRouter>,
    #[allow(dead_code)]
    account_manager: Arc<AccountManager>,
    #[allow(dead_code)]
    action_manager: Arc<ActionManager>,
    #[allow(dead_code)]
    storage: Arc<Storage>,
    #[allow(dead_code)]
    client_id: String,
}

impl IssueEngine {
    pub fn new(
        router: Arc<CapabilityRouter>,
        account_manager: Arc<AccountManager>,
        action_manager: Arc<ActionManager>,
        storage: Arc<Storage>,
        client_id: String,
    ) -> Self {
        Self {
            router,
            account_manager,
            action_manager,
            storage,
            client_id,
        }
    }

    /// Diagnoses an account's state and detects known issues
    pub async fn diagnose_account(
        &self,
        account_ctx: &AccountContext,
    ) -> Result<DiagnosisReport, IssueError> {
        let alias = &account_ctx.account.alias;
        let health = self.router.health_check(account_ctx).await?;

        // 1. Check for revoked OAuth
        if let Some(issue) = health
            .issues
            .iter()
            .find(|i| i.code == "OAUTH_REVOKED" || i.code == "NO_REFRESH_TOKEN")
        {
            return Ok(DiagnosisReport {
                account: alias.clone(),
                category: IssueCategory::AuthRevoked,
                code: issue.code.clone(),
                summary: issue.message.clone(),
                evidence: json!({"health_issue": issue}),
                human_action_mandatory: true,
                remediation_code: "START_OAUTH_FLOW".to_string(),
            });
        }

        // 2. Check missing scopes
        if let Some(issue) = health.issues.iter().find(|i| i.code == "MISSING_SCOPE") {
            return Ok(DiagnosisReport {
                account: alias.clone(),
                category: IssueCategory::MissingScope,
                code: "MISSING_SCOPE".to_string(),
                summary: issue.message.clone(),
                evidence: json!({"current_scopes": account_ctx.scopes}),
                human_action_mandatory: true,
                remediation_code: "REAUTH_WITH_SCOPES".to_string(),
            });
        }

        // 3. Check rate limiting
        if let Some(issue) = health.issues.iter().find(|i| i.code == "RATE_LIMITED") {
            return Ok(DiagnosisReport {
                account: alias.clone(),
                category: IssueCategory::RateLimited,
                code: "RATE_LIMITED".to_string(),
                summary: issue.message.clone(),
                evidence: json!({"details": issue.message}),
                human_action_mandatory: false,
                remediation_code: "WAIT_RATE_LIMIT_RESET".to_string(),
            });
        }

        // 4. Check browser session if browser enabled
        if let Some(issue) = health
            .issues
            .iter()
            .find(|i| i.code == "BROWSER_SESSION_EXPIRED" || i.code == "BROWSER_PROFILE_MISSING")
        {
            return Ok(DiagnosisReport {
                account: alias.clone(),
                category: IssueCategory::BrowserSessionExpired,
                code: "BROWSER_SESSION_EXPIRED".to_string(),
                summary: issue.message.clone(),
                evidence: json!({"profile_path": account_ctx.browser_profile_path}),
                human_action_mandatory: true,
                remediation_code: "OPEN_BROWSER_LOGIN".to_string(),
            });
        }

        // Check inbox messages for security challenges or account suspensions
        let inbox_res = self
            .router
            .execute(
                &Capability::GetInbox,
                account_ctx,
                json!({"filter": "messages"}),
            )
            .await;
        if let Ok(inbox_val) = inbox_res {
            if let Some(messages) = inbox_val.as_array() {
                for msg in messages {
                    let subject = msg["data"]["subject"].as_str().unwrap_or("").to_lowercase();
                    let body = msg["data"]["body"].as_str().unwrap_or("").to_lowercase();

                    if subject.contains("suspended")
                        || body.contains("account has been suspended")
                        || subject.contains("security alert")
                        || body.contains("locked")
                    {
                        return Ok(DiagnosisReport {
                            account: alias.clone(),
                            category: IssueCategory::SecurityChallenge,
                            code: "ACCOUNT_LOCKED_OR_SUSPENDED".to_string(),
                            summary: "Reddit platform security notice or suspension detected in account inbox.".to_string(),
                            evidence: json!({"subject": subject, "body_preview": &body[..body.len().min(300)]}),
                            human_action_mandatory: true,
                            remediation_code: "HUMAN_APPEAL_CHECKPOINT".to_string(),
                        });
                    }
                }
            }
        }

        // Healthy
        Ok(DiagnosisReport {
            account: alias.clone(),
            category: IssueCategory::Unknown,
            code: "HEALTHY".to_string(),
            summary: format!(
                "Account '{}' is fully operational with zero detected issues.",
                alias
            ),
            evidence: json!({"status": "healthy"}),
            human_action_mandatory: false,
            remediation_code: "NONE".to_string(),
        })
    }

    /// Diagnoses a rejected post by analyzing the API error against subreddit rules and requirements
    pub async fn diagnose_post_rejection(
        &self,
        account_ctx: &AccountContext,
        subreddit: &str,
        error_message: &str,
        post_payload: &Value,
    ) -> Result<DiagnosisReport, IssueError> {
        let alias = &account_ctx.account.alias;

        // Fetch post requirements for this subreddit
        let reqs_res = self
            .router
            .execute(
                &Capability::GetPostRequirements,
                account_ctx,
                json!({"subreddit": subreddit}),
            )
            .await;

        let mut violations = Vec::new();
        if let Ok(reqs_val) = reqs_res {
            if let Ok(reqs) = serde_json::from_value::<PostRequirements>(reqs_val) {
                let title = post_payload["title"].as_str().unwrap_or("");
                let body = post_payload["text"].as_str().unwrap_or("");
                let flair_id = post_payload["flair_id"].as_str();

                violations =
                    ActionManager::validate_post_requirements(&reqs, title, body, flair_id);
            }
        }

        let summary = if !violations.is_empty() {
            format!(
                "Post rejected because it violates r/{} submission requirements: {}",
                subreddit,
                violations.join("; ")
            )
        } else {
            format!("Post submission rejected by Reddit API: {}", error_message)
        };

        Ok(DiagnosisReport {
            account: alias.clone(),
            category: IssueCategory::PostRejected,
            code: "POST_REJECTED".to_string(),
            summary,
            evidence: json!({
                "subreddit": subreddit,
                "api_error": error_message,
                "detected_violations": violations,
                "payload_preview": post_payload
            }),
            human_action_mandatory: false,
            remediation_code: "CORRECT_POST_PAYLOAD".to_string(),
        })
    }

    /// Plans a deterministic remediation for a diagnosed issue
    pub fn plan_repair(&self, diagnosis: &DiagnosisReport) -> Result<RepairPlan, IssueError> {
        let id = format!("rep_{}", Uuid::new_v4().simple());
        let account = diagnosis.account.clone();

        match diagnosis.remediation_code.as_str() {
            "START_OAUTH_FLOW" => Ok(RepairPlan {
                id,
                account: account.clone(),
                issue_code: diagnosis.code.clone(),
                remediation: "Initialize interactive OAuth consent flow in browser.".to_string(),
                is_safe_automatic: false,
                requires_human: true,
                repair_action: "start_oauth".to_string(),
                parameters: json!({"alias": account}),
            }),

            "OPEN_BROWSER_LOGIN" => Ok(RepairPlan {
                id,
                account: account.clone(),
                issue_code: diagnosis.code.clone(),
                remediation: "Open visible Chrome window for manual authentication.".to_string(),
                is_safe_automatic: false,
                requires_human: true,
                repair_action: "browser_login".to_string(),
                parameters: json!({"alias": account}),
            }),

            "HUMAN_APPEAL_CHECKPOINT" => Ok(RepairPlan {
                id,
                account: account.clone(),
                issue_code: diagnosis.code.clone(),
                remediation: "Account requires human appeal or security verification on Reddit. Halting automation.".to_string(),
                is_safe_automatic: false,
                requires_human: true,
                repair_action: "open_appeal_page".to_string(),
                parameters: json!({"url": "https://www.reddit.com/appeals"}),
            }),

            "WAIT_RATE_LIMIT_RESET" => {
                let wait_secs = diagnosis.evidence["details"].as_str()
                    .and_then(|s| s.split_whitespace().find_map(|w| w.parse::<u64>().ok()))
                    .unwrap_or(60);

                Ok(RepairPlan {
                    id,
                    account,
                    issue_code: diagnosis.code.clone(),
                    remediation: format!("Wait {} seconds for Reddit rate limit counter to reset.", wait_secs),
                    is_safe_automatic: true,
                    requires_human: false,
                    repair_action: "backoff_wait".to_string(),
                    parameters: json!({"wait_seconds": wait_secs}),
                })
            }

            "CORRECT_POST_PAYLOAD" => Ok(RepairPlan {
                id,
                account,
                issue_code: diagnosis.code.clone(),
                remediation: "Review detected violations, adjust title/body/flair, and prepare a corrected action plan.".to_string(),
                is_safe_automatic: false,
                requires_human: true,
                repair_action: "adjust_post_draft".to_string(),
                parameters: diagnosis.evidence.clone(),
            }),

            "PATCH_PREFERENCE" => Ok(RepairPlan {
                id,
                account,
                issue_code: diagnosis.code.clone(),
                remediation: "Apply requested preference correction using Reddit API driver.".to_string(),
                is_safe_automatic: true,
                requires_human: false,
                repair_action: "patch_preference".to_string(),
                parameters: diagnosis.evidence["desired_preference"].clone(),
            }),

            _ => Ok(RepairPlan {
                id,
                account,
                issue_code: diagnosis.code.clone(),
                remediation: "No remediation required. System is healthy.".to_string(),
                is_safe_automatic: true,
                requires_human: false,
                repair_action: "noop".to_string(),
                parameters: json!({}),
            }),
        }
    }

    /// Executes safe automated remediations; halts and preserves safety for human-required steps
    pub async fn apply_safe_repair(
        &self,
        account_ctx: &AccountContext,
        plan: &RepairPlan,
    ) -> Result<RepairResult, IssueError> {
        if plan.requires_human {
            return Ok(RepairResult {
                repair_id: plan.id.clone(),
                status: "human_action_required".to_string(),
                details: format!(
                    "Remediation for '{}' requires human interaction. Action halted: {}",
                    plan.issue_code, plan.remediation
                ),
                data: Some(plan.parameters.clone()),
            });
        }

        match plan.repair_action.as_str() {
            "patch_preference" => {
                let res = self
                    .router
                    .execute(
                        &Capability::SetPreferences,
                        account_ctx,
                        plan.parameters.clone(),
                    )
                    .await?;
                Ok(RepairResult {
                    repair_id: plan.id.clone(),
                    status: "applied".to_string(),
                    details: "Successfully patched preference via Reddit API.".to_string(),
                    data: Some(res),
                })
            }

            "backoff_wait" => {
                let secs = plan.parameters["wait_seconds"].as_u64().unwrap_or(5);
                tokio::time::sleep(std::time::Duration::from_secs(secs.min(30))).await;
                Ok(RepairResult {
                    repair_id: plan.id.clone(),
                    status: "applied".to_string(),
                    details: format!("Waited {} seconds for rate limit reset.", secs),
                    data: None,
                })
            }

            "noop" => Ok(RepairResult {
                repair_id: plan.id.clone(),
                status: "applied".to_string(),
                details: "No repair needed.".to_string(),
                data: None,
            }),

            other => Err(IssueError::InvalidRemediation(format!(
                "Unknown repair action: {}",
                other
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditLogger;
    use crate::auth::{AuthManager, MemoryKeyring};
    use crate::drivers::{Driver, HealthReport};
    use serde_json::json;

    struct MockRouterDriver;
    #[async_trait::async_trait]
    impl Driver for MockRouterDriver {
        fn name(&self) -> &'static str {
            "mock_router"
        }
        fn supports(&self, _cap: &Capability) -> bool {
            true
        }
        async fn execute(
            &self,
            _cap: &Capability,
            _ctx: &AccountContext,
            in_val: Value,
        ) -> Result<Value, DriverError> {
            Ok(in_val)
        }
        async fn health_check(&self, ctx: &AccountContext) -> Result<HealthReport, DriverError> {
            Ok(HealthReport {
                account: ctx.account.alias.clone(),
                reddit_username: ctx.account.reddit_username.clone(),
                status: "healthy".to_string(),
                issues: vec![],
            })
        }
    }

    #[tokio::test]
    async fn test_issue_diagnosis_post_rejection() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let keyring = Arc::new(MemoryKeyring::new());
        let client = reqwest::Client::new();
        let auth = Arc::new(AuthManager::new(keyring, client));
        let account_mgr = Arc::new(AccountManager::new(storage.clone(), auth));

        let _acc = account_mgr.add_account("test_account").unwrap();
        let ctx = account_mgr
            .get_account_context(Some("test_account"))
            .unwrap();

        let driver = Arc::new(MockRouterDriver);
        let router = Arc::new(CapabilityRouter::new(driver.clone(), driver));
        let audit = Arc::new(AuditLogger::new(storage.clone()));
        let action_mgr = Arc::new(ActionManager::new(
            storage.clone(),
            router.clone(),
            account_mgr.clone(),
            audit,
        ));

        let engine = IssueEngine::new(
            router,
            account_mgr,
            action_mgr,
            storage,
            "client_id".to_string(),
        );

        let post_payload = json!({
            "subreddit": "rust",
            "title": "Short",
            "text": "Check this out and buy now affiliate link"
        });

        let diagnosis = engine
            .diagnose_post_rejection(&ctx, "rust", "SUBREDDIT_RATELIMIT_OR_RULE", &post_payload)
            .await
            .unwrap();

        assert_eq!(diagnosis.category, IssueCategory::PostRejected);
        assert_eq!(diagnosis.remediation_code, "CORRECT_POST_PAYLOAD");

        let plan = engine.plan_repair(&diagnosis).unwrap();
        assert!(
            plan.requires_human,
            "Correcting post payload must require human review"
        );
    }

    #[tokio::test]
    async fn test_issue_diagnosis_security_challenge_halts_automation() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let keyring = Arc::new(MemoryKeyring::new());
        let client = reqwest::Client::new();
        let auth = Arc::new(AuthManager::new(keyring, client));
        let account_mgr = Arc::new(AccountManager::new(storage.clone(), auth));

        let _acc = account_mgr.add_account("suspended_acc").unwrap();
        let ctx = account_mgr
            .get_account_context(Some("suspended_acc"))
            .unwrap();

        let driver = Arc::new(MockRouterDriver);
        let router = Arc::new(CapabilityRouter::new(driver.clone(), driver));
        let audit = Arc::new(AuditLogger::new(storage.clone()));
        let action_mgr = Arc::new(ActionManager::new(
            storage.clone(),
            router.clone(),
            account_mgr.clone(),
            audit,
        ));

        let engine = IssueEngine::new(
            router,
            account_mgr,
            action_mgr,
            storage,
            "client_id".to_string(),
        );

        let diagnosis = DiagnosisReport {
            account: "suspended_acc".to_string(),
            category: IssueCategory::SecurityChallenge,
            code: "ACCOUNT_LOCKED_OR_SUSPENDED".to_string(),
            summary: "Account suspended".to_string(),
            evidence: json!({}),
            human_action_mandatory: true,
            remediation_code: "HUMAN_APPEAL_CHECKPOINT".to_string(),
        };

        let plan = engine.plan_repair(&diagnosis).unwrap();
        assert!(plan.requires_human);

        let repair_res = engine.apply_safe_repair(&ctx, &plan).await.unwrap();
        assert_eq!(repair_res.status, "human_action_required");
        assert!(repair_res.details.contains("human interaction"));
    }
}
