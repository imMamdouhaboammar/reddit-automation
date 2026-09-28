use crate::accounts::AccountContext;
use crate::drivers::{Capability, Driver, DriverError, HealthReport};
use serde_json::Value;
use std::sync::Arc;

pub struct CapabilityRouter {
    api_driver: Arc<dyn Driver>,
    browser_driver: Arc<dyn Driver>,
}

impl CapabilityRouter {
    pub fn new(api_driver: Arc<dyn Driver>, browser_driver: Arc<dyn Driver>) -> Self {
        Self {
            api_driver,
            browser_driver,
        }
    }

    pub fn resolve_driver(&self, capability: &Capability) -> Result<&Arc<dyn Driver>, DriverError> {
        // API first
        if self.api_driver.supports(capability) {
            Ok(&self.api_driver)
        } else if self.browser_driver.supports(capability) {
            Ok(&self.browser_driver)
        } else {
            Err(DriverError::Unsupported(*capability))
        }
    }

    pub async fn execute(
        &self,
        capability: &Capability,
        account_ctx: &AccountContext,
        input: Value,
    ) -> Result<Value, DriverError> {
        let driver = self.resolve_driver(capability)?;
        driver.execute(capability, account_ctx, input).await
    }

    pub async fn health_check(
        &self,
        account_ctx: &AccountContext,
    ) -> Result<HealthReport, DriverError> {
        // Core health check is performed primarily through API driver
        self.api_driver.health_check(account_ctx).await
    }

    pub fn preview_action(
        account_ctx: &AccountContext,
        action: &str,
        target: &str,
        payload_summary: &str,
    ) -> String {
        let username = account_ctx
            .account
            .reddit_username
            .as_deref()
            .unwrap_or("unverified");
        format!(
            "Account: {}\nReddit user: u/{}\nAction: {}\nTarget: {}\nDetails: {}",
            account_ctx.account.alias, username, action, target, payload_summary
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::AccountRecord;
    use async_trait::async_trait;
    use serde_json::json;

    struct MockApiDriver;
    #[async_trait]
    impl Driver for MockApiDriver {
        fn name(&self) -> &'static str {
            "mock_api"
        }
        fn supports(&self, cap: &Capability) -> bool {
            !matches!(cap, Capability::BrowserSettings)
        }
        async fn execute(
            &self,
            _cap: &Capability,
            _ctx: &AccountContext,
            _in: Value,
        ) -> Result<Value, DriverError> {
            Ok(json!({"source": "api"}))
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

    struct MockBrowserDriver;
    #[async_trait]
    impl Driver for MockBrowserDriver {
        fn name(&self) -> &'static str {
            "mock_browser"
        }
        fn supports(&self, cap: &Capability) -> bool {
            matches!(cap, Capability::BrowserSettings)
        }
        async fn execute(
            &self,
            _cap: &Capability,
            _ctx: &AccountContext,
            _in: Value,
        ) -> Result<Value, DriverError> {
            Ok(json!({"source": "browser"}))
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
    async fn test_capability_routing_api_first() {
        let router = CapabilityRouter::new(Arc::new(MockApiDriver), Arc::new(MockBrowserDriver));

        let ctx = AccountContext {
            account: AccountRecord {
                id: "acc_1".to_string(),
                alias: "company".to_string(),
                reddit_username: Some("company_bot".to_string()),
                is_active: true,
                created_at: "".to_string(),
                last_verified_at: None,
                api_enabled: true,
                browser_enabled: true,
                default_driver: "api".to_string(),
            },
            scopes: vec![],
            browser_profile_path: None,
        };

        // GetIdentity should route to API
        let api_driver = router.resolve_driver(&Capability::GetIdentity).unwrap();
        assert_eq!(api_driver.name(), "mock_api");

        // BrowserSettings should route to Browser
        let browser_driver = router.resolve_driver(&Capability::BrowserSettings).unwrap();
        assert_eq!(browser_driver.name(), "mock_browser");

        // Preview verification
        let preview =
            CapabilityRouter::preview_action(&ctx, "Create post", "r/rust", "Title: Hello Rust");
        assert!(preview.contains("Account: company"));
        assert!(preview.contains("Reddit user: u/company_bot"));
        assert!(preview.contains("Action: Create post"));
        assert!(preview.contains("Target: r/rust"));
    }
}
