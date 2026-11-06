use crate::auth::{AuthError, CredentialStore, SecretToken};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Cloud-native Headless Credential Provider for Google Cloud Run and containerized environments.
/// Resolves secrets in order:
/// 1. Mounted Secret Manager files (`/secrets/<service>/<account>` or `GCP_SECRETS_MOUNT_PATH`)
/// 2. Secret Manager environment variables (`REDDIT_SECRET_<NAME>`)
/// 3. In-memory cache fallback
#[derive(Debug, Clone)]
pub struct GcpSecretManagerStore {
    mount_dir: Option<PathBuf>,
    memory_store: Arc<RwLock<HashMap<String, String>>>,
}

impl Default for GcpSecretManagerStore {
    fn default() -> Self {
        Self::new()
    }
}

impl GcpSecretManagerStore {
    pub fn new() -> Self {
        let mount_dir = std::env::var("GCP_SECRETS_MOUNT_PATH")
            .ok()
            .map(PathBuf::from);

        Self {
            mount_dir,
            memory_store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn new_with_memory_fallback() -> Self {
        Self::new()
    }

    pub fn with_mount_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.mount_dir = Some(path.into());
        self
    }

    fn normalize_env_key(service: &str, account: &str) -> String {
        format!("{}_{}", service, account)
            .to_uppercase()
            .replace(['-', ':', '.', '/'], "_")
    }

    fn make_key(service: &str, account: &str) -> String {
        format!("{}:{}", service, account)
    }
}

impl CredentialStore for GcpSecretManagerStore {
    fn get_secret(&self, service: &str, account: &str) -> Result<SecretToken, AuthError> {
        // 1. Check mounted file path if configured (Google Cloud Secret Manager volume mount)
        if let Some(ref base) = self.mount_dir {
            let file_path = base.join(service).join(account);
            if file_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&file_path) {
                    return Ok(SecretToken::new(content.trim().to_string()));
                }
            }
        }

        // 2. Check environment variable injected via Cloud Secret Manager
        let primary_env_key = Self::normalize_env_key(service, account);
        let secret_prefix_key = format!(
            "REDDIT_SECRET_{}",
            account.to_uppercase().replace(['-', ':', '.', '/'], "_")
        );

        if let Ok(val) = std::env::var(&primary_env_key) {
            return Ok(SecretToken::new(val));
        }
        if let Ok(val) = std::env::var(&secret_prefix_key) {
            return Ok(SecretToken::new(val));
        }

        // 3. Check memory store
        let key = Self::make_key(service, account);
        let guard = self
            .memory_store
            .read()
            .map_err(|e| AuthError::Keyring(format!("Lock error: {}", e)))?;

        if let Some(val) = guard.get(&key) {
            return Ok(SecretToken::new(val.clone()));
        }

        Err(AuthError::Keyring(format!(
            "Secret '{}:{}' not found in Secret Manager mount, environment, or memory store",
            service, account
        )))
    }

    fn set_secret(&self, service: &str, account: &str, secret: &str) -> Result<(), AuthError> {
        let key = Self::make_key(service, account);
        let mut guard = self
            .memory_store
            .write()
            .map_err(|e| AuthError::Keyring(format!("Lock error: {}", e)))?;

        guard.insert(key, secret.to_string());
        Ok(())
    }

    fn delete_secret(&self, service: &str, account: &str) -> Result<(), AuthError> {
        let key = Self::make_key(service, account);
        let mut guard = self
            .memory_store
            .write()
            .map_err(|e| AuthError::Keyring(format!("Lock error: {}", e)))?;

        guard.remove(&key);
        Ok(())
    }
}
