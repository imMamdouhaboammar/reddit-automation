use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use tracing::info;

use crate::accounts::AccountManager;
use crate::actions::{ActionManager, RiskLevel};
use crate::agents::community_builder::CommunityBuilderWorker;
use crate::agents::opportunity::{
    DiggerSource, OpportunityWorker, ProductContext, RankedOpportunity,
};
use crate::drivers::reddit_keyless::RedditKeylessDriver;
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
    browser_driver: Option<Arc<crate::drivers::browser::BrowserDriver>>,
    storage: Option<Arc<crate::storage::Storage>>,
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
            browser_driver: None,
            storage: None,
        }
    }

    pub fn with_browser_driver(
        mut self,
        browser_driver: Arc<crate::drivers::browser::BrowserDriver>,
    ) -> Self {
        self.browser_driver = Some(browser_driver);
        self
    }

    pub fn with_storage(mut self, storage: Arc<crate::storage::Storage>) -> Self {
        self.storage = Some(storage);
        self
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
            },
            {
                "name": "reddit_digger",
                "description": "Deep candidate source gatherer (reddit-digger). Scans and maps where search happens, executes multi-tier scraping (Public Shreddit + Arctic Shift archive), and gathers candidate threads with canonical URLs, metrics, and previews.",
                "inputSchema": {
                    "type": "object",
                    "required": ["target_subreddits"],
                    "properties": {
                        "product_name": { "type": "string", "description": "Optional name of product/context." },
                        "target_subreddits": { "type": "array", "items": { "type": "string" }, "description": "List of subreddits to deeply scan (e.g. ['SaaS', 'startups'])." },
                        "high_intent_phrases": { "type": "array", "items": { "type": "string" }, "description": "Search phrases or question queries." },
                        "limit_per_sub": { "type": "integer", "description": "Max threads to collect per subreddit (default: 15)." }
                    }
                }
            },
            {
                "name": "reddit_miner",
                "description": "Deep content extraction and opportunity analyzer (reddit-miner). Takes candidate sources from reddit_digger, extracts full text, sanitizes prompt injection, detects intent signals (RecommendationAsk, ExpressedPain, CompetitorFrustration, UrgentWorkflow), scores product fit, and ranks opportunities.",
                "inputSchema": {
                    "type": "object",
                    "required": ["sources", "product"],
                    "properties": {
                        "sources": { "type": "array", "items": { "type": "object" }, "description": "Array of DiggerSource objects returned by reddit_digger." },
                        "product": { "type": "object", "description": "ProductContext object containing product_name, pain_points, competitors, etc." }
                    }
                }
            },
            {
                "name": "reddit_discover_opportunities",
                "description": "Unified discovery pipeline: runs reddit_digger followed by reddit_miner to find and rank high-intent Reddit opportunities in a single call.",
                "inputSchema": {
                    "type": "object",
                    "required": ["product_name", "value_prop", "target_subreddits"],
                    "properties": {
                        "product_name": { "type": "string", "description": "Name of product/service." },
                        "value_prop": { "type": "string", "description": "One-line value proposition." },
                        "icp_target": { "type": "string", "description": "Ideal customer profile." },
                        "target_subreddits": { "type": "array", "items": { "type": "string" }, "description": "List of subreddits (e.g. ['SaaS', 'startups'])." },
                        "pain_points": { "type": "array", "items": { "type": "string" }, "description": "Specific pain points the product solves." },
                        "competitors": { "type": "array", "items": { "type": "string" }, "description": "Competitor tools commonly mentioned." },
                        "high_intent_phrases": { "type": "array", "items": { "type": "string" }, "description": "Phrases buyers use when seeking solutions." },
                        "affiliation_disclosure": { "type": "string", "description": "Mandatory affiliation disclaimer (e.g. 'Full disclosure: I work on X')." },
                        "limit_per_sub": { "type": "integer", "description": "Max threads to fetch per subreddit (default: 15)." }
                    }
                }
            },
            {
                "name": "reddit_draft_response",
                "description": "Draft a genuinely helpful 25-55 word response adhering to Experience Grammar ('I ran into this...') with mandatory affiliation disclosure and anti-slop rules.",
                "inputSchema": {
                    "type": "object",
                    "required": ["opportunity", "product"],
                    "properties": {
                        "opportunity": { "type": "object", "description": "RankedOpportunity returned by reddit_miner or reddit_discover_opportunities." },
                        "product": { "type": "object", "description": "ProductContext object." }
                    }
                }
            },
            {
                "name": "reddit_audit_community_ratio",
                "description": "Audit an account's karma and recent posting history against Reddit's 90/10 content-to-promotion rule.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "account": { "type": "string", "description": "Account alias to audit." }
                    }
                }
            },
            {
                "name": "reddit_2_skill",
                "description": "Mine deep Reddit consensus and engineer a production-grade Agent Skill conforming to OmniSkill v6.2.0 and Poteto Mode.",
                "inputSchema": {
                    "type": "object",
                    "required": ["topic"],
                    "properties": {
                        "topic": { "type": "string", "description": "Topic or problem domain to research and synthesize." },
                        "subreddits": { "type": "array", "items": { "type": "string" }, "description": "Optional list of subreddits to focus on." },
                        "output_dir": { "type": "string", "description": "Optional base directory to write the generated skill." }
                    }
                }
            },
            {
                "name": "reddit_agent_route",
                "description": "Dynamic OmniSkill Agent Router: analyzes any user intent/prompt (in Arabic or English), classifies intent category, and compiles an optimal multi-step Directed Acyclic Graph (DAG) with dependency waves and safety notes.",
                "inputSchema": {
                    "type": "object",
                    "required": ["prompt"],
                    "properties": {
                        "prompt": { "type": "string", "description": "Natural language request, instruction, or goal in any language." },
                        "context": { "type": "object", "description": "Optional context object with product_name, subreddits, etc." }
                    }
                }
            },
            {
                "name": "reddit_agent_orchestrate",
                "description": "Master OmniSkill Orchestrator: autonomously executes a natural language goal wave-by-wave by dispatching specialized Tool-Agents (DiggerAgent, MinerAgent, DrafterAgent) with fail-closed safety checkpoints.",
                "inputSchema": {
                    "type": "object",
                    "required": ["prompt"],
                    "properties": {
                        "prompt": { "type": "string", "description": "Natural language goal to orchestrate." },
                        "product": { "type": "object", "description": "Optional ProductContext object." }
                    }
                }
            },
            {
                "name": "reddit_account_login",
                "description": "Interactive Reddit account login & takeover using Obscura stealth browser. Opens visible anti-detection window, restores existing Keychain session if present, waits for authentication, and securely stores session cookies encrypted in OS Keychain.",
                "inputSchema": {
                    "type": "object",
                    "required": ["account"],
                    "properties": {
                        "account": { "type": "string", "description": "Account alias to log in." },
                        "timeout_secs": { "type": "integer", "description": "Timeout in seconds to wait for login (default: 180)." }
                    }
                }
            },
            {
                "name": "reddit_takeover_account",
                "description": "Initiate Obscura stealth account takeover with optional custom start URL, monitor session tokens, and persist encrypted cookies to OS Keychain.",
                "inputSchema": {
                    "type": "object",
                    "required": ["account"],
                    "properties": {
                        "account": { "type": "string", "description": "Account alias to take over." },
                        "url": { "type": "string", "description": "Optional starting target URL (default: https://www.reddit.com/login)." },
                        "timeout_secs": { "type": "integer", "description": "Timeout in seconds (default: 180)." }
                    }
                }
            },
            {
                "name": "reddit_account_cookies",
                "description": "Inspect metadata for account session cookies stored in OS Keychain (cookie count, verified username, timestamp) with zero raw token exposure.",
                "inputSchema": {
                    "type": "object",
                    "required": ["account"],
                    "properties": {
                        "account": { "type": "string", "description": "Account alias." }
                    }
                }
            },
            {
                "name": "reddit_human_browse",
                "description": "Stealth browser navigation with human behavior emulation (anti-detection flags, Keychain cookie restoration, humanized reading hesitation, smooth momentum scrolling) returning a compact semantic element snapshot (@e1, @e2 references) for AI agent interaction.",
                "inputSchema": {
                    "type": "object",
                    "required": ["url"],
                    "properties": {
                        "account": { "type": "string", "description": "Optional account alias. Restores Keychain cookies if provided." },
                        "url": { "type": "string", "description": "Target Reddit URL to browse (e.g. 'https://www.reddit.com/r/rust')." },
                        "headless": { "type": "boolean", "description": "Run in headless mode (default: true)." }
                    }
                }
            },
            {
                "name": "reddit_ontology_get",
                "description": "Inspect the semantic Community Ontology (terms, taboo rules, certified angles, relationships) for a given subreddit.",
                "inputSchema": {
                    "type": "object",
                    "required": ["subreddit"],
                    "properties": {
                        "subreddit": { "type": "string", "description": "Target subreddit name (e.g. 'vibecoding', 'SaaS')." }
                    }
                }
            },
            {
                "name": "reddit_ontology_check_engagable",
                "description": "Preflight Answerability Gate: verifies whether a thread can be safely engaged without risk of bans, evaluating against community taboos and human-certified angles.",
                "inputSchema": {
                    "type": "object",
                    "required": ["subreddit", "title"],
                    "properties": {
                        "subreddit": { "type": "string", "description": "Target subreddit name." },
                        "title": { "type": "string", "description": "Reddit thread title." },
                        "body": { "type": "string", "description": "Reddit thread body content." },
                        "product_name": { "type": "string", "description": "Optional product name override." },
                        "pain_point": { "type": "string", "description": "Optional pain point override." }
                    }
                }
            },
            {
                "name": "reddit_ontology_certify_angle",
                "description": "Human-in-the-loop certification: approve a value-first engagement angle and mandatory disclosure for a subreddit.",
                "inputSchema": {
                    "type": "object",
                    "required": ["subreddit", "title", "pitch", "target_pain_point"],
                    "properties": {
                        "subreddit": { "type": "string", "description": "Subreddit to certify angle for." },
                        "title": { "type": "string", "description": "Descriptive title of the angle." },
                        "pitch": { "type": "string", "description": "Approved value-first pitch or solution pattern." },
                        "target_pain_point": { "type": "string", "description": "Target pain point or recurring trigger." },
                        "mandatory_disclosure": { "type": "string", "description": "Mandatory affiliation disclosure." },
                        "certified_by": { "type": "string", "description": "Reviewer name or identifier." }
                    }
                }
            },
            {
                "name": "reddit_ontology_list",
                "description": "List all subreddits currently configured with active Community Ontologies.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }
        ])
    }

    pub async fn handle_tool_call(&self, name: &str, args: Value) -> Result<Value, String> {
        if name == "reddit_agent_route" {
            let prompt = args["prompt"]
                .as_str()
                .ok_or_else(|| "Missing required 'prompt' argument".to_string())?;

            let context_opt = args.get("context").cloned();
            let dag =
                crate::agents::router::DynamicRouter::build_execution_dag(prompt, context_opt);
            return Ok(json!(dag));
        }

        if name == "reddit_agent_orchestrate" {
            let prompt = args["prompt"]
                .as_str()
                .ok_or_else(|| "Missing required 'prompt' argument".to_string())?;

            let product: Option<crate::agents::opportunity::ProductContext> = args
                .get("product")
                .and_then(|p| serde_json::from_value(p.clone()).ok());

            let orchestrator = crate::agents::orchestrator::MasterOrchestrator::new();
            let report = orchestrator
                .orchestrate_from_prompt(prompt, product)
                .await?;
            return Ok(json!(report));
        }

        if name == "reddit_account_login" || name == "reddit_takeover_account" {
            let account = args["account"]
                .as_str()
                .ok_or_else(|| "Missing required 'account' argument".to_string())?;

            let timeout_secs = args["timeout_secs"].as_u64().unwrap_or(180);
            let url = args["url"].as_str();

            let ctx = self
                .account_manager
                .get_account_context(Some(account))
                .map_err(|e| format!("Account error: {}", e))?;

            let browser_driver = self.browser_driver.clone().unwrap_or_else(|| {
                Arc::new(
                    crate::drivers::browser::BrowserDriver::new()
                        .with_credential_store(self.account_manager.auth().credential_store()),
                )
            });

            let res = browser_driver
                .takeover_account(&ctx, url, std::time::Duration::from_secs(timeout_secs))
                .await
                .map_err(|e| format!("Takeover failed: {}", e))?;

            return Ok(json!(res));
        }

        if name == "reddit_account_cookies" {
            let account = args["account"]
                .as_str()
                .ok_or_else(|| "Missing required 'account' argument".to_string())?;

            let _ctx = self
                .account_manager
                .get_account_context(Some(account))
                .map_err(|e| format!("Account error: {}", e))?;

            let browser_driver = self.browser_driver.clone().unwrap_or_else(|| {
                Arc::new(
                    crate::drivers::browser::BrowserDriver::new()
                        .with_credential_store(self.account_manager.auth().credential_store()),
                )
            });

            let maybe_jar = browser_driver
                .load_cookies_from_keychain(account)
                .map_err(|e| format!("Failed to read Keychain: {}", e))?;

            return match maybe_jar {
                Some(jar) => Ok(json!({
                    "account": account,
                    "has_cookies": true,
                    "verified_username": jar.reddit_username,
                    "captured_at": jar.captured_at,
                    "cookie_count": jar.cookie_count,
                    "keychain_storage": "Encrypted in OS Keychain ('reddit-agent')",
                    "cookie_names": jar.cookies.iter().map(|c| c.name.clone()).collect::<Vec<_>>()
                })),
                None => Ok(json!({
                    "account": account,
                    "has_cookies": false,
                    "message": format!(
                        "No cookies found in OS Keychain for account '{}'. Run reddit_account_login.",
                        account
                    )
                })),
            };
        }

        if name == "reddit_human_browse" {
            let url = args["url"]
                .as_str()
                .ok_or_else(|| "Missing required 'url' argument".to_string())?;

            let account = args["account"].as_str();
            let headless = args["headless"].as_bool().unwrap_or(true);

            let ctx = self
                .account_manager
                .get_account_context(account)
                .map_err(|e| format!("Account error: {}", e))?;

            let browser_driver = self.browser_driver.clone().unwrap_or_else(|| {
                Arc::new(
                    crate::drivers::browser::BrowserDriver::new()
                        .with_credential_store(self.account_manager.auth().credential_store()),
                )
            });

            let snapshot = browser_driver
                .human_browse(&ctx, url, headless)
                .await
                .map_err(|e| format!("Human browse failed: {}", e))?;

            return Ok(json!(snapshot));
        }

        if name == "reddit_2_skill" {
            let topic = args["topic"]
                .as_str()
                .ok_or_else(|| "Missing required 'topic' argument".to_string())?;

            let subreddits: Option<Vec<String>> = args["subreddits"].as_array().map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            });

            let output_dir_str = args["output_dir"].as_str();
            let output_dir_path = output_dir_str.map(std::path::Path::new);

            let engine = crate::agents::reddit_2_skill::Reddit2SkillEngine::new();
            let package = engine
                .mine_and_forge_skill(topic, subreddits, output_dir_path)
                .await
                .map_err(|e| format!("reddit-2-skill failed: {}", e))?;

            return Ok(json!(package));
        }

        if name == "reddit_digger" || name == "reddit-digger" {
            let subreddits: Vec<String> = args["target_subreddits"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let high_intent_phrases: Vec<String> = args["high_intent_phrases"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let limit_per_sub = args["limit_per_sub"].as_u64().unwrap_or(15) as usize;

            let keyless_driver = RedditKeylessDriver::new();
            let threads = keyless_driver
                .discover_threads(&subreddits, &high_intent_phrases, limit_per_sub)
                .await;

            let digger_result =
                OpportunityWorker::dig_sources(&threads, &subreddits, &high_intent_phrases);
            return Ok(json!(digger_result));
        }

        if name == "reddit_miner" || name == "reddit-miner" {
            let sources: Vec<DiggerSource> = serde_json::from_value(args["sources"].clone())
                .unwrap_or_else(|_| {
                    // Try parsing as raw threads if passed directly
                    let threads: Vec<crate::agents::opportunity::RedditThread> =
                        serde_json::from_value(args["sources"].clone()).unwrap_or_default();
                    threads.iter().map(DiggerSource::from).collect()
                });

            let product: ProductContext = serde_json::from_value(args["product"].clone())
                .map_err(|e| format!("Invalid product payload: {}", e))?;

            let now_utc = chrono::Utc::now().timestamp();
            let miner_result = OpportunityWorker::mine_content(&sources, &product, now_utc);
            return Ok(json!(miner_result));
        }

        if name == "reddit_discover_opportunities" {
            let product_name = args["product_name"]
                .as_str()
                .unwrap_or("Product")
                .to_string();
            let value_prop = args["value_prop"].as_str().unwrap_or("").to_string();
            let icp_target = args["icp_target"].as_str().unwrap_or("").to_string();
            let affiliation_disclosure = args["affiliation_disclosure"]
                .as_str()
                .unwrap_or("Full disclosure: I am affiliated with this product")
                .to_string();

            let subreddits: Vec<String> = args["target_subreddits"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let pain_points: Vec<String> = args["pain_points"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let competitors: Vec<String> = args["competitors"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let high_intent_phrases: Vec<String> = args["high_intent_phrases"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let limit_per_sub = args["limit_per_sub"].as_u64().unwrap_or(15) as usize;

            let product = ProductContext {
                product_name,
                value_prop,
                icp_target,
                pain_points,
                competitors,
                target_subreddits: subreddits.clone(),
                high_intent_phrases: high_intent_phrases.clone(),
                affiliation_disclosure,
            };

            let keyless_driver = RedditKeylessDriver::new();
            let threads = keyless_driver
                .discover_threads(&subreddits, &high_intent_phrases, limit_per_sub)
                .await;

            let digger_result =
                OpportunityWorker::dig_sources(&threads, &subreddits, &high_intent_phrases);
            let now_utc = chrono::Utc::now().timestamp();
            let miner_result =
                OpportunityWorker::mine_content(&digger_result.sources, &product, now_utc);
            return Ok(json!(miner_result.opportunities));
        }

        if name == "reddit_draft_response" {
            let opp: RankedOpportunity = serde_json::from_value(args["opportunity"].clone())
                .map_err(|e| format!("Invalid opportunity payload: {}", e))?;
            let product: ProductContext = serde_json::from_value(args["product"].clone())
                .map_err(|e| format!("Invalid product payload: {}", e))?;

            let draft = OpportunityWorker::draft_reply(&opp, &product)
                .map_err(|e| format!("Drafting failed: {}", e))?;
            return Ok(json!(draft));
        }

        if name == "reddit_ontology_get" {
            let subreddit = args["subreddit"]
                .as_str()
                .ok_or_else(|| "Missing required 'subreddit' argument".to_string())?
                .trim_start_matches("r/");

            if let Some(storage) = &self.storage {
                if let Ok(Some(ont)) = storage.get_community_ontology(subreddit) {
                    return Ok(json!(ont));
                }
            }

            // Fallback: build baseline ontology
            let baseline = crate::ontology::AutoOntologyBuilder::build_from_threads(
                subreddit,
                &format!("Domain knowledge for r/{}", subreddit),
                &[],
            );
            return Ok(json!(baseline));
        }

        if name == "reddit_ontology_check_engagable" {
            let subreddit = args["subreddit"]
                .as_str()
                .ok_or_else(|| "Missing required 'subreddit' argument".to_string())?
                .trim_start_matches("r/");
            let title = args["title"]
                .as_str()
                .ok_or_else(|| "Missing required 'title' argument".to_string())?;
            let body = args["body"].as_str().unwrap_or("");

            let ontology = if let Some(storage) = &self.storage {
                storage.get_community_ontology(subreddit).ok().flatten()
            } else {
                None
            }
            .unwrap_or_else(|| {
                crate::ontology::AutoOntologyBuilder::build_from_threads(
                    subreddit,
                    &format!("Domain knowledge for r/{}", subreddit),
                    &[],
                )
            });

            let mut context = ProductContext::default();
            if let Some(p_name) = args["product_name"].as_str() {
                context.product_name = p_name.to_string();
            }
            if let Some(p_pain) = args["pain_point"].as_str() {
                context.pain_points.push(p_pain.to_string());
            }

            let thread = crate::agents::opportunity::RedditThread {
                id: "eval_thread".to_string(),
                subreddit: subreddit.to_string(),
                title: title.to_string(),
                body: body.to_string(),
                author: "evaluator".to_string(),
                created_utc: chrono::Utc::now().timestamp(),
                num_comments: 0,
                is_locked: false,
                is_archived: false,
                url: "".to_string(),
            };

            let verdict = crate::ontology::PreflightAnswerabilityGate::check_engagable(
                &thread, &ontology, &context,
            );

            return Ok(json!(verdict));
        }

        if name == "reddit_ontology_certify_angle" {
            let subreddit = args["subreddit"]
                .as_str()
                .ok_or_else(|| "Missing required 'subreddit' argument".to_string())?
                .trim_start_matches("r/");
            let title = args["title"]
                .as_str()
                .ok_or_else(|| "Missing required 'title' argument".to_string())?;
            let pitch = args["pitch"]
                .as_str()
                .ok_or_else(|| "Missing required 'pitch' argument".to_string())?;
            let target_pain_point = args["target_pain_point"]
                .as_str()
                .ok_or_else(|| "Missing required 'target_pain_point' argument".to_string())?;
            let mandatory_disclosure = args["mandatory_disclosure"]
                .as_str()
                .unwrap_or("Full disclosure: I am working on this tool");
            let certified_by = args["certified_by"].as_str().unwrap_or("mcp_reviewer");

            let angle = crate::ontology::CertifiedAngle {
                id: format!("angle_{}", uuid::Uuid::new_v4().simple()),
                title: title.to_string(),
                angle_pitch: pitch.to_string(),
                target_pain_point: target_pain_point.to_string(),
                mandatory_disclosure: mandatory_disclosure.to_string(),
                certified_by: certified_by.to_string(),
                certified_at: chrono::Utc::now().to_rfc3339(),
                active: true,
            };

            if let Some(storage) = &self.storage {
                storage
                    .save_certified_angle(subreddit, &angle)
                    .map_err(|e| format!("Failed to save certified angle: {}", e))?;

                // Also update or create ontology in storage
                let mut ont = storage
                    .get_community_ontology(subreddit)
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| {
                        crate::ontology::AutoOntologyBuilder::build_from_threads(
                            subreddit,
                            &format!("Domain knowledge for r/{}", subreddit),
                            &[],
                        )
                    });
                ont.certified_angles.push(angle.clone());
                storage
                    .save_community_ontology(&ont)
                    .map_err(|e| format!("Failed to update community ontology: {}", e))?;
            }

            return Ok(json!(angle));
        }

        if name == "reddit_ontology_list" {
            if let Some(storage) = &self.storage {
                let subs = storage
                    .list_community_ontologies()
                    .map_err(|e| format!("Storage error: {}", e))?;
                return Ok(json!(subs));
            } else {
                return Ok(json!([]));
            }
        }

        let account_param = args["account"].as_str();
        let ctx = self
            .account_manager
            .get_account_context(account_param)
            .map_err(|e| format!("Failed to resolve account: {}", e))?;

        match name {
            "reddit_audit_community_ratio" => {
                let identity = self
                    .router
                    .execute(&Capability::GetIdentity, &ctx, json!({}))
                    .await
                    .map_err(|e| e.to_string())?;

                let link_karma = identity["link_karma"].as_i64().unwrap_or(0);
                let comment_karma = identity["comment_karma"].as_i64().unwrap_or(0);

                let input_records: Vec<crate::agents::community_builder::EngagementRecord> = args
                    ["history"]
                    .as_array()
                    .and_then(|arr| {
                        serde_json::from_value(serde_json::Value::Array(arr.clone())).ok()
                    })
                    .unwrap_or_else(|| {
                        let value_count = (comment_karma.max(0) / 10).clamp(1, 90) as usize;
                        let promo_count = (link_karma.max(0) / 50).clamp(0, 10) as usize;
                        let mut records = Vec::new();
                        for i in 0..value_count {
                            records.push(crate::agents::community_builder::EngagementRecord {
                                id: format!("val_{}", i),
                                subreddit: "community".to_string(),
                                is_promotional: false,
                                summary: "Organic value discussion".to_string(),
                                timestamp_utc: chrono::Utc::now().timestamp(),
                            });
                        }
                        for i in 0..promo_count {
                            records.push(crate::agents::community_builder::EngagementRecord {
                                id: format!("promo_{}", i),
                                subreddit: "community".to_string(),
                                is_promotional: true,
                                summary: "Promotional submission".to_string(),
                                timestamp_utc: chrono::Utc::now().timestamp(),
                            });
                        }
                        records
                    });

                let audit_res = CommunityBuilderWorker::audit_90_10_ratio(&input_records);
                Ok(json!({
                    "account": ctx.account.alias,
                    "reddit_username": ctx.account.reddit_username,
                    "link_karma": link_karma,
                    "comment_karma": comment_karma,
                    "audit": audit_res
                }))
            }
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
            storage.clone(),
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

        // Verify newly added opportunity and community audit tools are present in tools list
        assert!(tools_arr.iter().any(|t| t["name"] == "reddit_digger"));
        assert!(tools_arr.iter().any(|t| t["name"] == "reddit_miner"));
        assert!(tools_arr
            .iter()
            .any(|t| t["name"] == "reddit_discover_opportunities"));
        assert!(tools_arr
            .iter()
            .any(|t| t["name"] == "reddit_draft_response"));
        assert!(tools_arr
            .iter()
            .any(|t| t["name"] == "reddit_audit_community_ratio"));

        let sample_product = json!({
            "product_name": "RedditAgent",
            "value_prop": "Local-first multi-account Reddit operator",
            "icp_target": "SaaS Founders",
            "pain_points": ["spammy bots", "wasting hours"],
            "competitors": ["spam-blaster"],
            "target_subreddits": ["SaaS"],
            "high_intent_phrases": ["tool recommendation"],
            "affiliation_disclosure": "Full disclosure: I built RedditAgent"
        });

        // Call reddit_miner on sample source
        let sample_source = json!([{
            "id": "thread_123",
            "subreddit": "SaaS",
            "title": "Need tool recommendation for finding customers without spam",
            "author": "founder_x",
            "score": 15,
            "num_comments": 8,
            "created_utc": 1700000000,
            "permalink": "/r/SaaS/comments/thread_123/",
            "canonical_url": "https://reddit.com/r/SaaS/comments/thread_123/",
            "body_snippet": "Does anyone know a good alternative to spammy reddit scrapers?",
            "full_body": "Does anyone know a good alternative to spammy reddit scrapers?"
        }]);

        let miner_res = server
            .handle_tool_call(
                "reddit_miner",
                json!({
                    "sources": sample_source,
                    "product": sample_product
                }),
            )
            .await
            .unwrap();

        assert_eq!(miner_res["analyzed_count"], 1);
        assert_eq!(miner_res["opportunities"][0]["thread"]["id"], "thread_123");

        // Call reddit_draft_response tool without requiring an account
        let sample_opp = json!({
            "thread": {
                "id": "thread_123",
                "subreddit": "SaaS",
                "title": "Need tool recommendation for finding customers without spam",
                "body": "Does anyone know a good alternative to spammy reddit scrapers?",
                "author": "founder_x",
                "created_utc": 1700000000,
                "num_comments": 8,
                "is_locked": false,
                "is_archived": false
            },
            "op_signal_score": 0.95,
            "product_fit_score": 0.85,
            "timing_score": 0.90,
            "composite_score": 0.88,
            "detected_signals": ["RecommendationAsk", "UrgentWorkflow"],
            "opportunity_reason": "OP is asking for recommendation"
        });

        let draft_res = server
            .handle_tool_call(
                "reddit_draft_response",
                json!({
                    "opportunity": sample_opp,
                    "product": sample_product
                }),
            )
            .await
            .unwrap();

        assert_eq!(draft_res["thread_id"], "thread_123");
        assert_eq!(draft_res["experience_grammar_valid"], true);
        assert_eq!(draft_res["affiliation_disclosed"], true);
        assert_eq!(draft_res["product_named"], true);

        // Call reddit_audit_community_ratio
        let ratio_res = server
            .handle_tool_call(
                "reddit_audit_community_ratio",
                json!({ "account": "personal" }),
            )
            .await
            .unwrap();

        assert_eq!(ratio_res["account"], "personal");
        assert!(ratio_res["audit"]["is_compliant"].is_boolean());

        // Verify newly added stealth browser & keychain takeover tools are present
        assert!(tools_arr
            .iter()
            .any(|t| t["name"] == "reddit_account_login"));
        assert!(tools_arr
            .iter()
            .any(|t| t["name"] == "reddit_takeover_account"));
        assert!(tools_arr
            .iter()
            .any(|t| t["name"] == "reddit_account_cookies"));
        assert!(tools_arr.iter().any(|t| t["name"] == "reddit_human_browse"));

        // Call reddit_account_cookies when empty
        let cookie_res_empty = server
            .handle_tool_call("reddit_account_cookies", json!({ "account": "personal" }))
            .await
            .unwrap();
        assert_eq!(cookie_res_empty["has_cookies"], false);

        // Populate keychain cookies and test again
        let store = server.account_manager.auth().credential_store();
        let browser_driver =
            Arc::new(crate::drivers::browser::BrowserDriver::new().with_credential_store(store));
        let sample_jar = crate::drivers::browser::SessionCookieJar {
            alias: "personal".to_string(),
            reddit_username: Some("my_user".to_string()),
            captured_at: "2026-11-04T12:00:00Z".to_string(),
            cookie_count: 1,
            cookies: vec![crate::drivers::browser::SessionCookie {
                name: "reddit_session".to_string(),
                value: "session_tok".to_string(),
                domain: Some(".reddit.com".to_string()),
                path: Some("/".to_string()),
                secure: Some(true),
                http_only: Some(true),
                same_site: Some("Lax".to_string()),
                expires: None,
            }],
        };
        browser_driver
            .save_cookies_to_keychain("personal", &sample_jar)
            .unwrap();

        let server_with_browser = server.with_browser_driver(browser_driver);
        let cookie_res_saved = server_with_browser
            .handle_tool_call("reddit_account_cookies", json!({ "account": "personal" }))
            .await
            .unwrap();
        assert_eq!(cookie_res_saved["has_cookies"], true);
        assert_eq!(cookie_res_saved["cookie_count"], 1);
        assert_eq!(cookie_res_saved["verified_username"], "my_user");

        // Verify newly added auto-ontology tools
        assert!(tools_arr.iter().any(|t| t["name"] == "reddit_ontology_get"));
        assert!(tools_arr
            .iter()
            .any(|t| t["name"] == "reddit_ontology_check_engagable"));
        assert!(tools_arr
            .iter()
            .any(|t| t["name"] == "reddit_ontology_certify_angle"));
        assert!(tools_arr
            .iter()
            .any(|t| t["name"] == "reddit_ontology_list"));

        let server_with_storage = server_with_browser.with_storage(storage.clone());

        // Test reddit_ontology_certify_angle
        let angle_res = server_with_storage
            .handle_tool_call(
                "reddit_ontology_certify_angle",
                json!({
                    "subreddit": "vibecoding",
                    "title": "Local-First Architecture",
                    "pitch": "Use local sqlite and deterministic state",
                    "target_pain_point": "wasting hours",
                    "mandatory_disclosure": "Disclosure: I maintain reddit-agent",
                    "certified_by": "qa_lead"
                }),
            )
            .await
            .unwrap();
        assert_eq!(angle_res["title"], "Local-First Architecture");

        // Test reddit_ontology_get
        let ont_res = server_with_storage
            .handle_tool_call("reddit_ontology_get", json!({ "subreddit": "vibecoding" }))
            .await
            .unwrap();
        assert_eq!(ont_res["subreddit"], "vibecoding");
        assert_eq!(ont_res["certified_angles"].as_array().unwrap().len(), 1);

        // Test reddit_ontology_check_engagable (Safe)
        let check_safe = server_with_storage
            .handle_tool_call(
                "reddit_ontology_check_engagable",
                json!({
                    "subreddit": "vibecoding",
                    "title": "Tired of wasting hours with buggy cloud tools",
                    "body": "Need a reliable setup."
                }),
            )
            .await
            .unwrap();
        assert!(check_safe.get("CertifiedSafe").is_some());

        // Test reddit_ontology_check_engagable (Spam / Taboo Violation)
        let check_spam = server_with_storage
            .handle_tool_call(
                "reddit_ontology_check_engagable",
                json!({
                    "subreddit": "vibecoding",
                    "title": "Join our upvote for upvote group!",
                    "body": "Free upvotes!"
                }),
            )
            .await
            .unwrap();
        assert!(check_spam.get("HighRiskViolation").is_some());

        // Test reddit_ontology_list
        let list_res = server_with_storage
            .handle_tool_call("reddit_ontology_list", json!({}))
            .await
            .unwrap();
        let list_arr = list_res.as_array().unwrap();
        assert!(list_arr.iter().any(|s| s == "vibecoding"));
    }
}
