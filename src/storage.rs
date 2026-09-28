use chrono::Utc;
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    #[error("Action not found: {0}")]
    ActionNotFound(String),
    #[error("Database lock error")]
    LockError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRecord {
    pub id: String,
    pub alias: String,
    pub reddit_username: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub last_verified_at: Option<String>,
    pub api_enabled: bool,
    pub browser_enabled: bool,
    pub default_driver: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthMetadataRecord {
    pub account_id: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<String>,
    pub keychain_service: String,
    pub keychain_account: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserProfileRecord {
    pub account_id: String,
    pub profile_path: String,
    pub last_verified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingActionRecord {
    pub id: String,
    pub account_id: String,
    pub driver: String,
    pub action_type: String,
    pub target: String,
    pub payload_json: String,
    pub payload_hash: String,
    pub risk_level: String,
    pub warnings_json: String,
    pub status: String,
    pub created_at: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogRecord {
    pub id: String,
    pub timestamp: String,
    pub account_id: String,
    pub reddit_username: Option<String>,
    pub driver: String,
    pub action: String,
    pub target: String,
    pub status: String,
    pub reddit_id: Option<String>,
    pub canonical_url: Option<String>,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitRecord {
    pub account_id: String,
    pub remaining: f64,
    pub reset_seconds: u64,
    pub updated_at: String,
}

pub struct Storage {
    conn: Mutex<Connection>,
}

impl Storage {
    pub fn new_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        let storage = Self {
            conn: Mutex::new(conn),
        };
        storage.init_schema()?;
        Ok(storage)
    }

    pub fn new_at_path<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let storage = Self {
            conn: Mutex::new(conn),
        };
        storage.init_schema()?;
        Ok(storage)
    }

    pub fn config_dir() -> PathBuf {
        if let Ok(agent_home) = std::env::var("REDDIT_AGENT_HOME") {
            return PathBuf::from(agent_home);
        }
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            return PathBuf::from(xdg).join("reddit-agent");
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config").join("reddit-agent")
    }

    pub fn default_path() -> PathBuf {
        Self::config_dir().join("state.db")
    }

    pub fn open_default() -> Result<Self, StorageError> {
        Self::new_at_path(Self::default_path())
    }

    fn init_schema(&self) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                alias TEXT UNIQUE NOT NULL,
                reddit_username TEXT,
                is_active INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                last_verified_at TEXT,
                api_enabled INTEGER NOT NULL DEFAULT 1,
                browser_enabled INTEGER NOT NULL DEFAULT 0,
                default_driver TEXT NOT NULL DEFAULT 'api'
            );

            CREATE TABLE IF NOT EXISTS oauth_metadata (
                account_id TEXT PRIMARY KEY,
                scopes TEXT NOT NULL,
                expires_at TEXT,
                keychain_service TEXT NOT NULL,
                keychain_account TEXT NOT NULL,
                FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS browser_profiles (
                account_id TEXT PRIMARY KEY,
                profile_path TEXT NOT NULL,
                last_verified_at TEXT,
                FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS pending_actions (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                driver TEXT NOT NULL,
                action_type TEXT NOT NULL,
                target TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                payload_hash TEXT NOT NULL,
                risk_level TEXT NOT NULL,
                warnings_json TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                account_id TEXT NOT NULL,
                reddit_username TEXT,
                driver TEXT NOT NULL,
                action TEXT NOT NULL,
                target TEXT NOT NULL,
                status TEXT NOT NULL,
                reddit_id TEXT,
                canonical_url TEXT,
                metadata_json TEXT,
                FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS rate_limits (
                account_id TEXT PRIMARY KEY,
                remaining REAL NOT NULL,
                reset_seconds INTEGER NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
            );
            "#,
        )?;
        Ok(())
    }

    // Account operations
    pub fn add_account(&self, alias: &str) -> Result<AccountRecord, StorageError> {
        let id = format!("acc_{}", Uuid::new_v4().simple());
        let now = Utc::now().to_rfc3339();

        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))?;
        let is_active = count == 0;

        conn.execute(
            r#"
            INSERT INTO accounts (id, alias, is_active, created_at, default_driver)
            VALUES (?1, ?2, ?3, ?4, 'api')
            "#,
            params![id, alias, is_active as i32, now],
        )?;

        drop(conn);
        self.get_account_by_alias(alias)
    }

    pub fn get_account_by_alias(&self, alias: &str) -> Result<AccountRecord, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, alias, reddit_username, is_active, created_at, last_verified_at,
                   api_enabled, browser_enabled, default_driver
            FROM accounts
            WHERE alias = ?1
            "#,
        )?;

        let account = stmt
            .query_row(params![alias], |row| {
                Ok(AccountRecord {
                    id: row.get(0)?,
                    alias: row.get(1)?,
                    reddit_username: row.get(2)?,
                    is_active: row.get::<_, i32>(3)? == 1,
                    created_at: row.get(4)?,
                    last_verified_at: row.get(5)?,
                    api_enabled: row.get::<_, i32>(6)? == 1,
                    browser_enabled: row.get::<_, i32>(7)? == 1,
                    default_driver: row.get(8)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    StorageError::AccountNotFound(alias.to_string())
                }
                other => StorageError::Sqlite(other),
            })?;

        Ok(account)
    }

    pub fn get_account_by_id(&self, id: &str) -> Result<AccountRecord, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, alias, reddit_username, is_active, created_at, last_verified_at,
                   api_enabled, browser_enabled, default_driver
            FROM accounts
            WHERE id = ?1
            "#,
        )?;

        let account = stmt
            .query_row(params![id], |row| {
                Ok(AccountRecord {
                    id: row.get(0)?,
                    alias: row.get(1)?,
                    reddit_username: row.get(2)?,
                    is_active: row.get::<_, i32>(3)? == 1,
                    created_at: row.get(4)?,
                    last_verified_at: row.get(5)?,
                    api_enabled: row.get::<_, i32>(6)? == 1,
                    browser_enabled: row.get::<_, i32>(7)? == 1,
                    default_driver: row.get(8)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    StorageError::AccountNotFound(id.to_string())
                }
                other => StorageError::Sqlite(other),
            })?;

        Ok(account)
    }

    pub fn get_active_account(&self) -> Result<AccountRecord, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, alias, reddit_username, is_active, created_at, last_verified_at,
                   api_enabled, browser_enabled, default_driver
            FROM accounts
            WHERE is_active = 1
            LIMIT 1
            "#,
        )?;

        let account = stmt
            .query_row([], |row| {
                Ok(AccountRecord {
                    id: row.get(0)?,
                    alias: row.get(1)?,
                    reddit_username: row.get(2)?,
                    is_active: true,
                    created_at: row.get(4)?,
                    last_verified_at: row.get(5)?,
                    api_enabled: row.get::<_, i32>(6)? == 1,
                    browser_enabled: row.get::<_, i32>(7)? == 1,
                    default_driver: row.get(8)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    StorageError::AccountNotFound("No active account".to_string())
                }
                other => StorageError::Sqlite(other),
            })?;

        Ok(account)
    }

    pub fn set_active_account(&self, alias: &str) -> Result<(), StorageError> {
        let account = self.get_account_by_alias(alias)?;
        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        conn.execute("UPDATE accounts SET is_active = 0", [])?;
        conn.execute(
            "UPDATE accounts SET is_active = 1 WHERE id = ?1",
            params![account.id],
        )?;
        Ok(())
    }

    pub fn list_accounts(&self) -> Result<Vec<AccountRecord>, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, alias, reddit_username, is_active, created_at, last_verified_at,
                   api_enabled, browser_enabled, default_driver
            FROM accounts
            ORDER BY created_at ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(AccountRecord {
                id: row.get(0)?,
                alias: row.get(1)?,
                reddit_username: row.get(2)?,
                is_active: row.get::<_, i32>(3)? == 1,
                created_at: row.get(4)?,
                last_verified_at: row.get(5)?,
                api_enabled: row.get::<_, i32>(6)? == 1,
                browser_enabled: row.get::<_, i32>(7)? == 1,
                default_driver: row.get(8)?,
            })
        })?;

        let mut accounts = Vec::new();
        for acc in rows {
            accounts.push(acc?);
        }
        Ok(accounts)
    }

    pub fn update_reddit_username(
        &self,
        account_id: &str,
        username: &str,
    ) -> Result<(), StorageError> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        conn.execute(
            r#"
            UPDATE accounts
            SET reddit_username = ?1, last_verified_at = ?2
            WHERE id = ?3
            "#,
            params![username, now, account_id],
        )?;
        Ok(())
    }

    // OAuth metadata operations
    pub fn save_oauth_metadata(
        &self,
        account_id: &str,
        scopes: &[String],
        expires_at: Option<&str>,
        keychain_service: &str,
        keychain_account: &str,
    ) -> Result<(), StorageError> {
        let scopes_json = serde_json::to_string(scopes)?;
        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        conn.execute(
            r#"
            INSERT INTO oauth_metadata (account_id, scopes, expires_at, keychain_service, keychain_account)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(account_id) DO UPDATE SET
                scopes = excluded.scopes,
                expires_at = excluded.expires_at,
                keychain_service = excluded.keychain_service,
                keychain_account = excluded.keychain_account
            "#,
            params![account_id, scopes_json, expires_at, keychain_service, keychain_account],
        )?;
        Ok(())
    }

    pub fn get_oauth_metadata(
        &self,
        account_id: &str,
    ) -> Result<Option<OAuthMetadataRecord>, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        let mut stmt = conn.prepare(
            r#"
            SELECT account_id, scopes, expires_at, keychain_service, keychain_account
            FROM oauth_metadata
            WHERE account_id = ?1
            "#,
        )?;

        let res = stmt.query_row(params![account_id], |row| {
            let scopes_str: String = row.get(1)?;
            let scopes: Vec<String> = serde_json::from_str(&scopes_str).unwrap_or_default();
            Ok(OAuthMetadataRecord {
                account_id: row.get(0)?,
                scopes,
                expires_at: row.get(2)?,
                keychain_service: row.get(3)?,
                keychain_account: row.get(4)?,
            })
        });

        match res {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(StorageError::Sqlite(e)),
        }
    }

    // Browser profile operations
    pub fn save_browser_profile(
        &self,
        account_id: &str,
        profile_path: &str,
    ) -> Result<(), StorageError> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        conn.execute(
            r#"
            INSERT INTO browser_profiles (account_id, profile_path, last_verified_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(account_id) DO UPDATE SET
                profile_path = excluded.profile_path,
                last_verified_at = excluded.last_verified_at
            "#,
            params![account_id, profile_path, now],
        )?;
        conn.execute(
            "UPDATE accounts SET browser_enabled = 1 WHERE id = ?1",
            params![account_id],
        )?;
        Ok(())
    }

    pub fn get_browser_profile(
        &self,
        account_id: &str,
    ) -> Result<Option<BrowserProfileRecord>, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        let mut stmt = conn.prepare(
            r#"
            SELECT account_id, profile_path, last_verified_at
            FROM browser_profiles
            WHERE account_id = ?1
            "#,
        )?;

        let res = stmt.query_row(params![account_id], |row| {
            Ok(BrowserProfileRecord {
                account_id: row.get(0)?,
                profile_path: row.get(1)?,
                last_verified_at: row.get(2)?,
            })
        });

        match res {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(StorageError::Sqlite(e)),
        }
    }

    // Pending action operations
    pub fn save_pending_action(&self, record: &PendingActionRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        conn.execute(
            r#"
            INSERT INTO pending_actions (
                id, account_id, driver, action_type, target, payload_json,
                payload_hash, risk_level, warnings_json, status, created_at, expires_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(id) DO UPDATE SET
                account_id = excluded.account_id,
                driver = excluded.driver,
                action_type = excluded.action_type,
                target = excluded.target,
                status = excluded.status,
                payload_json = excluded.payload_json,
                payload_hash = excluded.payload_hash,
                warnings_json = excluded.warnings_json
            "#,
            params![
                record.id,
                record.account_id,
                record.driver,
                record.action_type,
                record.target,
                record.payload_json,
                record.payload_hash,
                record.risk_level,
                record.warnings_json,
                record.status,
                record.created_at,
                record.expires_at
            ],
        )?;
        Ok(())
    }

    pub fn get_pending_action(&self, id: &str) -> Result<PendingActionRecord, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, account_id, driver, action_type, target, payload_json,
                   payload_hash, risk_level, warnings_json, status, created_at, expires_at
            FROM pending_actions
            WHERE id = ?1
            "#,
        )?;

        stmt.query_row(params![id], |row| {
            Ok(PendingActionRecord {
                id: row.get(0)?,
                account_id: row.get(1)?,
                driver: row.get(2)?,
                action_type: row.get(3)?,
                target: row.get(4)?,
                payload_json: row.get(5)?,
                payload_hash: row.get(6)?,
                risk_level: row.get(7)?,
                warnings_json: row.get(8)?,
                status: row.get(9)?,
                created_at: row.get(10)?,
                expires_at: row.get(11)?,
            })
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => StorageError::ActionNotFound(id.to_string()),
            other => StorageError::Sqlite(other),
        })
    }

    pub fn list_pending_actions(
        &self,
        account_id: Option<&str>,
    ) -> Result<Vec<PendingActionRecord>, StorageError> {
        let query = match account_id {
            Some(_) => "SELECT id, account_id, driver, action_type, target, payload_json, payload_hash, risk_level, warnings_json, status, created_at, expires_at FROM pending_actions WHERE account_id = ?1 ORDER BY created_at DESC",
            None => "SELECT id, account_id, driver, action_type, target, payload_json, payload_hash, risk_level, warnings_json, status, created_at, expires_at FROM pending_actions ORDER BY created_at DESC",
        };

        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        let mut stmt = conn.prepare(query)?;
        let rows = if let Some(acc_id) = account_id {
            stmt.query_map(params![acc_id], Self::map_pending_action_row)?
        } else {
            stmt.query_map([], Self::map_pending_action_row)?
        };

        let mut actions = Vec::new();
        for act in rows {
            actions.push(act?);
        }
        Ok(actions)
    }

    fn map_pending_action_row(row: &rusqlite::Row) -> SqlResult<PendingActionRecord> {
        Ok(PendingActionRecord {
            id: row.get(0)?,
            account_id: row.get(1)?,
            driver: row.get(2)?,
            action_type: row.get(3)?,
            target: row.get(4)?,
            payload_json: row.get(5)?,
            payload_hash: row.get(6)?,
            risk_level: row.get(7)?,
            warnings_json: row.get(8)?,
            status: row.get(9)?,
            created_at: row.get(10)?,
            expires_at: row.get(11)?,
        })
    }

    // Audit log operations
    pub fn append_audit_log(&self, record: &AuditLogRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        conn.execute(
            r#"
            INSERT INTO audit_log (
                id, timestamp, account_id, reddit_username, driver,
                action, target, status, reddit_id, canonical_url, metadata_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                record.id,
                record.timestamp,
                record.account_id,
                record.reddit_username,
                record.driver,
                record.action,
                record.target,
                record.status,
                record.reddit_id,
                record.canonical_url,
                record.metadata_json
            ],
        )?;
        Ok(())
    }

    pub fn list_audit_logs(
        &self,
        account_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<AuditLogRecord>, StorageError> {
        let query = match account_id {
            Some(_) => "SELECT id, timestamp, account_id, reddit_username, driver, action, target, status, reddit_id, canonical_url, metadata_json FROM audit_log WHERE account_id = ?1 ORDER BY timestamp DESC LIMIT ?2",
            None => "SELECT id, timestamp, account_id, reddit_username, driver, action, target, status, reddit_id, canonical_url, metadata_json FROM audit_log ORDER BY timestamp DESC LIMIT ?1",
        };

        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        let mut stmt = conn.prepare(query)?;
        let rows = if let Some(acc_id) = account_id {
            stmt.query_map(params![acc_id, limit as i64], Self::map_audit_log_row)?
        } else {
            stmt.query_map(params![limit as i64], Self::map_audit_log_row)?
        };

        let mut logs = Vec::new();
        for item in rows {
            logs.push(item?);
        }
        Ok(logs)
    }

    fn map_audit_log_row(row: &rusqlite::Row) -> SqlResult<AuditLogRecord> {
        Ok(AuditLogRecord {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            account_id: row.get(2)?,
            reddit_username: row.get(3)?,
            driver: row.get(4)?,
            action: row.get(5)?,
            target: row.get(6)?,
            status: row.get(7)?,
            reddit_id: row.get(8)?,
            canonical_url: row.get(9)?,
            metadata_json: row.get(10)?,
        })
    }

    // Rate limits operations
    pub fn update_rate_limit(
        &self,
        account_id: &str,
        remaining: f64,
        reset_seconds: u64,
    ) -> Result<(), StorageError> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        conn.execute(
            r#"
            INSERT INTO rate_limits (account_id, remaining, reset_seconds, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(account_id) DO UPDATE SET
                remaining = excluded.remaining,
                reset_seconds = excluded.reset_seconds,
                updated_at = excluded.updated_at
            "#,
            params![account_id, remaining, reset_seconds as i64, now],
        )?;
        Ok(())
    }

    pub fn get_rate_limit(
        &self,
        account_id: &str,
    ) -> Result<Option<RateLimitRecord>, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::LockError)?;
        let mut stmt = conn.prepare(
            r#"
            SELECT account_id, remaining, reset_seconds, updated_at
            FROM rate_limits
            WHERE account_id = ?1
            "#,
        )?;

        let res = stmt.query_row(params![account_id], |row| {
            Ok(RateLimitRecord {
                account_id: row.get(0)?,
                remaining: row.get(1)?,
                reset_seconds: row.get::<_, i64>(2)? as u64,
                updated_at: row.get(3)?,
            })
        });

        match res {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(StorageError::Sqlite(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_account_lifecycle() {
        let storage = Storage::new_in_memory().expect("in-memory db");

        let acc1 = storage.add_account("personal").expect("add personal");
        assert_eq!(acc1.alias, "personal");
        assert!(acc1.is_active);

        let acc2 = storage.add_account("company").expect("add company");
        assert_eq!(acc2.alias, "company");
        assert!(!acc2.is_active);

        let active = storage.get_active_account().expect("active");
        assert_eq!(active.alias, "personal");

        storage.set_active_account("company").expect("set active");
        let active_new = storage.get_active_account().expect("active after switch");
        assert_eq!(active_new.alias, "company");

        let list = storage.list_accounts().expect("list");
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_storage_pending_action_tamper_proofing() {
        let storage = Storage::new_in_memory().expect("in-memory db");
        let acc = storage.add_account("testing").expect("add");

        let action = PendingActionRecord {
            id: "act_100".to_string(),
            account_id: acc.id.clone(),
            driver: "api".to_string(),
            action_type: "create_post".to_string(),
            target: "r/rust".to_string(),
            payload_json: r#"{"title":"Hello","kind":"self"}"#.to_string(),
            payload_hash: "hash123".to_string(),
            risk_level: "PUBLIC_WRITE".to_string(),
            warnings_json: "[]".to_string(),
            status: "pending".to_string(),
            created_at: Utc::now().to_rfc3339(),
            expires_at: Utc::now().to_rfc3339(),
        };

        storage.save_pending_action(&action).expect("save action");
        let retrieved = storage.get_pending_action("act_100").expect("get");
        assert_eq!(retrieved.payload_hash, "hash123");
        assert_eq!(retrieved.status, "pending");
    }
}
