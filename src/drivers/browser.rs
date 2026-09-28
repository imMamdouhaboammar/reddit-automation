use async_trait::async_trait;
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

use crate::accounts::AccountContext;
use crate::drivers::{Capability, Driver, DriverError, HealthIssue, HealthReport};
use crate::storage::Storage;

pub struct BrowserDriver {
    chrome_path: Option<PathBuf>,
    storage: Option<Arc<Storage>>,
}

impl BrowserDriver {
    pub fn new() -> Self {
        Self {
            chrome_path: Self::find_chrome_executable(),
            storage: None,
        }
    }

    pub fn with_storage(mut self, storage: Arc<Storage>) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn with_chrome_path(mut self, path: PathBuf) -> Self {
        self.chrome_path = Some(path);
        self
    }

    pub fn find_chrome_executable() -> Option<PathBuf> {
        if let Ok(env_path) = std::env::var("CHROME_BIN") {
            let p = PathBuf::from(env_path);
            if p.exists() {
                return Some(p);
            }
        }

        let candidates = [
            // macOS paths
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
            "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
            // Linux paths
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
        ];

        for c in &candidates {
            let p = PathBuf::from(c);
            if p.exists() {
                return Some(p);
            }
        }

        // Try which command via std::process
        for bin in &["google-chrome", "chromium", "brave-browser"] {
            if let Ok(output) = std::process::Command::new("which").arg(bin).output() {
                if output.status.success() {
                    let out_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !out_str.is_empty() {
                        let p = PathBuf::from(out_str);
                        if p.exists() {
                            return Some(p);
                        }
                    }
                }
            }
        }

        None
    }

    pub fn profile_dir_for_account(alias: &str) -> PathBuf {
        Storage::config_dir().join("browser").join(alias)
    }

    pub async fn launch_isolated_browser(
        &self,
        alias: &str,
        headless: bool,
    ) -> Result<Browser, DriverError> {
        let chrome_exe = self.chrome_path.clone().ok_or_else(|| {
            DriverError::BrowserCapabilityBroken(
                "No Chrome/Chromium executable found on system. Set CHROME_BIN environment variable.".to_string(),
            )
        })?;

        let profile_dir = Self::profile_dir_for_account(alias);
        std::fs::create_dir_all(&profile_dir).map_err(|e| {
            DriverError::Internal(format!(
                "Failed to create isolated profile dir {:?}: {}",
                profile_dir, e
            ))
        })?;

        let mut builder = BrowserConfig::builder()
            .chrome_executable(chrome_exe)
            .user_data_dir(profile_dir)
            .window_size(1280, 900);

        if !headless {
            builder = builder.with_head();
        }

        let config = builder.build().map_err(|e| {
            DriverError::BrowserCapabilityBroken(format!("Failed to build browser config: {}", e))
        })?;

        let (browser, mut handler) = Browser::launch(config).await.map_err(|e| {
            DriverError::BrowserCapabilityBroken(format!("Failed to launch Chrome browser: {}", e))
        })?;

        // Background task to pump the CDP event stream
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    debug!("CDP event error: {}", e);
                }
            }
        });

        Ok(browser)
    }

    /// Interactive human login: opens visible Chrome window to Reddit login page,
    /// waits for human to complete password/passkey/2FA, detects success and persists profile.
    pub async fn interactive_login(
        &self,
        account_ctx: &AccountContext,
        timeout_duration: Duration,
    ) -> Result<String, DriverError> {
        let alias = &account_ctx.account.alias;
        info!(
            "Launching visible browser for interactive login for account '{}'",
            alias
        );

        let browser = self.launch_isolated_browser(alias, false).await?;
        let page = browser
            .new_page("https://www.reddit.com/login")
            .await
            .map_err(|e| {
                DriverError::BrowserCapabilityBroken(format!("Failed to open login page: {}", e))
            })?;

        let start = std::time::Instant::now();
        let mut verified_user = None;

        while start.elapsed() < timeout_duration {
            tokio::time::sleep(Duration::from_secs(2)).await;

            let current_url = page.url().await.unwrap_or(None).unwrap_or_default();
            // If user left the login screen and reached reddit.com
            if !current_url.contains("/login")
                && (current_url.contains("reddit.com") || current_url.starts_with("https://"))
            {
                // Try to evaluate user identity from DOM
                let js = r#"
                    (() => {
                        // Check shreddit user header or drawer
                        const userEl = document.querySelector('header faceplate-tracker[noun="user_profile"], [data-testid="user-dropdown"], #USER_DROPDOWN_ID');
                        if (userEl) return userEl.textContent.trim();
                        // Check shreddit user link
                        const link = document.querySelector('header a[href^="/user/"]');
                        if (link) {
                            const m = link.getAttribute('href').match(/\/user\/([^\/]+)/);
                            if (m) return m[1];
                        }
                        // Check URL if profile page
                        const urlMatch = window.location.href.match(/reddit\.com\/user\/([^\/]+)/);
                        if (urlMatch) return urlMatch[1];
                        return "";
                    })()
                "#;

                if let Ok(eval_res) = page.evaluate(js).await {
                    if let Ok(user_str) = eval_res.into_value::<String>() {
                        let clean = user_str.trim().trim_start_matches("u/");
                        if !clean.is_empty() {
                            verified_user = Some(clean.to_string());
                            break;
                        }
                    }
                }

                // If on homepage/feed and cookies exist, mark authenticated
                if current_url == "https://www.reddit.com/" || current_url.contains("reddit.com/r/")
                {
                    verified_user =
                        verified_user.or_else(|| account_ctx.account.reddit_username.clone());
                    if verified_user.is_some() {
                        break;
                    }
                }
            }
        }

        let verified_username = verified_user.ok_or_else(|| {
            DriverError::HumanCheckpointRequired(
                "Interactive login timed out before user authentication was completed".to_string(),
            )
        })?;

        let profile_dir = Self::profile_dir_for_account(alias);
        let profile_str = profile_dir.to_string_lossy().to_string();

        if let Some(storage) = &self.storage {
            let _ = storage.save_browser_profile(&account_ctx.account.id, &profile_str);
            let _ = storage.update_reddit_username(&account_ctx.account.id, &verified_username);
        }

        info!(
            "Browser profile successfully initialized for account '{}' (u/{})",
            alias, verified_username
        );
        Ok(verified_username)
    }

    /// Navigates to a Reddit settings page, enforcing human checkpoints for sensitive settings
    pub async fn open_settings(
        &self,
        account_ctx: &AccountContext,
        setting_page: &str,
    ) -> Result<String, DriverError> {
        let alias = &account_ctx.account.alias;

        // Security check: halt and require interactive human checkpoint for security-sensitive flows
        let sensitive_pages = ["password", "safety", "account", "delete", "2fa"];
        if sensitive_pages.iter().any(|s| setting_page.contains(s)) {
            return Err(DriverError::HumanCheckpointRequired(format!(
                "Navigating to '{}' involves account security credentials or deletion. Human action mandatory.",
                setting_page
            )));
        }

        let target_url = format!("https://www.reddit.com/settings/{}", setting_page);
        let browser = self.launch_isolated_browser(alias, false).await?;
        let _page = browser.new_page(&target_url).await.map_err(|e| {
            DriverError::BrowserCapabilityBroken(format!("Failed to navigate to settings: {}", e))
        })?;

        Ok(target_url)
    }

    /// Verifies session status by checking profile directory existence
    pub fn verify_session_profile(&self, account_ctx: &AccountContext) -> bool {
        let profile_dir = Self::profile_dir_for_account(&account_ctx.account.alias);
        profile_dir.exists()
    }
}

impl Default for BrowserDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Driver for BrowserDriver {
    fn name(&self) -> &'static str {
        "browser"
    }

    fn supports(&self, capability: &Capability) -> bool {
        matches!(capability, Capability::BrowserSettings)
    }

    async fn execute(
        &self,
        capability: &Capability,
        account_ctx: &AccountContext,
        input: Value,
    ) -> Result<Value, DriverError> {
        match capability {
            Capability::BrowserSettings => {
                let section = input["section"].as_str().unwrap_or("feed");
                let url = self.open_settings(account_ctx, section).await?;
                Ok(json!({
                    "action": "open_settings",
                    "section": section,
                    "url": url,
                    "status": "opened_in_browser"
                }))
            }
            other => Err(DriverError::Unsupported(*other)),
        }
    }

    async fn health_check(
        &self,
        account_ctx: &AccountContext,
    ) -> Result<HealthReport, DriverError> {
        let alias = &account_ctx.account.alias;
        let mut issues = Vec::new();
        let mut status = "healthy".to_string();

        if account_ctx.account.browser_enabled && !self.verify_session_profile(account_ctx) {
            status = "degraded".to_string();
            issues.push(HealthIssue {
                code: "BROWSER_SESSION_EXPIRED".to_string(),
                capability: "browser".to_string(),
                message: format!(
                    "Browser profile directory for account '{}' does not exist",
                    alias
                ),
                remediation: format!(
                    "Run `reddit-agent account login {}` to authenticate browser session.",
                    alias
                ),
            });
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
    use crate::storage::AccountRecord;

    #[test]
    fn test_find_chrome_executable() {
        let exe = BrowserDriver::find_chrome_executable();
        // Since we are running on macOS with Google Chrome installed, it should find it
        assert!(
            exe.is_some(),
            "Google Chrome or Chromium should be found on this system"
        );
    }

    #[test]
    fn test_profile_dir_isolation() {
        let p1 = BrowserDriver::profile_dir_for_account("personal");
        let p2 = BrowserDriver::profile_dir_for_account("company");

        assert!(p1.to_string_lossy().contains("personal"));
        assert!(p2.to_string_lossy().contains("company"));
        assert_ne!(
            p1, p2,
            "Accounts must have strictly isolated browser profiles"
        );
    }

    #[tokio::test]
    async fn test_open_settings_security_checkpoint_enforcement() {
        let driver = BrowserDriver::new();
        let ctx = AccountContext {
            account: AccountRecord {
                id: "acc_1".to_string(),
                alias: "personal".to_string(),
                reddit_username: Some("tester".to_string()),
                is_active: true,
                created_at: "".to_string(),
                last_verified_at: None,
                api_enabled: true,
                browser_enabled: true,
                default_driver: "browser".to_string(),
            },
            scopes: vec![],
            browser_profile_path: None,
        };

        // Navigating to password setting MUST halt with HumanCheckpointRequired
        let res = driver.open_settings(&ctx, "password").await;
        assert!(matches!(res, Err(DriverError::HumanCheckpointRequired(_))));

        let res2 = driver.open_settings(&ctx, "2fa").await;
        assert!(matches!(res2, Err(DriverError::HumanCheckpointRequired(_))));
    }
}
