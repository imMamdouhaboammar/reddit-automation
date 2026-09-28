use chrono::{Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

use crate::accounts::{AccountContext, AccountManager};
use crate::audit::AuditLogger;
use crate::drivers::{Capability, DriverError};
use crate::reddit::PostRequirements;
use crate::router::CapabilityRouter;
use crate::storage::{PendingActionRecord, Storage, StorageError};

#[derive(Error, Debug)]
pub enum ActionError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Router/Driver error: {0}")]
    Driver(#[from] DriverError),
    #[error("Action not found: {0}")]
    NotFound(String),
    #[error("Action plan expired at {0}")]
    Expired(String),
    #[error("Action requires approval before execution (Risk level: {0:?})")]
    ApprovalRequired(RiskLevel),
    #[error("Payload tampered! Hash mismatch: expected {expected}, computed {computed}")]
    PayloadTampered { expected: String, computed: String },
    #[error("Security sensitive action halted. Mandatory interactive human action required")]
    SecurityCheckpoint(String),
    #[error("Action already executed or rejected: {0}")]
    InvalidState(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskLevel {
    Read,
    Reversible,
    PublicWrite,
    Destructive,
    SecuritySensitive,
}

impl RiskLevel {
    pub fn requires_approval(&self) -> bool {
        matches!(
            self,
            RiskLevel::PublicWrite | RiskLevel::Destructive | RiskLevel::SecuritySensitive
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionStatus {
    Pending,
    Approved,
    Executed,
    Rejected,
    Expired,
}

impl std::fmt::Display for ActionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ActionStatus::Pending => "pending",
            ActionStatus::Approved => "approved",
            ActionStatus::Executed => "executed",
            ActionStatus::Rejected => "rejected",
            ActionStatus::Expired => "expired",
        };
        write!(f, "{}", s)
    }
}

impl From<&str> for ActionStatus {
    fn from(s: &str) -> Self {
        match s {
            "approved" => ActionStatus::Approved,
            "executed" => ActionStatus::Executed,
            "rejected" => ActionStatus::Rejected,
            "expired" => ActionStatus::Expired,
            _ => ActionStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPlan {
    pub id: String,
    pub account: String,
    pub reddit_username: Option<String>,
    pub driver: String,
    pub action: String,
    pub target: String,
    pub payload_preview: Value,
    pub payload_hash: String,
    pub risk_level: RiskLevel,
    pub warnings: Vec<String>,
    pub status: ActionStatus,
    pub created_at: String,
    pub expires_at: String,
}

pub struct ActionManager {
    storage: Arc<Storage>,
    router: Arc<CapabilityRouter>,
    account_manager: Arc<AccountManager>,
    audit_logger: Arc<AuditLogger>,
}

impl ActionManager {
    pub fn new(
        storage: Arc<Storage>,
        router: Arc<CapabilityRouter>,
        account_manager: Arc<AccountManager>,
        audit_logger: Arc<AuditLogger>,
    ) -> Self {
        Self {
            storage,
            router,
            account_manager,
            audit_logger,
        }
    }

    pub fn compute_action_hash(
        account_id: &str,
        action: &str,
        target: &str,
        payload: &Value,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(account_id.as_bytes());
        hasher.update(b":");
        hasher.update(action.as_bytes());
        hasher.update(b":");
        hasher.update(target.as_bytes());
        hasher.update(b":");
        let serialized = serde_json::to_string(payload).unwrap_or_default();
        hasher.update(serialized.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn compute_payload_hash(payload: &Value) -> String {
        let mut hasher = Sha256::new();
        let serialized = serde_json::to_string(payload).unwrap_or_default();
        hasher.update(serialized.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn validate_post_requirements(
        reqs: &PostRequirements,
        title: &str,
        body: &str,
        flair_id: Option<&str>,
    ) -> Vec<String> {
        let mut warnings = Vec::new();

        // 1. Title length
        if let Some(min) = reqs.title_text_min_length {
            if title.len() < min {
                warnings.push(format!(
                    "Title is shorter than required minimum ({} < {} chars)",
                    title.len(),
                    min
                ));
            }
        }
        if let Some(max) = reqs.title_text_max_length {
            if title.len() > max {
                warnings.push(format!(
                    "Title exceeds maximum length ({} > {} chars)",
                    title.len(),
                    max
                ));
            }
        }

        // 2. Body length
        if let Some(min) = reqs.body_text_min_length {
            if body.len() < min {
                warnings.push(format!(
                    "Body is shorter than required minimum ({} < {} chars)",
                    body.len(),
                    min
                ));
            }
        }
        if let Some(max) = reqs.body_text_max_length {
            if body.len() > max {
                warnings.push(format!(
                    "Body exceeds maximum length ({} > {} chars)",
                    body.len(),
                    max
                ));
            }
        }

        // 3. Blacklisted strings
        if let Some(blacklisted) = &reqs.body_blacklisted_strings {
            let lower_body = body.to_lowercase();
            for word in blacklisted {
                if lower_body.contains(&word.to_lowercase()) {
                    warnings.push(format!("Body contains prohibited phrase: '{}'", word));
                }
            }
        }

        // 4. Flair requirement
        if reqs.is_flair_required.unwrap_or(false) && flair_id.is_none() {
            warnings.push("Subreddit requires post flair, but no flair was selected".to_string());
        }

        warnings
    }

    pub fn validate_post_submission(
        subreddit: &str,
        title: &str,
        kind: &str,
        url: Option<&str>,
    ) -> Result<(), ActionError> {
        let clean_sub = subreddit.trim().trim_start_matches("r/");
        if clean_sub.len() < 3 || clean_sub.len() > 21 {
            return Err(ActionError::Driver(DriverError::Internal(format!(
                "Invalid subreddit name '{}': Subreddit names must be 3-21 characters.",
                subreddit
            ))));
        }
        if !clean_sub
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(ActionError::Driver(DriverError::Internal(format!(
                "Invalid subreddit name '{}': Subreddit names must contain only letters, numbers, or underscores.",
                subreddit
            ))));
        }

        let trimmed_title = title.trim();
        if trimmed_title.is_empty() || title.len() > 300 {
            return Err(ActionError::Driver(DriverError::Internal(
                "Invalid title: Submission titles must be between 1 and 300 characters."
                    .to_string(),
            )));
        }

        if kind != "self" && kind != "link" {
            return Err(ActionError::Driver(DriverError::Internal(format!(
                "Invalid post kind '{}': Must be 'self' or 'link'.",
                kind
            ))));
        }

        if kind == "link" {
            let u = url.ok_or_else(|| {
                ActionError::Driver(DriverError::Internal(
                    "Link posts require a 'url' parameter.".to_string(),
                ))
            })?;
            let parsed = url::Url::parse(u).map_err(|e| {
                ActionError::Driver(DriverError::Internal(format!(
                    "Invalid URL for link post: {}",
                    e
                )))
            })?;
            if parsed.scheme() != "http" && parsed.scheme() != "https" {
                return Err(ActionError::Driver(DriverError::Internal(
                    "Link posts require an HTTP or HTTPS URL.".to_string(),
                )));
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare_action(
        &self,
        account_ctx: &AccountContext,
        action: &str,
        target: &str,
        payload: Value,
        risk_level: RiskLevel,
        warnings: Vec<String>,
        ttl_minutes: i64,
    ) -> Result<ActionPlan, ActionError> {
        let id = format!("act_{}", Uuid::new_v4().simple());
        let payload_hash =
            Self::compute_action_hash(&account_ctx.account.id, action, target, &payload);
        let now = Utc::now();
        let expires_at = now + ChronoDuration::minutes(ttl_minutes);

        let plan = ActionPlan {
            id: id.clone(),
            account: account_ctx.account.alias.clone(),
            reddit_username: account_ctx.account.reddit_username.clone(),
            driver: account_ctx.account.default_driver.clone(),
            action: action.to_string(),
            target: target.to_string(),
            payload_preview: payload.clone(),
            payload_hash: payload_hash.clone(),
            risk_level,
            warnings: warnings.clone(),
            status: ActionStatus::Pending,
            created_at: now.to_rfc3339(),
            expires_at: expires_at.to_rfc3339(),
        };

        let record = PendingActionRecord {
            id,
            account_id: account_ctx.account.id.clone(),
            driver: account_ctx.account.default_driver.clone(),
            action_type: action.to_string(),
            target: target.to_string(),
            payload_json: serde_json::to_string(&payload).unwrap_or_default(),
            payload_hash,
            risk_level: serde_json::to_string(&risk_level)
                .unwrap_or_default()
                .replace('"', ""),
            warnings_json: serde_json::to_string(&warnings).unwrap_or_default(),
            status: "pending".to_string(),
            created_at: now.to_rfc3339(),
            expires_at: expires_at.to_rfc3339(),
        };

        self.storage.save_pending_action(&record)?;
        Ok(plan)
    }

    pub fn get_action(&self, id: &str) -> Result<ActionPlan, ActionError> {
        let record = self.storage.get_pending_action(id)?;
        let account = self.storage.get_account_by_id(&record.account_id)?;
        let payload: Value = serde_json::from_str(&record.payload_json).unwrap_or(Value::Null);
        let warnings: Vec<String> = serde_json::from_str(&record.warnings_json).unwrap_or_default();
        let risk_level: RiskLevel = serde_json::from_str(&format!("\"{}\"", record.risk_level))
            .unwrap_or(RiskLevel::PublicWrite);

        Ok(ActionPlan {
            id: record.id,
            account: account.alias,
            reddit_username: account.reddit_username,
            driver: record.driver,
            action: record.action_type,
            target: record.target,
            payload_preview: payload,
            payload_hash: record.payload_hash,
            risk_level,
            warnings,
            status: ActionStatus::from(record.status.as_str()),
            created_at: record.created_at,
            expires_at: record.expires_at,
        })
    }

    pub fn approve_action(&self, id: &str) -> Result<ActionPlan, ActionError> {
        let mut plan = self.get_action(id)?;
        if plan.status != ActionStatus::Pending {
            return Err(ActionError::InvalidState(format!(
                "Cannot approve action in status '{:?}'",
                plan.status
            )));
        }

        let record = self.storage.get_pending_action(id)?;
        let mut updated = record;
        updated.status = "approved".to_string();
        self.storage.save_pending_action(&updated)?;

        plan.status = ActionStatus::Approved;
        Ok(plan)
    }

    pub fn reject_action(&self, id: &str, _reason: &str) -> Result<ActionPlan, ActionError> {
        let mut plan = self.get_action(id)?;
        let record = self.storage.get_pending_action(id)?;
        let mut updated = record;
        updated.status = "rejected".to_string();
        self.storage.save_pending_action(&updated)?;

        plan.status = ActionStatus::Rejected;
        Ok(plan)
    }

    pub async fn execute_action(&self, id: &str) -> Result<Value, ActionError> {
        let record = self.storage.get_pending_action(id)?;
        let plan = self.get_action(id)?;

        // 1. Expiration check
        let expires_at = chrono::DateTime::parse_from_rfc3339(&plan.expires_at)
            .map_err(|_| ActionError::Expired(plan.expires_at.clone()))?;
        if Utc::now() > expires_at {
            return Err(ActionError::Expired(plan.expires_at));
        }

        // 2. Tamper check: recalculate SHA-256 of bound account, action, target, and payload
        let payload: Value = serde_json::from_str(&record.payload_json).unwrap_or(Value::Null);
        let computed_hash = Self::compute_action_hash(
            &record.account_id,
            &record.action_type,
            &record.target,
            &payload,
        );
        if computed_hash != record.payload_hash {
            return Err(ActionError::PayloadTampered {
                expected: record.payload_hash,
                computed: computed_hash,
            });
        }

        // 3. Status and approval check
        if plan.status == ActionStatus::Executed {
            return Err(ActionError::InvalidState(
                "Action has already been executed".to_string(),
            ));
        }
        if plan.status == ActionStatus::Rejected {
            return Err(ActionError::InvalidState(
                "Action has been rejected".to_string(),
            ));
        }
        if plan.risk_level.requires_approval() && plan.status != ActionStatus::Approved {
            return Err(ActionError::ApprovalRequired(plan.risk_level));
        }

        // 4. Security checkpoint check
        if plan.risk_level == RiskLevel::SecuritySensitive {
            return Err(ActionError::SecurityCheckpoint(
                "Security sensitive operation requires explicit local interactive confirmation."
                    .to_string(),
            ));
        }

        // 5. Resolve capability and account context
        let account_ctx = self
            .account_manager
            .get_account_context(Some(&plan.account))
            .map_err(|e| ActionError::Driver(DriverError::Account(e.to_string())))?;

        let capability = match plan.action.as_str() {
            "create_post" => Capability::CreatePost,
            "create_comment" => Capability::CreateComment,
            "edit_post" => Capability::EditPost,
            "delete_post" => Capability::DeletePost,
            "edit_comment" => Capability::EditComment,
            "delete_comment" => Capability::DeleteComment,
            "set_preferences" => Capability::SetPreferences,
            "manage_subscription" => Capability::ManageSubscription,
            "manage_saved" => Capability::ManageSaved,
            "manage_hidden" => Capability::ManageHidden,
            _ => {
                return Err(ActionError::Driver(DriverError::Internal(format!(
                    "Unknown action: {}",
                    plan.action
                ))))
            }
        };

        // 6. Execute through CapabilityRouter
        let result = self
            .router
            .execute(&capability, &account_ctx, payload)
            .await?;

        // 7. Update action status to executed
        let mut updated = record;
        updated.status = "executed".to_string();
        self.storage.save_pending_action(&updated)?;

        // 8. Structured audit log entry
        let reddit_id = result["id"].as_str().or_else(|| result["name"].as_str());
        let canonical_url = result["url"].as_str();

        self.audit_logger.log_event(
            &account_ctx.account.id,
            account_ctx.account.reddit_username.as_deref(),
            &plan.driver,
            &plan.action,
            &plan.target,
            "success",
            reddit_id,
            canonical_url,
            Some(&result),
        )?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{AuthManager, MemoryKeyring};
    use crate::drivers::HealthReport;
    use serde_json::json;

    struct TestDriver;
    #[async_trait::async_trait]
    impl crate::drivers::Driver for TestDriver {
        fn name(&self) -> &'static str {
            "test_driver"
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
            Ok(json!({
                "id": "t3_test123",
                "url": "https://reddit.com/r/rust/comments/test123",
                "echo": in_val
            }))
        }
        async fn health_check(&self, _ctx: &AccountContext) -> Result<HealthReport, DriverError> {
            Ok(HealthReport {
                account: "tester".to_string(),
                reddit_username: None,
                status: "healthy".to_string(),
                issues: vec![],
            })
        }
    }

    #[tokio::test]
    async fn test_action_plan_tamper_proofing_and_approval_gate() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let keyring = Arc::new(MemoryKeyring::new());
        let client = reqwest::Client::new();
        let auth = Arc::new(AuthManager::new(keyring, client));
        let account_mgr = Arc::new(AccountManager::new(storage.clone(), auth));

        let acc = account_mgr.add_account("company").unwrap();
        account_mgr
            .link_identity(&acc.id, "brand_account", &["submit".to_string()])
            .unwrap();
        let ctx = account_mgr.get_account_context(Some("company")).unwrap();

        let driver = Arc::new(TestDriver);
        let router = Arc::new(CapabilityRouter::new(driver.clone(), driver));
        let audit = Arc::new(AuditLogger::new(storage.clone()));
        let action_mgr = ActionManager::new(storage.clone(), router, account_mgr, audit);

        let payload = json!({"subreddit": "rust", "title": "Hello Rustaceans", "kind": "self"});

        // Prepare post (PublicWrite)
        let plan = action_mgr
            .prepare_action(
                &ctx,
                "create_post",
                "r/rust",
                payload.clone(),
                RiskLevel::PublicWrite,
                vec![],
                30,
            )
            .unwrap();

        // 1. Attempting to execute unapproved action MUST fail closed
        let unapproved_res = action_mgr.execute_action(&plan.id).await;
        assert!(matches!(
            unapproved_res,
            Err(ActionError::ApprovalRequired(RiskLevel::PublicWrite))
        ));

        // 2. Approve action
        action_mgr.approve_action(&plan.id).unwrap();

        // 3. Execution succeeds after approval
        let exec_res = action_mgr.execute_action(&plan.id).await.unwrap();
        assert_eq!(exec_res["id"], "t3_test123");
        assert_eq!(
            exec_res["url"],
            "https://reddit.com/r/rust/comments/test123"
        );

        // 4. Verify audit log was recorded in SQLite
        let logs = storage.list_audit_logs(Some(&acc.id), 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].action, "create_post");
        assert_eq!(logs[0].target, "r/rust");
        assert_eq!(logs[0].reddit_id.as_deref(), Some("t3_test123"));
    }

    #[test]
    fn test_validate_post_requirements() {
        let reqs = PostRequirements {
            title_regexes: None,
            body_blacklisted_strings: Some(vec!["buy now".to_string(), "affiliate".to_string()]),
            body_required_strings: None,
            body_restriction: None,
            is_flair_required: Some(true),
            title_text_min_length: Some(10),
            title_text_max_length: Some(100),
            body_text_min_length: Some(20),
            body_text_max_length: None,
            guidelines_text: None,
        };

        // Short title, blacklisted phrase, missing flair
        let warnings = ActionManager::validate_post_requirements(
            &reqs,
            "Short",
            "Check this out and buy now please!",
            None,
        );

        assert!(warnings.iter().any(|w| w.contains("Title is shorter")));
        assert!(warnings.iter().any(|w| w.contains("buy now")));
        assert!(warnings.iter().any(|w| w.contains("requires post flair")));
    }
}
