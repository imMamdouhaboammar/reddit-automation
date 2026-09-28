use crate::storage::{AuditLogRecord, Storage, StorageError};
use chrono::Utc;
use serde_json::Value;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub struct AuditLogger {
    storage: Arc<Storage>,
}

impl AuditLogger {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn log_event(
        &self,
        account_id: &str,
        reddit_username: Option<&str>,
        driver: &str,
        action: &str,
        target: &str,
        status: &str,
        reddit_id: Option<&str>,
        canonical_url: Option<&str>,
        metadata: Option<&Value>,
    ) -> Result<(), StorageError> {
        let id = format!("aud_{}", Uuid::new_v4().simple());
        let timestamp = Utc::now().to_rfc3339();

        // Sanitize metadata to guarantee zero token or auth header persistence
        let sanitized_metadata = metadata.map(|m| {
            let mut val = m.clone();
            if let Some(obj) = val.as_object_mut() {
                obj.remove("access_token");
                obj.remove("refresh_token");
                obj.remove("authorization");
                obj.remove("cookie");
            }
            serde_json::to_string(&val).unwrap_or_default()
        });

        let record = AuditLogRecord {
            id,
            timestamp: timestamp.clone(),
            account_id: account_id.to_string(),
            reddit_username: reddit_username.map(String::from),
            driver: driver.to_string(),
            action: action.to_string(),
            target: target.to_string(),
            status: status.to_string(),
            reddit_id: reddit_id.map(String::from),
            canonical_url: canonical_url.map(String::from),
            metadata_json: sanitized_metadata,
        };

        self.storage.append_audit_log(&record)?;

        info!(
            timestamp = %timestamp,
            account_id = %account_id,
            reddit_username = ?reddit_username,
            driver = %driver,
            action = %action,
            target = %target,
            status = %status,
            reddit_id = ?reddit_id,
            canonical_url = ?canonical_url,
            "Reddit Operator side-effect audit recorded"
        );

        Ok(())
    }

    pub fn query_logs(
        &self,
        account_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<AuditLogRecord>, StorageError> {
        self.storage.list_audit_logs(account_id, limit)
    }
}
