use async_trait::async_trait;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::network::CookieParam;
use chromiumoxide::page::Page;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::accounts::AccountContext;
use crate::auth::CredentialStore;
use crate::drivers::{Capability, Driver, DriverError, HealthIssue, HealthReport};
use crate::storage::Storage;

/// Obscura anti-detection stealth script (inspired by h4ckf0r0day/obscura).
/// Overrides navigator.webdriver, mocks window.chrome, injects realistic plugins/languages,
/// and randomizes Canvas/WebGL hardware renderer fingerprints to prevent bot detection.
pub const OBSCURA_STEALTH_JS: &str = r#"
(() => {
    // 1. Mask navigator.webdriver
    Object.defineProperty(Object.getPrototypeOf(navigator), 'webdriver', {
        get: () => undefined,
        configurable: true
    });
    try {
        delete navigator.__proto__.webdriver;
    } catch (_) {}

    // 2. Mock window.chrome object
    if (!window.chrome) {
        window.chrome = {};
    }
    window.chrome.app = {
        isInstalled: false,
        InstallState: { DISABLED: 'disabled', INSTALLED: 'installed', NOT_INSTALLED: 'not_installed' },
        RunningState: { CANNOT_RUN: 'cannot_run', READY_TO_RUN: 'ready_to_run', RUNNING: 'running' }
    };
    window.chrome.runtime = {
        OnInstalledReason: { CHROME_UPDATE: 'chrome_update', INSTALL: 'install', SHARED_MODULE_UPDATE: 'shared_module_update', UPDATE: 'update' },
        OnRestartRequiredReason: { APP_UPDATE: 'app_update', OS_UPDATE: 'os_update', PERIODIC: 'periodic' },
        PlatformArch: { ARM: 'arm', ARM64: 'arm64', MIPS: 'mips', MIPS64: 'mips64', X86_32: 'x86-32', X86_64: 'x86-64' },
        PlatformNaclArch: { ARM: 'arm', MIPS: 'mips', MIPS64: 'mips64', X86_32: 'x86-32', X86_64: 'x86-64' },
        PlatformOs: { ANDROID: 'android', CROS: 'cros', LINUX: 'linux', MAC: 'mac', OPENBSD: 'openbsd', WIN: 'win' },
        RequestUpdateCheckStatus: { NO_UPDATE: 'no_update', THROTTLED: 'throttled', UPDATE_AVAILABLE: 'update_available' }
    };
    window.chrome.csi = function() {};
    window.chrome.loadTimes = function() {};

    // 3. Realistic navigator.plugins & languages
    Object.defineProperty(navigator, 'languages', {
        get: () => ['en-US', 'en'],
        configurable: true
    });
    Object.defineProperty(navigator, 'plugins', {
        get: () => [
            { name: 'Chrome PDF Plugin', filename: 'internal-pdf-viewer', description: 'Portable Document Format' },
            { name: 'Chrome PDF Viewer', filename: 'mhjfbmdgcfjbbpaeojofohoefgiehjai', description: '' },
            { name: 'Native Client', filename: 'internal-nacl-plugin', description: '' }
        ],
        configurable: true
    });

    // 4. Subtle WebGL Unmasked Vendor & Renderer spoofing (Apple Metal / M-Series)
    if (typeof WebGLRenderingContext !== 'undefined') {
        const origGetParameter = WebGLRenderingContext.prototype.getParameter;
        WebGLRenderingContext.prototype.getParameter = function(parameter) {
            if (parameter === 37445) return 'Apple Inc.'; // UNMASKED_VENDOR_WEBGL
            if (parameter === 37446) return 'Apple M2 Pro (Metal)'; // UNMASKED_RENDERER_WEBGL
            return origGetParameter.apply(this, arguments);
        };
    }
    if (typeof WebGL2RenderingContext !== 'undefined') {
        const origGetParameter2 = WebGL2RenderingContext.prototype.getParameter;
        WebGL2RenderingContext.prototype.getParameter = function(parameter) {
            if (parameter === 37445) return 'Apple Inc.';
            if (parameter === 37446) return 'Apple M2 Pro (Metal)';
            return origGetParameter2.apply(this, arguments);
        };
    }
})();
"#;

/// Semantic snapshot JS extractor: scans DOM, marks interactive nodes with @e1, @e2,
/// and builds a high-signal text accessibility tree (inspired by vercel-labs/agent-browser).
pub const SEMANTIC_SNAPSHOT_JS: &str = r#"
(() => {
    const interactiveSelectors = [
        'button',
        'a[href]',
        'input',
        'textarea',
        'select',
        '[role="button"]',
        '[role="link"]',
        '[role="tab"]',
        '[role="menuitem"]',
        'shreddit-post',
        'article',
        'h1', 'h2', 'h3'
    ];

    const elements = Array.from(document.querySelectorAll(interactiveSelectors.join(',')))
        .filter(el => {
            const rect = el.getBoundingClientRect();
            if (rect.width === 0 || rect.height === 0) return false;
            const style = window.getComputedStyle(el);
            if (style.display === 'none' || style.visibility === 'hidden' || style.opacity === '0') return false;
            return true;
        })
        .slice(0, 100);

    const results = [];
    const treeLines = [];
    treeLines.push(`[Page: ${document.title}] (URL: ${window.location.href})`);

    elements.forEach((el, index) => {
        const refId = `@e${index + 1}`;
        el.setAttribute('data-agent-ref', refId);

        let tag = el.tagName.toLowerCase();
        let role = el.getAttribute('role') || tag;
        let label = (el.innerText || el.getAttribute('placeholder') || el.getAttribute('aria-label') || el.getAttribute('title') || el.value || '').trim().replace(/\s+/g, ' ');
        if (label.length > 80) label = label.substring(0, 77) + '...';

        let href = el.getAttribute('href') || null;

        results.push({
            ref_id: refId,
            tag: role,
            label: label,
            href: href
        });

        let line = `  - ${refId} [${role.toUpperCase()}] "${label}"`;
        if (href) line += ` -> ${href}`;
        treeLines.push(line);
    });

    return JSON.stringify({
        title: document.title,
        url: window.location.href,
        element_count: results.length,
        tree_text: treeLines.join('\n'),
        elements: results
    });
})()
"#;

/// Individual session cookie item stored in the OS Keychain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionCookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub secure: Option<bool>,
    pub http_only: Option<bool>,
    pub same_site: Option<String>,
    pub expires: Option<i64>,
}

/// Encrypted container for account session cookies
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SessionCookieJar {
    pub alias: String,
    pub reddit_username: Option<String>,
    pub captured_at: String,
    pub cookie_count: usize,
    pub cookies: Vec<SessionCookie>,
}

/// Representation of an interactive DOM element tagged by the semantic extractor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotElement {
    pub ref_id: String,
    pub tag: String,
    pub label: String,
    pub href: Option<String>,
}

/// Token-efficient semantic DOM tree for AI agents (inspired by vercel-labs/agent-browser)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSnapshot {
    pub title: String,
    pub url: String,
    pub element_count: usize,
    pub tree_text: String,
    pub elements: Vec<SnapshotElement>,
}

/// Outcome of an interactive account login / takeover operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeoverResult {
    pub account: String,
    pub verified_username: String,
    pub cookies_captured: usize,
    pub keychain_saved: bool,
    pub profile_path: String,
    pub initial_snapshot: Option<SemanticSnapshot>,
}

/// Natural Human Behavior Engine: simulates humanized typing cadence,
/// stepped deceleration scrolling, reading hesitation, and element references.
pub struct HumanBehaviorEngine;

impl HumanBehaviorEngine {
    /// Realistic keystroke delays (45ms - 110ms with natural pauses on spaces and punctuation)
    pub fn keystroke_delay_for_char(c: char) -> Duration {
        match c {
            ' ' => Duration::from_millis(160),
            '.' | ',' | '!' | '?' | ';' | ':' => Duration::from_millis(260),
            '\n' => Duration::from_millis(320),
            _ => {
                let jitter = 50 + ((c as u32 % 45) as u64);
                Duration::from_millis(jitter)
            }
        }
    }

    /// Simulates natural reading pause / hesitation
    pub async fn hesitate_like_human(min_ms: u64, max_ms: u64) {
        let span = if max_ms > min_ms { max_ms - min_ms } else { 1 };
        let jitter = min_ms + (rand_simple() % span);
        tokio::time::sleep(Duration::from_millis(jitter)).await;
    }

    /// Types text character-by-character with realistic human jitter
    pub async fn type_like_human(page: &Page, selector: &str, text: &str) -> Result<(), DriverError> {
        let focus_js = format!(
            r#"(() => {{
                const el = document.querySelector({});
                if (el) {{
                    el.focus();
                    return true;
                }}
                return false;
            }})()"#,
            serde_json::to_string(selector).unwrap_or_default()
        );

        let focused: bool = page
            .evaluate(focus_js)
            .await
            .map_err(|e| DriverError::BrowserCapabilityBroken(format!("Failed to focus selector: {}", e)))?
            .into_value()
            .unwrap_or(false);

        if !focused {
            return Err(DriverError::BrowserCapabilityBroken(format!(
                "Element '{}' not found to type into",
                selector
            )));
        }

        for c in text.chars() {
            let delay = Self::keystroke_delay_for_char(c);
            let char_str = c.to_string();
            let append_js = format!(
                r#"(() => {{
                    const el = document.querySelector({});
                    if (el) {{
                        el.value = (el.value || '') + {};
                        el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    }}
                }})()"#,
                serde_json::to_string(selector).unwrap_or_default(),
                serde_json::to_string(&char_str).unwrap_or_default()
            );

            let _ = page.evaluate(append_js).await;
            tokio::time::sleep(delay).await;
        }

        Ok(())
    }

    /// Types text into an element referenced by its @e1, @e2 token
    pub async fn type_into_ref(page: &Page, ref_id: &str, text: &str) -> Result<(), DriverError> {
        let selector = format!("[data-agent-ref=\"{}\"]", ref_id);
        Self::type_like_human(page, &selector, text).await
    }

    /// Clicks an element referenced by its @e1, @e2 token
    pub async fn click_ref(page: &Page, ref_id: &str) -> Result<(), DriverError> {
        let click_js = format!(
            r#"(() => {{
                const el = document.querySelector('[data-agent-ref="{}"]');
                if (el) {{
                    el.scrollIntoView({{ behavior: 'smooth', block: 'center' }});
                    el.click();
                    return true;
                }}
                return false;
            }})()"#,
            ref_id
        );

        let clicked: bool = page
            .evaluate(click_js)
            .await
            .map_err(|e| DriverError::BrowserCapabilityBroken(format!("Failed to click ref {}: {}", ref_id, e)))?
            .into_value()
            .unwrap_or(false);

        if !clicked {
            return Err(DriverError::BrowserCapabilityBroken(format!(
                "Ref element '{}' not found",
                ref_id
            )));
        }

        Self::hesitate_like_human(250, 600).await;
        Ok(())
    }

    /// Smooth stepped scrolling with deceleration physics and frame jitter
    pub async fn scroll_like_human(page: &Page, total_pixels: i32, steps: usize) -> Result<(), DriverError> {
        if steps == 0 {
            return Ok(());
        }

        let step_count = steps.clamp(5, 30);
        let base_step = total_pixels as f32 / step_count as f32;

        for i in 0..step_count {
            let progress = i as f32 / step_count as f32;
            let ease_factor = 1.0 - (progress * 0.5);
            let delta = (base_step * ease_factor).round() as i32;

            let scroll_js = format!("window.scrollBy({{ top: {}, behavior: 'smooth' }});", delta);
            let _ = page.evaluate(scroll_js).await;

            let frame_delay = 20 + ((i as u64 * 3) % 20);
            tokio::time::sleep(Duration::from_millis(frame_delay)).await;
        }

        tokio::time::sleep(Duration::from_millis(150)).await;
        Ok(())
    }

    /// Takes a token-efficient semantic snapshot with @e1, @e2 element references
    pub async fn take_semantic_snapshot(page: &Page) -> Result<SemanticSnapshot, DriverError> {
        let raw_json: String = page
            .evaluate(SEMANTIC_SNAPSHOT_JS)
            .await
            .map_err(|e| DriverError::BrowserCapabilityBroken(format!("Failed to extract semantic snapshot: {}", e)))?
            .into_value()
            .unwrap_or_else(|_| "{}".to_string());

        let snapshot: SemanticSnapshot = serde_json::from_str(&raw_json).map_err(|e| {
            DriverError::BrowserCapabilityBroken(format!("Failed to parse semantic snapshot JSON: {}", e))
        })?;

        Ok(snapshot)
    }
}

fn rand_simple() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64
}

pub struct BrowserDriver {
    chrome_path: Option<PathBuf>,
    storage: Option<Arc<Storage>>,
    credential_store: Option<Arc<dyn CredentialStore>>,
}

impl BrowserDriver {
    pub fn new() -> Self {
        Self {
            chrome_path: Self::find_chrome_executable(),
            storage: None,
            credential_store: None,
        }
    }

    pub fn with_storage(mut self, storage: Arc<Storage>) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn with_credential_store(mut self, store: Arc<dyn CredentialStore>) -> Self {
        self.credential_store = Some(store);
        self
    }

    pub fn with_chrome_path(mut self, path: PathBuf) -> Self {
        self.chrome_path = Some(path);
        self
    }

    pub fn credential_store(&self) -> Option<Arc<dyn CredentialStore>> {
        self.credential_store.clone()
    }

    pub fn find_chrome_executable() -> Option<PathBuf> {
        if let Ok(env_path) = std::env::var("CHROME_BIN") {
            let p = PathBuf::from(env_path);
            if p.exists() {
                return Some(p);
            }
        }

        let candidates = [
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
            "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
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

    /// Saves session cookies encrypted into the OS Keychain via CredentialStore
    pub fn save_cookies_to_keychain(
        &self,
        alias: &str,
        jar: &SessionCookieJar,
    ) -> Result<(), DriverError> {
        let store = self.credential_store.as_ref().ok_or_else(|| {
            DriverError::Internal("No CredentialStore configured on BrowserDriver".to_string())
        })?;

        let json = serde_json::to_string(jar).map_err(|e| {
            DriverError::Internal(format!("Failed to serialize cookie jar: {}", e))
        })?;

        store
            .set_secret(
                crate::auth::AuthManager::keyring_service(),
                &crate::auth::AuthManager::keyring_cookies_account(alias),
                &json,
            )
            .map_err(|e| DriverError::Internal(format!("Keyring cookie save error: {}", e)))?;

        info!(
            "Encrypted {} session cookies saved to OS Keychain for alias '{}'",
            jar.cookies.len(),
            alias
        );
        Ok(())
    }

    /// Loads session cookies from the OS Keychain via CredentialStore
    pub fn load_cookies_from_keychain(
        &self,
        alias: &str,
    ) -> Result<Option<SessionCookieJar>, DriverError> {
        let store = match &self.credential_store {
            Some(s) => s,
            None => return Ok(None),
        };

        match store.get_secret(
            crate::auth::AuthManager::keyring_service(),
            &crate::auth::AuthManager::keyring_cookies_account(alias),
        ) {
            Ok(secret) => {
                let raw = secret.expose_secret();
                let clean_hex = raw
                    .strip_prefix("0x")
                    .or_else(|| raw.strip_prefix("0X"))
                    .unwrap_or(raw);

                let decoded_str = if clean_hex.len() % 2 == 0
                    && (clean_hex.starts_with("7b22") || clean_hex.starts_with("7B22"))
                    && clean_hex.chars().all(|c| c.is_ascii_hexdigit())
                {
                    if let Ok(bytes) = hex::decode(clean_hex) {
                        String::from_utf8(bytes).unwrap_or_else(|_| raw.to_string())
                    } else {
                        raw.to_string()
                    }
                } else {
                    raw.to_string()
                };

                let jar: SessionCookieJar =
                    serde_json::from_str(&decoded_str).map_err(|e| {
                        DriverError::Internal(format!(
                            "Failed to parse session cookies from Keychain: {}",
                            e
                        ))
                    })?;
                Ok(Some(jar))
            }
            Err(_) => Ok(None),
        }
    }

    /// Deletes session cookies from the OS Keychain
    pub fn delete_cookies_from_keychain(&self, alias: &str) -> Result<(), DriverError> {
        if let Some(store) = &self.credential_store {
            let _ = store.delete_secret(
                crate::auth::AuthManager::keyring_service(),
                &crate::auth::AuthManager::keyring_cookies_account(alias),
            );
        }
        Ok(())
    }

    /// Launches an isolated Chrome instance with Obscura anti-detection stealth arguments
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
            .window_size(1280, 900)
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--disable-infobars")
            .arg("--no-default-browser-check")
            .arg("--disable-background-networking")
            .arg("--disable-features=IsolateOrigins,site-per-process")
            .arg("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36");

        if !headless {
            builder = builder.with_head();
        }

        let config = builder.build().map_err(|e| {
            DriverError::BrowserCapabilityBroken(format!("Failed to build browser config: {}", e))
        })?;

        let (browser, mut handler) = Browser::launch(config).await.map_err(|e| {
            DriverError::BrowserCapabilityBroken(format!("Failed to launch Chrome browser: {}", e))
        })?;

        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    debug!("CDP event error: {}", e);
                }
            }
        });

        Ok(browser)
    }

    /// Injects Obscura anti-detection stealth overrides into a page
    pub async fn apply_stealth_engine(&self, page: &Page) -> Result<(), DriverError> {
        let _ = page.enable_stealth_mode().await;
        let _ = page.evaluate(OBSCURA_STEALTH_JS).await;
        Ok(())
    }

    /// Injects Keychain cookies into the page context
    pub async fn inject_keychain_cookies(
        &self,
        page: &Page,
        alias: &str,
    ) -> Result<usize, DriverError> {
        if let Some(jar) = self.load_cookies_from_keychain(alias)? {
            let count = jar.cookies.len();
            for c in &jar.cookies {
                let mut param = CookieParam::new(&c.name, &c.value);
                if let Some(dom) = &c.domain {
                    param.domain = Some(dom.clone());
                }
                if let Some(path) = &c.path {
                    param.path = Some(path.clone());
                }
                if let Some(sec) = c.secure {
                    param.secure = Some(sec);
                }
                if let Some(http) = c.http_only {
                    param.http_only = Some(http);
                }
                param.url = Some("https://www.reddit.com/".to_string());
                let _ = page.set_cookie(param).await;
            }
            debug!("Injected {} cookies from Keychain for '{}'", count, alias);
            Ok(count)
        } else {
            Ok(0)
        }
    }

    /// Extracts cookies from CDP and DOM to construct a SessionCookieJar
    pub async fn extract_cookies_from_page(
        &self,
        page: &Page,
        alias: &str,
        username: Option<String>,
    ) -> Result<SessionCookieJar, DriverError> {
        let cdp_cookies = page.get_cookies().await.unwrap_or_default();
        let mut session_cookies = Vec::new();

        for c in cdp_cookies {
            session_cookies.push(SessionCookie {
                name: c.name,
                value: c.value,
                domain: Some(c.domain),
                path: Some(c.path),
                secure: Some(c.secure),
                http_only: Some(c.http_only),
                same_site: c.same_site.map(|s| format!("{:?}", s)),
                expires: if c.expires > 0.0 {
                    Some(c.expires as i64)
                } else {
                    None
                },
            });
        }

        if let Ok(eval_val) = page.evaluate("document.cookie").await {
            if let Ok(cookie_str) = eval_val.into_value::<String>() {
                for pair in cookie_str.split(';') {
                    let parts: Vec<&str> = pair.trim().splitn(2, '=').collect();
                    if parts.len() == 2 {
                        let name = parts[0].trim().to_string();
                        let value = parts[1].trim().to_string();
                        if !session_cookies.iter().any(|c| c.name == name) {
                            session_cookies.push(SessionCookie {
                                name,
                                value,
                                domain: Some(".reddit.com".to_string()),
                                path: Some("/".to_string()),
                                secure: Some(true),
                                http_only: Some(false),
                                same_site: None,
                                expires: None,
                            });
                        }
                    }
                }
            }
        }

        let jar = SessionCookieJar {
            alias: alias.to_string(),
            reddit_username: username,
            captured_at: chrono::Utc::now().to_rfc3339(),
            cookie_count: session_cookies.len(),
            cookies: session_cookies,
        };

        Ok(jar)
    }

    /// Account Takeover & Interactive Login:
    /// 1. Launches visible Obscura stealth browser
    /// 2. Pre-injects Keychain cookies if present to resume active session
    /// 3. Navigates to target (or https://www.reddit.com/login)
    /// 4. Waits for user authentication / 2FA completion
    /// 5. Extracts and securely stores all session cookies in OS Keychain
    /// 6. Captures initial semantic snapshot and updates storage
    pub async fn takeover_account(
        &self,
        account_ctx: &AccountContext,
        target_url: Option<&str>,
        timeout_duration: Duration,
    ) -> Result<TakeoverResult, DriverError> {
        let alias = &account_ctx.account.alias;
        info!(
            "Launching Obscura stealth browser for account takeover: '{}'",
            alias
        );

        let browser = self.launch_isolated_browser(alias, false).await?;
        let page = browser
            .new_page("https://www.reddit.com/login")
            .await
            .map_err(|e| {
                DriverError::BrowserCapabilityBroken(format!("Failed to open browser page: {}", e))
            })?;

        self.apply_stealth_engine(&page).await?;

        let injected = self.inject_keychain_cookies(&page, alias).await.unwrap_or(0);
        if injected > 0 {
            info!(
                "Pre-injected {} existing Keychain cookies for '{}'",
                injected, alias
            );
        }

        if let Some(target) = target_url {
            if target != "https://www.reddit.com/login" {
                let _ = page.goto(target).await;
            }
        }

        let start = std::time::Instant::now();
        let mut verified_user = None;

        while start.elapsed() < timeout_duration {
            tokio::time::sleep(Duration::from_secs(2)).await;

            let current_url = page.url().await.unwrap_or(None).unwrap_or_default();
            if !current_url.contains("/login")
                && (current_url.contains("reddit.com") || current_url.starts_with("https://"))
            {
                let js = r#"
                    (() => {
                        const userEl = document.querySelector('header faceplate-tracker[noun="user_profile"], [data-testid="user-dropdown"], #USER_DROPDOWN_ID');
                        if (userEl) return userEl.textContent.trim();
                        const link = document.querySelector('header a[href^="/user/"]');
                        if (link) {
                            const m = link.getAttribute('href').match(/\/user\/([^\/]+)/);
                            if (m) return m[1];
                        }
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

                if current_url == "https://www.reddit.com/"
                    || current_url.contains("reddit.com/r/")
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
                "Interactive login / takeover timed out before user authentication was completed"
                    .to_string(),
            )
        })?;

        let jar = self
            .extract_cookies_from_page(&page, alias, Some(verified_username.clone()))
            .await?;
        let cookies_count = jar.cookies.len();
        let mut keychain_saved = false;

        if self.credential_store.is_some() {
            if let Err(e) = self.save_cookies_to_keychain(alias, &jar) {
                warn!("Failed to save cookies to Keychain: {}", e);
            } else {
                keychain_saved = true;
            }
        }

        let profile_dir = Self::profile_dir_for_account(alias);
        let profile_str = profile_dir.to_string_lossy().to_string();

        if let Some(storage) = &self.storage {
            let _ = storage.save_browser_profile(&account_ctx.account.id, &profile_str);
            let _ = storage.update_reddit_username(&account_ctx.account.id, &verified_username);
        }

        let initial_snapshot = HumanBehaviorEngine::take_semantic_snapshot(&page).await.ok();

        info!(
            "Account takeover verified for '{}' (u/{}), captured {} cookies (keychain={})",
            alias, verified_username, cookies_count, keychain_saved
        );

        Ok(TakeoverResult {
            account: alias.to_string(),
            verified_username,
            cookies_captured: cookies_count,
            keychain_saved,
            profile_path: profile_str,
            initial_snapshot,
        })
    }

    /// Interactive human login: opens visible Chrome window, waits for human authentication,
    /// persists cookies to Keychain, and returns verified username.
    pub async fn interactive_login(
        &self,
        account_ctx: &AccountContext,
        timeout_duration: Duration,
    ) -> Result<String, DriverError> {
        let res = self
            .takeover_account(account_ctx, None, timeout_duration)
            .await?;
        Ok(res.verified_username)
    }

    /// Navigates using Obscura stealth browser with human behavior simulation (hesitation,
    /// smooth momentum scrolling) and returns a compact semantic snapshot.
    pub async fn human_browse(
        &self,
        account_ctx: &AccountContext,
        target_url: &str,
        headless: bool,
    ) -> Result<SemanticSnapshot, DriverError> {
        let alias = &account_ctx.account.alias;
        let browser = self.launch_isolated_browser(alias, headless).await?;
        let page = browser.new_page(target_url).await.map_err(|e| {
            DriverError::BrowserCapabilityBroken(format!("Failed to open {}: {}", target_url, e))
        })?;

        self.apply_stealth_engine(&page).await?;
        let _ = self.inject_keychain_cookies(&page, alias).await;

        HumanBehaviorEngine::hesitate_like_human(800, 1600).await;
        let _ = HumanBehaviorEngine::scroll_like_human(&page, 450, 15).await;
        HumanBehaviorEngine::hesitate_like_human(400, 800).await;

        let snapshot = HumanBehaviorEngine::take_semantic_snapshot(&page).await?;
        Ok(snapshot)
    }

    /// Navigates to a Reddit settings page, enforcing human checkpoints for sensitive settings
    pub async fn open_settings(
        &self,
        account_ctx: &AccountContext,
        setting_page: &str,
    ) -> Result<String, DriverError> {
        let alias = &account_ctx.account.alias;

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

    /// Verifies session status by checking profile directory and Keychain cookies
    pub fn verify_session_profile(&self, account_ctx: &AccountContext) -> bool {
        let profile_dir = Self::profile_dir_for_account(&account_ctx.account.alias);
        if profile_dir.exists() {
            return true;
        }

        if let Ok(Some(jar)) = self.load_cookies_from_keychain(&account_ctx.account.alias) {
            return jar.cookie_count > 0;
        }

        false
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
                    "Neither browser profile directory nor Keychain cookies found for account '{}'",
                    alias
                ),
                remediation: format!(
                    "Run `reddit-agent account login {}` to authenticate and save session cookies.",
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
    use crate::auth::MemoryKeyring;
    use crate::storage::AccountRecord;

    #[test]
    fn test_find_chrome_executable() {
        let exe = BrowserDriver::find_chrome_executable();
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

    #[test]
    fn test_cookie_jar_serialization() {
        let jar = SessionCookieJar {
            alias: "test_alias".to_string(),
            reddit_username: Some("test_user".to_string()),
            captured_at: "2026-11-04T12:00:00Z".to_string(),
            cookie_count: 2,
            cookies: vec![
                SessionCookie {
                    name: "reddit_session".to_string(),
                    value: "sess_secret_val".to_string(),
                    domain: Some(".reddit.com".to_string()),
                    path: Some("/".to_string()),
                    secure: Some(true),
                    http_only: Some(true),
                    same_site: Some("Lax".to_string()),
                    expires: Some(1800000000),
                },
                SessionCookie {
                    name: "loid".to_string(),
                    value: "loid_secret_val".to_string(),
                    domain: Some(".reddit.com".to_string()),
                    path: Some("/".to_string()),
                    secure: Some(true),
                    http_only: Some(false),
                    same_site: None,
                    expires: None,
                },
            ],
        };

        let json = serde_json::to_string(&jar).unwrap();
        let decoded: SessionCookieJar = serde_json::from_str(&json).unwrap();
        assert_eq!(jar, decoded);
    }

    #[test]
    fn test_keychain_cookie_storage_memory() {
        let store = Arc::new(MemoryKeyring::new());
        let driver = BrowserDriver::new().with_credential_store(store.clone());

        let jar = SessionCookieJar {
            alias: "my_acc".to_string(),
            reddit_username: Some("reddit_dev".to_string()),
            captured_at: "2026-11-04T12:00:00Z".to_string(),
            cookie_count: 1,
            cookies: vec![SessionCookie {
                name: "token_v2".to_string(),
                value: "val123".to_string(),
                domain: Some(".reddit.com".to_string()),
                path: Some("/".to_string()),
                secure: Some(true),
                http_only: Some(true),
                same_site: None,
                expires: None,
            }],
        };

        // Save
        driver.save_cookies_to_keychain("my_acc", &jar).unwrap();

        // Load
        let loaded = driver.load_cookies_from_keychain("my_acc").unwrap().unwrap();
        assert_eq!(loaded.alias, "my_acc");
        assert_eq!(loaded.reddit_username.as_deref(), Some("reddit_dev"));
        assert_eq!(loaded.cookies.len(), 1);
        assert_eq!(loaded.cookies[0].name, "token_v2");

        // Delete
        driver.delete_cookies_from_keychain("my_acc").unwrap();
        let after_delete = driver.load_cookies_from_keychain("my_acc").unwrap();
        assert!(after_delete.is_none());
    }

    #[test]
    fn test_obscura_stealth_script_syntax() {
        assert!(OBSCURA_STEALTH_JS.contains("navigator.webdriver"));
        assert!(OBSCURA_STEALTH_JS.contains("window.chrome"));
        assert!(OBSCURA_STEALTH_JS.contains("WebGLRenderingContext"));
        assert!(OBSCURA_STEALTH_JS.contains("Apple M2 Pro (Metal)"));
    }

    #[test]
    fn test_semantic_snapshot_script_syntax() {
        assert!(SEMANTIC_SNAPSHOT_JS.contains("interactiveSelectors"));
        assert!(SEMANTIC_SNAPSHOT_JS.contains("@e${index + 1}"));
        assert!(SEMANTIC_SNAPSHOT_JS.contains("data-agent-ref"));
        assert!(SEMANTIC_SNAPSHOT_JS.contains("tree_text"));
    }

    #[test]
    fn test_human_behavior_delays() {
        let space_delay = HumanBehaviorEngine::keystroke_delay_for_char(' ');
        let char_delay = HumanBehaviorEngine::keystroke_delay_for_char('a');
        let punct_delay = HumanBehaviorEngine::keystroke_delay_for_char('.');

        assert!(space_delay >= Duration::from_millis(150));
        assert!(punct_delay >= Duration::from_millis(200));
        assert!(char_delay >= Duration::from_millis(50));
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

        let res = driver.open_settings(&ctx, "password").await;
        assert!(matches!(res, Err(DriverError::HumanCheckpointRequired(_))));

        let res2 = driver.open_settings(&ctx, "2fa").await;
        assert!(matches!(res2, Err(DriverError::HumanCheckpointRequired(_))));
    }
}
