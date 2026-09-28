use crate::auth::{AuthError, AuthManager};
use crate::storage::{AccountRecord, Storage, StorageError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Auth error: {0}")]
    Auth(#[from] AuthError),
    #[error("Account not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountContext {
    pub account: AccountRecord,
    pub scopes: Vec<String>,
    pub browser_profile_path: Option<String>,
}

pub struct AccountManager {
    storage: Arc<Storage>,
    auth: Arc<AuthManager>,
}

impl AccountManager {
    pub fn new(storage: Arc<Storage>, auth: Arc<AuthManager>) -> Self {
        Self { storage, auth }
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    pub fn auth(&self) -> &AuthManager {
        &self.auth
    }

    pub fn add_account(&self, alias: &str) -> Result<AccountRecord, AccountError> {
        let account = self.storage.add_account(alias)?;
        Ok(account)
    }

    pub fn resolve_account(
        &self,
        alias_or_id: Option<&str>,
    ) -> Result<AccountRecord, AccountError> {
        match alias_or_id {
            Some(name) => {
                // Try alias first
                if let Ok(acc) = self.storage.get_account_by_alias(name) {
                    return Ok(acc);
                }
                // Try ID
                if let Ok(acc) = self.storage.get_account_by_id(name) {
                    return Ok(acc);
                }
                Err(AccountError::NotFound(name.to_string()))
            }
            None => {
                // Get active account
                self.storage.get_active_account().map_err(|_| {
                    AccountError::NotFound("No default active account found. Add an account with `reddit-agent account add <alias>` first.".to_string())
                })
            }
        }
    }

    pub fn get_account_context(
        &self,
        alias_or_id: Option<&str>,
    ) -> Result<AccountContext, AccountError> {
        let account = self.resolve_account(alias_or_id)?;
        let oauth_meta = self.storage.get_oauth_metadata(&account.id)?;
        let scopes = oauth_meta.map(|m| m.scopes).unwrap_or_default();
        let browser_profile = self.storage.get_browser_profile(&account.id)?;
        let browser_profile_path = browser_profile.map(|p| p.profile_path);

        Ok(AccountContext {
            account,
            scopes,
            browser_profile_path,
        })
    }

    pub fn set_active_account(&self, alias: &str) -> Result<(), AccountError> {
        self.storage.set_active_account(alias)?;
        Ok(())
    }

    pub fn list_accounts(&self) -> Result<Vec<AccountRecord>, AccountError> {
        let accounts = self.storage.list_accounts()?;
        Ok(accounts)
    }

    pub fn link_identity(
        &self,
        account_id: &str,
        reddit_username: &str,
        scopes: &[String],
    ) -> Result<(), AccountError> {
        self.storage
            .update_reddit_username(account_id, reddit_username)?;
        self.storage.save_oauth_metadata(
            account_id,
            scopes,
            None,
            AuthManager::keyring_service(),
            &AuthManager::keyring_account(reddit_username),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::MemoryKeyring;

    #[test]
    fn test_account_manager_resolution() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let keyring = Arc::new(MemoryKeyring::new());
        let client = reqwest::Client::new();
        let auth = Arc::new(AuthManager::new(keyring, client));
        let manager = AccountManager::new(storage.clone(), auth);

        // Add two accounts
        manager.add_account("personal").unwrap();
        manager.add_account("company").unwrap();

        // Default active is personal
        let default_ctx = manager.get_account_context(None).unwrap();
        assert_eq!(default_ctx.account.alias, "personal");

        // Explicit resolve company
        let company_ctx = manager.get_account_context(Some("company")).unwrap();
        assert_eq!(company_ctx.account.alias, "company");

        // Switch default to company
        manager.set_active_account("company").unwrap();
        let new_default = manager.get_account_context(None).unwrap();
        assert_eq!(new_default.account.alias, "company");
    }
}
