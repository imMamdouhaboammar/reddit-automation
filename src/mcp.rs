use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use tracing::info;

use crate::accounts::AccountManager;
use crate::actions::{ActionManager, RiskLevel};
use crate::drivers::Capability;
use crate::issues::IssueEngine;
use crate::router::CapabilityRouter;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

pub struct McpServer {
    account_manager: Arc<AccountManager>,
    router: Arc<CapabilityRouter>,
    action_manager: Arc<ActionManager>,
    issue_engine: Arc<IssueEngine>,
}

impl McpServer {
    pub fn new(
        account_manager: Arc<AccountManager>,
        router: Arc<CapabilityRouter>,
        action_manager: Arc<ActionManager>,
        issue_engine: Arc<IssueEngine>,
    ) -> Self {
        Self {
            account_manager,
            router,
            action_manager,
            issue_engine,
        }
    }

    pub fn list_tools() -> Value {
        json!([
            {
                "name": "reddit_accounts",
                "description": "List all configured Reddit account aliases with active default indicator and verified username.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "reddit_whoami",
                "description": "Query identity, karma breakdown, and verified Reddit username for given account (or active default).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "account": { "type": "string", "description": "Optional account alias. Uses default if omitted." }
                    }
                }
            },
            {
                "name": "reddit_account_health",
                "description": "Run deterministic health diagnostics on an account (OAuth tokens, scopes, rate-limit, browser session).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "account": { "type": "string", "description": "Account alias to check." }
                    }
                }
            },
            {
                "name": "reddit_get_preferences",
                "description": "Read current Reddit preferences for the specified account.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "account": { "type": "string", "description": "Account alias." }
                    }
                }
            },
            {
                "name": "reddit_prepare_preference_change",
                "description": "Prepare a two-stage ActionPlan to patch Reddit account preferences.",
                "inputSchema": {
                    "type": "object",
                    "required": ["preferences"],
                    "properties": {
                        "account": { "type": "string", "description": "Account alias." },
                        "preferences": { "type": "object", "description": "Key-value map of preferences to update." }
                    }
                }
            },
            {
                "name": "reddit_get_subreddit",
                "description": "Fetch metadata and subscriber statistics for a community.",
                "inputSchema": {
                    "type": "object",
                    "required": ["subreddit"],
                    "properties": {
                        "account": { "type": "string", "description": "Account alias." },
                        "subreddit": { "type": "string", "description": "Subreddit name without r/ prefix." }
                    }
                }
            },
            {
                "name": "reddit_get_subreddit_rules",
                "description": "Retrieve official community rules and guidelines for a subreddit.",
                "inputSchema": {
                    "type": "object",
                    "required": ["subreddit"],
                    "properties": {
                        "account": { "type": "string", "description": "Account alias." },
                        "subreddit": { "type": "string", "description": "Subreddit name without r/ prefix." }
                    }
                }
            },
            {
                "name": "reddit_get_post_requirements",
                "description": "Fetch strict posting requirements (title regex, body length limits, required flair) for a subreddit.",
                "inputSchema": {
                    "type": "object",
                    "required": ["subreddit"],
                    "properties": {
                        "account": { "type": "string", "description": "Account alias." },
                        "subreddit": { "type": "string", "description": "Subreddit name." }
                    }
                }
            },
            {
                "name": "reddit_search",
                "description": "Search Reddit submissions within a subreddit or across Reddit.",
                "inputSchema": {
                    "type": "object",
                    "required": ["query"],
                    "properties": {
                        "account": { "type": "string", "description": "Account alias." },
                        "subreddit": { "type": "string", "description": "Target subreddit or 'all'." },
                        "query": { "type": "string", "description": "Search query terms." },
                        "limit": { "type": "integer", "description": "Max number of items to return." }
                    }
                }
            },
            {
                "name": "reddit_get_post",
                "description": "Fetch full submission details.",
                "inputSchema": {
                    "type": "object",
                    "required": ["post_id"],
                    "properties": {
                        "account": { "type": "string", "description": "Account alias." },
                        "subreddit": { "type": "string", "description": "Subreddit name." },
                        "post_id": { "type": "string", "description": "Post ID (e.g. '1h3xyz' or 't3_1h3xyz')." }
                    }
                }
            },
            {
                "name": "reddit_get_comments",
                "description": "Fetch comment tree for a specific post.",
                "inputSchema": {
                    "type": "object",
                    "required": ["post_id"],
                    "properties": {
                        "account": { "type": "string", "description": "Account alias." },
                        "subreddit": { "type": "string", "description": "Subreddit name." },
                        "post_id": { "type": "string", "description": "Post ID." }
                    }
                }
            },
            {
                "name": "reddit_prepare_post",
                "description": "Validate and prepare a two-stage ActionPlan to submit a post. Generates preview with cryptographic hash.",
                "inputSchema": {
                    "type": "object",
                    "required": ["subreddit", "title"],
                    "properties": {
                        "account": { "type": "string", "description": "Account alias." },
                        "subreddit": { "type": "string", "description": "Target subreddit." },
                        "title": { "type": "string", "description": "Submission title." },
                        "text": { "type": "string", "description": "Selftext markdown body." },
                        "url": { "type": "string", "description": "URL for link posts." },
                        "kind": { "type": "string", "enum": ["self", "link"], "description": "Post kind." },
                        "flair_id": { "type": "string", "description": "Optional flair template ID." },
                        "nsfw": { "type": "boolean" },
                        "spoiler": { "type": "boolean" }
                    }
                }
            },
            {
                "name": "reddit_approve_action",
                "description": "Approve a pending ActionPlan before execution. Enforces explicit approval for mutating and destructive actions.",
                "inputSchema": {
                    "type": "object",
                    "required": ["action_id"],
                    "properties": {
                        "action_id": { "type": "string", "description": "ActionPlan ID to approve (e.g. 'act_xxx')." }
                    }
                }
            },
            {
                "name": "reddit_publish_prepared_post",
                "description": "Execute an approved post ActionPlan. Verifies hash and publishes through Reddit API.",
                "inputSchema": {
                    "type": "object",
                    "required": ["action_id"],
                    "properties": {
                        "action_id": { "type": "string", "description": "ActionPlan ID (e.g. 'act_xxx')." }
                    }
                }
            },
            {
                "name": "reddit_prepare_comment",
                "description": "Prepare a two-stage ActionPlan to comment on a post or reply to another comment.",
                "inputSchema": {
                    "type": "object",
                    "required": ["thing_id", "text"],
                    "properties": {
                        "account": { "type": "string", "description": "Account alias." },
                        "thing_id": { "type": "string", "description": "Parent post or comment fullname (e.g. 't3_xxx' or 't1_xxx')." },
                        "text": { "type": "string", "description": "Markdown comment text." }
                    }
                }
            },
            {
                "name": "reddit_publish_prepared_comment",
                "description": "Execute an approved comment ActionPlan.",
                "inputSchema": {
                    "type": "object",
                    "required": ["action_id"],
                    "properties": {
                        "action_id": { "type": "string", "description": "ActionPlan ID." }
                    }
                }
            },
            {
                "name": "reddit_edit_own_content",
                "description": "Edit the text of your own post or comment.",
                "inputSchema": {
                    "type": "object",
                    "required": ["thing_id", "text"],
                    "properties": {
                        "account": { "type": "string", "description": "Account alias." },
                        "thing_id": { "type": "string", "description": "Fullname of content to edit." },
                        "text": { "type": "string", "description": "New text body." }
                    }
                }
            },
            {
                "name": "reddit_delete_own_content",
                "description": "Delete your own post or comment.",
                "inputSchema": {
                    "type": "object",
                    "required": ["id"],
                    "properties": {
                        "account": { "type": "string", "description": "Account alias." },
                        "id": { "type": "string", "description": "Fullname of content to delete (e.g. 't3_xxx' or 't1_xxx')." }
                    }
                }
            },
            {
                "name": "reddit_get_inbox",
                "description": "Fetch unread messages or notifications from Reddit inbox.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "account": { "type": "string", "description": "Account alias." },
                        "filter": { "type": "string", "enum": ["unread", "messages", "inbox"], "description": "Message category filter." }
                    }
                }
            },
            {
                "name": "reddit_diagnose_issue",
                "description": "Run diagnostic engine on an account or submission failure to identify root cause and evidence.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "account": { "type": "string", "description": "Account alias." },
                        "subreddit": { "type": "string", "description": "Optional subreddit if diagnosing submission failure." },
                        "error_message": { "type": "string", "description": "Optional error string returned by previous call." },
                        "payload": { "type": "object", "description": "Optional payload that was rejected." }
                    }
                }
            },
            {
                "name": "reddit_plan_repair",
                "description": "Generate a deterministic remediation plan for a diagnosed issue.",
                "inputSchema": {
                    "type": "object",
                    "required": ["diagnosis"],
                    "properties": {
                        "diagnosis": { "type": "object", "description": "DiagnosisReport returned from reddit_diagnose_issue." }
                    }
                }
            },
            {
                "name": "reddit_apply_safe_repair",
                "description": "Apply safe automated repair (e.g. preference patches, backoff waiting). Halts safely on human checkpoints.",
                "inputSchema": {
                    "type": "object",
                    "required": ["repair_plan"],
                    "properties": {
                        "account": { "type": "string", "description": "Account alias." },
                        "repair_plan": { "type": "object", "description": "RepairPlan returned from reddit_plan_repair." }
                    }
                }
            }
        ])
    }

    pub async fn handle_tool_call(&self, name: &str, args: Value) -> Result<Value, String> {
        let account_param = args["account"].as_str();
        let ctx = self
            .account_manager
            .get_account_context(account_param)
            .map_err(|e| format!("Failed to resolve account: {}", e))?;

        match name {
            "reddit_accounts" => {
                let accounts = self
                    .account_manager
                    .list_accounts()
                    .map_err(|e| e.to_string())?;
                Ok(json!(accounts))
            }

            "reddit_whoami" => {
                let res = self
                    .router
                    .execute(&Capability::GetIdentity, &ctx, json!({}))
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(res)
            }

            "reddit_account_health" => {
                let res = self
                    .router
                    .health_check(&ctx)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(json!(res))
            }

            "reddit_get_preferences" => {
                let res = self
                    .router
                    .execute(&Capability::GetPreferences, &ctx, json!({}))
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(res)
            }

            "reddit_prepare_preference_change" => {
                let prefs = args["preferences"].clone();
                let plan = self
                    .action_manager
                    .prepare_action(
                        &ctx,
                        "set_preferences",
                        "account_preferences",
                        prefs,
                        RiskLevel::Reversible,
                        vec![],
                        30,
                    )
                    .map_err(|e| e.to_string())?;
                Ok(json!(plan))
            }

            "reddit_get_subreddit" => {
                let sr = args["subreddit"].as_str().ok_or("Missing 'subreddit'")?;
                let res = self
                    .router
                    .execute(&Capability::GetSubreddit, &ctx, json!({"subreddit": sr}))
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(res)
            }

            "reddit_get_subreddit_rules" => {
                let sr = args["subreddit"].as_str().ok_or("Missing 'subreddit'")?;
                let res = self
                    .router
                    .execute(
                        &Capability::GetSubredditRules,
                        &ctx,
                        json!({"subreddit": sr}),
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(res)
            }

            "reddit_get_post_requirements" => {
                let sr = args["subreddit"].as_str().ok_or("Missing 'subreddit'")?;
                let res = self
                    .router
                    .execute(
                        &Capability::GetPostRequirements,
                        &ctx,
                        json!({"subreddit": sr}),
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(res)
            }

            "reddit_search" => {
                let res = self
                    .router
                    .execute(&Capability::Search, &ctx, args)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(res)
            }

            "reddit_get_post" => {
                let res = self
                    .router
                    .execute(&Capability::GetPost, &ctx, args)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(res)
            }

            "reddit_get_comments" => {
                let res = self
                    .router
                    .execute(&Capability::GetComments, &ctx, args)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(res)
            }

            "reddit_prepare_post" => {
                let sr = args["subreddit"].as_str().ok_or("Missing 'subreddit'")?;
                let title = args["title"].as_str().ok_or("Missing 'title'")?;
                let kind = args["kind"].as_str().unwrap_or("self");
                let url = args["url"].as_str();

                ActionManager::validate_post_submission(sr, title, kind, url)
                    .map_err(|e| e.to_string())?;

                let text = args["text"].as_str().unwrap_or("");
                let flair_id = args["flair_id"].as_str();

                // Retrieve subreddit requirements and compute warnings
                let mut warnings = Vec::new();
                if let Ok(reqs_val) = self
                    .router
                    .execute(
                        &Capability::GetPostRequirements,
                        &ctx,
                        json!({"subreddit": sr}),
                    )
                    .await
                {
                    if let Ok(reqs) =
                        serde_json::from_value::<crate::reddit::PostRequirements>(reqs_val)
                    {
                        warnings =
                            ActionManager::validate_post_requirements(&reqs, title, text, flair_id);
                    }
                }

                let plan = self
                    .action_manager
                    .prepare_action(
                        &ctx,
                        "create_post",
                        &format!("r/{}", sr),
                        args.clone(),
                        RiskLevel::PublicWrite,
                        warnings,
                        30,
                    )
                    .map_err(|e| e.to_string())?;

                Ok(json!(plan))
            }

            "reddit_approve_action" => {
                let action_id = args["action_id"].as_str().ok_or("Missing 'action_id'")?;
                let plan = self
                    .action_manager
                    .approve_action(action_id)
                    .map_err(|e| e.to_string())?;
                Ok(json!(plan))
            }

            "reddit_publish_prepared_post" => {
                let action_id = args["action_id"].as_str().ok_or("Missing 'action_id'")?;
                let res = self
                    .action_manager
                    .execute_action(action_id)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(res)
            }

            "reddit_prepare_comment" => {
                let thing_id = args["thing_id"].as_str().ok_or("Missing 'thing_id'")?;
                let plan = self
                    .action_manager
                    .prepare_action(
                        &ctx,
                        "create_comment",
                        thing_id,
                        args.clone(),
                        RiskLevel::PublicWrite,
                        vec![],
                        30,
                    )
                    .map_err(|e| e.to_string())?;
                Ok(json!(plan))
            }

            "reddit_publish_prepared_comment" => {
                let action_id = args["action_id"].as_str().ok_or("Missing 'action_id'")?;
                let res = self
                    .action_manager
                    .execute_action(action_id)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(res)
            }

            "reddit_edit_own_content" => {
                let res = self
                    .router
                    .execute(&Capability::EditPost, &ctx, args)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(res)
            }

            "reddit_delete_own_content" => {
                let res = self
                    .router
                    .execute(&Capability::DeletePost, &ctx, args)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(res)
            }

            "reddit_get_inbox" => {
                let res = self
                    .router
                    .execute(&Capability::GetInbox, &ctx, args)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(res)
            }

            "reddit_diagnose_issue" => {
                if let (Some(sr), Some(err), Some(payload)) = (
                    args["subreddit"].as_str(),
                    args["error_message"].as_str(),
                    args.get("payload"),
                ) {
                    let diag = self
                        .issue_engine
                        .diagnose_post_rejection(&ctx, sr, err, payload)
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(json!(diag))
                } else {
                    let diag = self
                        .issue_engine
                        .diagnose_account(&ctx)
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(json!(diag))
                }
            }

            "reddit_plan_repair" => {
                let diag: crate::issues::DiagnosisReport =
                    serde_json::from_value(args["diagnosis"].clone())
                        .map_err(|e| format!("Invalid diagnosis payload: {}", e))?;
                let plan = self
                    .issue_engine
                    .plan_repair(&diag)
                    .map_err(|e| e.to_string())?;
                Ok(json!(plan))
            }

            "reddit_apply_safe_repair" => {
                let plan: crate::issues::RepairPlan =
                    serde_json::from_value(args["repair_plan"].clone())
                        .map_err(|e| format!("Invalid repair plan payload: {}", e))?;
                let res = self
                    .issue_engine
                    .apply_safe_repair(&ctx, &plan)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(json!(res))
            }

            other => Err(format!("Unknown tool: {}", other)),
        }
    }

    /// Run the MCP Stdio JSON-RPC server loop
    pub async fn run_stdio_server(&self) -> io::Result<()> {
        info!("Reddit Agent MCP Server starting in stdio mode");
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let req: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(e) => {
                    let err_resp = json!({
                        "jsonrpc": "2.0",
                        "id": Value::Null,
                        "error": { "code": -32700, "message": format!("Parse error: {}", e) }
                    });
                    writeln!(stdout, "{}", serde_json::to_string(&err_resp).unwrap())?;
                    stdout.flush()?;
                    continue;
                }
            };

            let resp = match req.method.as_str() {
                "initialize" => {
                    json!({
                        "jsonrpc": "2.0",
                        "id": req.id,
                        "result": {
                            "protocolVersion": "2024-11-05",
                            "capabilities": {
                                "tools": {}
                            },
                            "serverInfo": {
                                "name": "reddit-agent",
                                "version": "0.1.0"
                            }
                        }
                    })
                }

                "notifications/initialized" => {
                    continue; // notification, no response required
                }

                "tools/list" => {
                    json!({
                        "jsonrpc": "2.0",
                        "id": req.id,
                        "result": {
                            "tools": Self::list_tools()
                        }
                    })
                }

                "tools/call" => {
                    let params = req.params.unwrap_or(Value::Null);
                    let tool_name = params["name"].as_str().unwrap_or("");
                    let arguments = params["arguments"].clone();

                    match self.handle_tool_call(tool_name, arguments).await {
                        Ok(res) => {
                            json!({
                                "jsonrpc": "2.0",
                                "id": req.id,
                                "result": {
                                    "content": [
                                        {
                                            "type": "text",
                                            "text": serde_json::to_string_pretty(&res).unwrap_or_default()
                                        }
                                    ],
                                    "isError": false
                                }
                            })
                        }
                        Err(err_msg) => {
                            json!({
                                "jsonrpc": "2.0",
                                "id": req.id,
                                "result": {
                                    "content": [
                                        {
                                            "type": "text",
                                            "text": format!("Error executing tool '{}': {}", tool_name, err_msg)
                                        }
                                    ],
                                    "isError": true
                                }
                            })
                        }
                    }
                }

                _ => {
                    json!({
                        "jsonrpc": "2.0",
                        "id": req.id,
                        "error": {
                            "code": -32601,
                            "message": format!("Method not found: {}", req.method)
                        }
                    })
                }
            };

            writeln!(stdout, "{}", serde_json::to_string(&resp).unwrap())?;
            stdout.flush()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditLogger;
    use crate::auth::MemoryKeyring;
    use crate::drivers::HealthReport;
    use crate::storage::Storage;

    struct DummyDriver;
    #[async_trait::async_trait]
    impl crate::drivers::Driver for DummyDriver {
        fn name(&self) -> &'static str {
            "dummy"
        }
        fn supports(&self, _cap: &Capability) -> bool {
            true
        }
        async fn execute(
            &self,
            cap: &Capability,
            _ctx: &crate::accounts::AccountContext,
            in_val: Value,
        ) -> Result<Value, crate::drivers::DriverError> {
            match cap {
                Capability::GetIdentity => Ok(json!({
                    "identity": {
                        "name": "tester",
                        "link_karma": 50,
                        "comment_karma": 100
                    }
                })),
                _ => Ok(in_val),
            }
        }
        async fn health_check(
            &self,
            ctx: &crate::accounts::AccountContext,
        ) -> Result<HealthReport, crate::drivers::DriverError> {
            Ok(HealthReport {
                account: ctx.account.alias.clone(),
                reddit_username: ctx.account.reddit_username.clone(),
                status: "healthy".to_string(),
                issues: vec![],
            })
        }
    }

    #[tokio::test]
    async fn test_mcp_tool_list_and_whoami_call() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let keyring = Arc::new(MemoryKeyring::new());
        let client = reqwest::Client::new();
        let auth = Arc::new(crate::auth::AuthManager::new(keyring, client));
        let account_mgr = Arc::new(AccountManager::new(storage.clone(), auth));

        let acc = account_mgr.add_account("personal").unwrap();
        account_mgr.link_identity(&acc.id, "my_user", &[]).unwrap();

        let driver = Arc::new(DummyDriver);
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
            storage,
            "cid".to_string(),
        ));

        let server = McpServer::new(account_mgr, router, action_mgr, issue_engine);

        // Verify tools list
        let tools = McpServer::list_tools();
        let tools_arr = tools.as_array().unwrap();
        assert!(tools_arr.iter().any(|t| t["name"] == "reddit_whoami"));
        assert!(tools_arr.iter().any(|t| t["name"] == "reddit_prepare_post"));
        assert!(tools_arr
            .iter()
            .any(|t| t["name"] == "reddit_diagnose_issue"));

        // Call reddit_whoami
        let res = server
            .handle_tool_call("reddit_whoami", json!({"account": "personal"}))
            .await
            .unwrap();
        assert_eq!(res["identity"]["name"], "tester");
    }
}
