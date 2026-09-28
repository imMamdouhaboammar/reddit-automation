use reddit_agent::accounts::AccountManager;
use reddit_agent::actions::{ActionError, ActionManager, RiskLevel};
use reddit_agent::audit::AuditLogger;
use reddit_agent::auth::{AuthManager, MemoryKeyring, SecretToken};
use reddit_agent::drivers::{Capability, Driver, DriverError, HealthReport};
use reddit_agent::issues::IssueEngine;
use reddit_agent::mcp::McpServer;
use reddit_agent::router::CapabilityRouter;
use reddit_agent::storage::Storage;
use serde_json::json;
use std::sync::Arc;

struct MockFullDriver;

#[async_trait::async_trait]
impl Driver for MockFullDriver {
    fn name(&self) -> &'static str {
        "mock_full_driver"
    }
    fn supports(&self, _cap: &Capability) -> bool {
        true
    }

    async fn execute(
        &self,
        cap: &Capability,
        ctx: &reddit_agent::accounts::AccountContext,
        in_val: serde_json::Value,
    ) -> Result<serde_json::Value, DriverError> {
        match cap {
            Capability::GetIdentity => Ok(json!({
                "identity": {
                    "name": ctx.account.reddit_username.as_deref().unwrap_or("anon"),
                    "link_karma": 1200,
                    "comment_karma": 3400,
                    "is_mod": false,
                    "over_18": false,
                }
            })),
            Capability::CreatePost => Ok(json!({
                "id": "t3_post999",
                "name": "t3_post999",
                "url": format!("https://reddit.com/{}/comments/post999", in_val["subreddit"].as_str().unwrap_or("r/test")),
            })),
            Capability::GetInbox => Ok(json!([])),
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
fn setup_test_harness() -> (
    Arc<Storage>,
    Arc<AccountManager>,
    Arc<ActionManager>,
    Arc<IssueEngine>,
    Arc<McpServer>,
) {
    let storage = Arc::new(Storage::new_in_memory().unwrap());
    let keyring = Arc::new(MemoryKeyring::new());
    let client = reqwest::Client::new();
    let auth = Arc::new(AuthManager::new(keyring.clone(), client));
    let account_mgr = Arc::new(AccountManager::new(storage.clone(), auth));

    let driver = Arc::new(MockFullDriver);
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
        "client_id".to_string(),
    ));

    let mcp = Arc::new(McpServer::new(
        account_mgr.clone(),
        router,
        action_mgr.clone(),
        issue_engine.clone(),
    ));

    (storage, account_mgr, action_mgr, issue_engine, mcp)
}

#[tokio::test]
async fn test_e2e_multi_account_isolation() {
    let (_, account_mgr, _, _, _) = setup_test_harness();

    // 1. Add account "personal" and account "company"
    let personal = account_mgr.add_account("personal").unwrap();
    let company = account_mgr.add_account("company").unwrap();

    account_mgr
        .link_identity(
            &personal.id,
            "personal_dev",
            &["read".to_string(), "submit".to_string()],
        )
        .unwrap();
    account_mgr
        .link_identity(
            &company.id,
            "acme_corp",
            &["identity".to_string(), "account".to_string()],
        )
        .unwrap();

    // 2. Default active is personal
    let default_ctx = account_mgr.get_account_context(None).unwrap();
    assert_eq!(default_ctx.account.alias, "personal");
    assert_eq!(
        default_ctx.account.reddit_username.as_deref(),
        Some("personal_dev")
    );

    // 3. Resolve company explicitly
    let company_ctx = account_mgr.get_account_context(Some("company")).unwrap();
    assert_eq!(company_ctx.account.alias, "company");
    assert_eq!(
        company_ctx.account.reddit_username.as_deref(),
        Some("acme_corp")
    );

    // 4. Switch default to company
    account_mgr.set_active_account("company").unwrap();
    let new_default = account_mgr.get_account_context(None).unwrap();
    assert_eq!(new_default.account.alias, "company");
    assert_eq!(
        new_default.account.reddit_username.as_deref(),
        Some("acme_corp")
    );
}

#[tokio::test]
async fn test_e2e_action_tampering_fails_closed() {
    let (storage, account_mgr, action_mgr, _, _) = setup_test_harness();

    let acc = account_mgr.add_account("brand").unwrap();
    account_mgr
        .link_identity(&acc.id, "brand_user", &["submit".to_string()])
        .unwrap();
    let ctx = account_mgr.get_account_context(Some("brand")).unwrap();

    // Prepare action
    let original_payload =
        json!({"subreddit": "rust", "title": "Original Title", "text": "Genuine content"});
    let plan = action_mgr
        .prepare_action(
            &ctx,
            "create_post",
            "r/rust",
            original_payload,
            RiskLevel::PublicWrite,
            vec![],
            30,
        )
        .unwrap();

    action_mgr.approve_action(&plan.id).unwrap();

    // Maliciously tamper with the stored payload in the database directly
    let mut raw_record = storage.get_pending_action(&plan.id).unwrap();
    raw_record.payload_json =
        json!({"subreddit": "rust", "title": "TAMPERED Malicious Title", "text": "Hacked"})
            .to_string();
    storage.save_pending_action(&raw_record).unwrap();

    // Execution MUST fail closed due to cryptographic SHA-256 hash mismatch
    let result = action_mgr.execute_action(&plan.id).await;
    assert!(matches!(result, Err(ActionError::PayloadTampered { .. })));
}

#[tokio::test]
async fn test_e2e_mcp_protocol_flow() {
    let (_, account_mgr, _, _, mcp) = setup_test_harness();

    let acc = account_mgr.add_account("agent_user").unwrap();
    account_mgr
        .link_identity(&acc.id, "ai_reddit_bot", &[])
        .unwrap();

    // 1. Tool call reddit_accounts
    let accounts_val = mcp
        .handle_tool_call("reddit_accounts", json!({}))
        .await
        .unwrap();
    assert!(!accounts_val.as_array().unwrap().is_empty());

    // 2. Tool call reddit_whoami
    let whoami_val = mcp
        .handle_tool_call("reddit_whoami", json!({"account": "agent_user"}))
        .await
        .unwrap();
    assert_eq!(whoami_val["identity"]["name"], "ai_reddit_bot");

    // 3. Tool call reddit_prepare_post
    let prep_val = mcp
        .handle_tool_call(
            "reddit_prepare_post",
            json!({
                "account": "agent_user",
                "subreddit": "rust",
                "title": "Agentic Reddit Operator in Rust",
                "text": "Tested and verified."
            }),
        )
        .await
        .unwrap();

    let action_id = prep_val["id"].as_str().unwrap();
    assert!(action_id.starts_with("act_"));

    // 3b. Approve action via MCP (explicit human/agent approval gate)
    let approve_val = mcp
        .handle_tool_call(
            "reddit_approve_action",
            json!({
                "action_id": action_id
            }),
        )
        .await
        .unwrap();
    assert_eq!(approve_val["status"], "approved");

    // 4. Tool call reddit_publish_prepared_post
    let pub_val = mcp
        .handle_tool_call(
            "reddit_publish_prepared_post",
            json!({
                "action_id": action_id
            }),
        )
        .await
        .unwrap();

    assert_eq!(pub_val["id"], "t3_post999");
    assert!(pub_val["url"]
        .as_str()
        .unwrap()
        .contains("comments/post999"));
}

#[test]
fn test_e2e_secret_token_redaction_guarantees() {
    let secret = SecretToken::new("top_secret_oauth_refresh_token_xyz");

    // 1. Display is redacted
    assert_eq!(format!("{}", secret), "[REDACTED]");

    // 2. Debug is redacted
    assert_eq!(format!("{:?}", secret), "[REDACTED]");

    // 3. Serde serialization is redacted
    let serialized = serde_json::to_string(&secret).unwrap();
    assert_eq!(serialized, "\"[REDACTED]\"");
    assert!(!serialized.contains("top_secret"));

    // 4. Expose strictly on demand for Bearer HTTP headers
    assert_eq!(secret.expose_secret(), "top_secret_oauth_refresh_token_xyz");
}
