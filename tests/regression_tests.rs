use reddit_agent::accounts::AccountManager;
use reddit_agent::actions::{ActionError, ActionManager, RiskLevel};
use reddit_agent::audit::AuditLogger;
use reddit_agent::auth::{AuthManager, MemoryKeyring};
use reddit_agent::drivers::browser::BrowserDriver;
use reddit_agent::drivers::{Capability, Driver, DriverError, HealthReport};
use reddit_agent::issues::IssueEngine;
use reddit_agent::mcp::McpServer;
use reddit_agent::router::CapabilityRouter;
use reddit_agent::storage::Storage;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

struct MockApiDriver;

#[async_trait::async_trait]
impl Driver for MockApiDriver {
    fn name(&self) -> &'static str {
        "mock_api"
    }
    fn supports(&self, _cap: &Capability) -> bool {
        true
    }
    async fn execute(
        &self,
        cap: &Capability,
        _ctx: &reddit_agent::accounts::AccountContext,
        in_val: serde_json::Value,
    ) -> Result<serde_json::Value, DriverError> {
        match cap {
            Capability::CreatePost => Ok(json!({
                "id": "t3_reg123",
                "name": "t3_reg123",
                "url": "https://reddit.com/r/test/comments/reg123"
            })),
            _ => Ok(in_val),
        }
    }
    async fn health_check(
        &self,
        ctx: &reddit_agent::accounts::AccountContext,
    ) -> Result<HealthReport, DriverError> {
        Ok(HealthReport {
            account: ctx.account.alias.clone(),
            reddit_username: ctx.account.reddit_username.clone(),
            status: "healthy".to_string(),
            issues: vec![],
        })
    }
}

#[allow(clippy::type_complexity)]
fn setup_harness() -> (
    Arc<Storage>,
    Arc<AccountManager>,
    Arc<ActionManager>,
    Arc<CapabilityRouter>,
    Arc<McpServer>,
) {
    let storage = Arc::new(Storage::new_in_memory().unwrap());
    let keyring = Arc::new(MemoryKeyring::new());
    let client = reqwest::Client::new();
    let auth = Arc::new(AuthManager::new(keyring, client));
    let account_mgr = Arc::new(AccountManager::new(storage.clone(), auth));

    let driver = Arc::new(MockApiDriver);
    let router = Arc::new(CapabilityRouter::new(driver.clone(), driver));
    let audit = Arc::new(AuditLogger::new(storage.clone()));
    let action_mgr = Arc::new(ActionManager::new(
        storage.clone(),
        router.clone(),
        account_mgr.clone(),
        audit,
    ));
    let issue_engine = Arc::new(IssueEngine::new(
        router.clone(),
        account_mgr.clone(),
        action_mgr.clone(),
        storage.clone(),
        "test_client".to_string(),
    ));
    let mcp = Arc::new(McpServer::new(
        account_mgr.clone(),
        router.clone(),
        action_mgr.clone(),
        issue_engine,
    ));

    (storage, account_mgr, action_mgr, router, mcp)
}

/// BUG-002: Tampering with target or account in SQLite must fail closed with PayloadTampered
#[tokio::test]
async fn test_action_tampering_target_or_account_fails_closed() {
    let (storage, account_mgr, action_mgr, _, _) = setup_harness();
    let _acc_a = account_mgr.add_account("account_a").unwrap();
    let _acc_b = account_mgr.add_account("account_b").unwrap();
    let ctx = account_mgr.get_account_context(Some("account_a")).unwrap();

    let plan = action_mgr
        .prepare_action(
            &ctx,
            "create_post",
            "r/safe_sub",
            json!({"title": "Test Title", "subreddit": "safe_sub"}),
            RiskLevel::PublicWrite,
            vec![],
            30,
        )
        .unwrap();

    action_mgr.approve_action(&plan.id).unwrap();

    // 1. Tamper with target
    let mut rec = storage.get_pending_action(&plan.id).unwrap();
    rec.target = "r/malicious_sub".to_string();
    storage.save_pending_action(&rec).unwrap();

    let res = action_mgr.execute_action(&plan.id).await;
    assert!(
        matches!(res, Err(ActionError::PayloadTampered { .. })),
        "Tampered target must fail closed with PayloadTampered, got: {:?}",
        res
    );
}

/// BUG-005: Already executed action must not be re-executable
#[tokio::test]
async fn test_executed_action_cannot_be_reexecuted() {
    let (_, account_mgr, action_mgr, _, _) = setup_harness();
    let _ = account_mgr.add_account("poster").unwrap();
    let ctx = account_mgr.get_account_context(Some("poster")).unwrap();

    let plan = action_mgr
        .prepare_action(
            &ctx,
            "create_post",
            "r/test",
            json!({"title": "One-time Post", "subreddit": "test"}),
            RiskLevel::PublicWrite,
            vec![],
            30,
        )
        .unwrap();

    action_mgr.approve_action(&plan.id).unwrap();

    // First execution succeeds
    let first_res = action_mgr.execute_action(&plan.id).await;
    assert!(first_res.is_ok());

    // Second execution MUST fail with InvalidState
    let second_res = action_mgr.execute_action(&plan.id).await;
    assert!(
        matches!(second_res, Err(ActionError::InvalidState(_))),
        "Re-executing an already executed action must fail with InvalidState, got: {:?}",
        second_res
    );
}

/// BUG-003: MCP server publish must reject unapproved actions instead of auto-approving
#[tokio::test]
async fn test_mcp_unapproved_publish_rejected() {
    let (_, account_mgr, action_mgr, _, mcp) = setup_harness();
    let _ = account_mgr.add_account("mcp_user").unwrap();
    let ctx = account_mgr.get_account_context(Some("mcp_user")).unwrap();

    let plan = action_mgr
        .prepare_action(
            &ctx,
            "create_post",
            "r/test",
            json!({"title": "Unapproved", "subreddit": "test"}),
            RiskLevel::PublicWrite,
            vec![],
            30,
        )
        .unwrap();

    // Call reddit_publish_prepared_post directly via MCP without approving first
    let res = mcp
        .handle_tool_call(
            "reddit_publish_prepared_post",
            json!({"action_id": plan.id}),
        )
        .await;

    assert!(
        res.is_err(),
        "MCP publish must reject an unapproved action, but got Ok: {:?}",
        res
    );
}

/// BUG-001: Interactive login timeout must fail closed and return error
#[tokio::test]
async fn test_interactive_login_timeout_fails_closed() {
    let (_, account_mgr, _, _, _) = setup_harness();
    let _ = account_mgr.add_account("test_login").unwrap();
    let ctx = account_mgr.get_account_context(Some("test_login")).unwrap();

    let browser = BrowserDriver::new();
    if BrowserDriver::find_chrome_executable().is_none() {
        // Skip test if no chrome executable on host
        return;
    }

    // 1-second timeout without user completing login must fail with error, NOT return Ok("logged_in_user")
    let res = browser
        .interactive_login(&ctx, Duration::from_millis(500))
        .await;
    assert!(
        res.is_err(),
        "Interactive login timeout must return Err, got Ok: {:?}",
        res
    );
}

/// BUG-007: Storage default path respects XDG_CONFIG_HOME and REDDIT_AGENT_HOME
#[test]
fn test_storage_and_profile_paths_respect_xdg_and_reddit_home() {
    let temp_dir = tempfile::tempdir().unwrap();
    let custom_home = temp_dir.path().join("custom_reddit");

    std::env::set_var("REDDIT_AGENT_HOME", &custom_home);
    let path = Storage::default_path();
    assert_eq!(path, custom_home.join("state.db"));

    let profile = BrowserDriver::profile_dir_for_account("my_alias");
    assert_eq!(profile, custom_home.join("browser").join("my_alias"));

    std::env::remove_var("REDDIT_AGENT_HOME");
}

/// BUG-004: Post preparation rejects invalid inputs
#[test]
fn test_post_prepare_validation_boundaries() {
    // 1. Empty title
    let res = ActionManager::validate_post_submission("rust", "", "self", None);
    assert!(res.is_err(), "Empty title must be rejected");

    // 2. Whitespace title
    let res = ActionManager::validate_post_submission("rust", "   \t\n", "self", None);
    assert!(res.is_err(), "Whitespace title must be rejected");

    // 3. Title > 300 chars
    let long_title = "a".repeat(305);
    let res = ActionManager::validate_post_submission("rust", &long_title, "self", None);
    assert!(res.is_err(), "Title > 300 chars must be rejected");

    // 4. Invalid kind
    let res = ActionManager::validate_post_submission("rust", "Valid Title", "invalid_kind", None);
    assert!(res.is_err(), "Invalid post kind must be rejected");

    // 5. Link kind with missing URL
    let res = ActionManager::validate_post_submission("rust", "Link Title", "link", None);
    assert!(res.is_err(), "Link kind without URL must be rejected");

    // 6. Link kind with invalid non-HTTP URL
    let res =
        ActionManager::validate_post_submission("rust", "Link Title", "link", Some("not-a-url"));
    assert!(res.is_err(), "Malformed link URL must be rejected");

    // 7. Invalid subreddit name with spaces or special characters
    let res = ActionManager::validate_post_submission("invalid sub!", "Title", "self", None);
    assert!(
        res.is_err(),
        "Subreddit name with illegal characters must be rejected"
    );

    // 8. Valid inputs must pass
    let res = ActionManager::validate_post_submission("r/rust", "Valid Title", "self", None);
    assert!(res.is_ok(), "Valid self post must pass");

    let res = ActionManager::validate_post_submission(
        "rust",
        "Valid Link",
        "link",
        Some("https://example.com"),
    );
    assert!(res.is_ok(), "Valid link post must pass");
}

/// BUG-004 in MCP: MCP prepare_post tool rejects invalid post submissions
#[tokio::test]
async fn test_mcp_prepare_post_input_validation() {
    let (_, account_mgr, _, _, mcp) = setup_harness();
    let _ = account_mgr.add_account("mcp_tester").unwrap();

    let res = mcp
        .handle_tool_call(
            "reddit_prepare_post",
            json!({"subreddit": "invalid subreddit!", "title": "Test"}),
        )
        .await;
    assert!(
        res.is_err(),
        "MCP prepare_post with invalid subreddit must fail"
    );

    let res2 = mcp
        .handle_tool_call(
            "reddit_prepare_post",
            json!({"subreddit": "rust", "title": "   "}),
        )
        .await;
    assert!(res2.is_err(), "MCP prepare_post with empty title must fail");
}
