use crate::accounts::AccountContext;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

pub mod browser;
pub mod reddit_api;

#[derive(Error, Debug)]
pub enum DriverError {
    #[error("Capability not supported: {0:?}")]
    Unsupported(Capability),
    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },
    #[error("Rate limited: retry after {0} seconds")]
    RateLimited(u64),
    #[error("Authentication required: {0}")]
    Unauthorized(String),
    #[error("Missing OAuth scope: required {0}")]
    MissingScope(String),
    #[error("Account context error: {0}")]
    Account(String),
    #[error("Browser capability broken: {0}")]
    BrowserCapabilityBroken(String),
    #[error("Human checkpoint required: {0}")]
    HumanCheckpointRequired(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    GetIdentity,
    GetPreferences,
    SetPreferences,
    GetSubreddit,
    GetSubredditRules,
    GetPostRequirements,
    Search,
    GetPost,
    GetComments,
    CreatePost,
    EditPost,
    DeletePost,
    CreateComment,
    EditComment,
    DeleteComment,
    GetInbox,
    ManageSubscription,
    ManageSaved,
    ManageHidden,
    InspectAccountHealth,
    BrowserSettings,
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    pub code: String,
    pub capability: String,
    pub message: String,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub account: String,
    pub reddit_username: Option<String>,
    pub status: String, // "healthy", "degraded", "unhealthy"
    pub issues: Vec<HealthIssue>,
}

#[async_trait]
pub trait Driver: Send + Sync {
    fn name(&self) -> &'static str;
    fn supports(&self, capability: &Capability) -> bool;
    async fn execute(
        &self,
        capability: &Capability,
        account_ctx: &AccountContext,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, DriverError>;
    async fn health_check(&self, account_ctx: &AccountContext)
        -> Result<HealthReport, DriverError>;
}
