use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::accounts::AccountManager;
use crate::actions::{ActionManager, RiskLevel};
use crate::agents::opportunity::{OpportunityWorker, ProductContext, RankedOpportunity, RedditThread};
use crate::audit::AuditLogger;
use crate::auth::AuthManager;
use crate::drivers::browser::BrowserDriver;
use crate::drivers::reddit_api::RedditApiDriver;
use crate::drivers::reddit_keyless::RedditKeylessDriver;
use crate::drivers::Capability;
use crate::issues::IssueEngine;
use crate::mcp::McpServer;
use crate::router::CapabilityRouter;
use crate::storage::Storage;

#[derive(Parser)]
#[command(name = "reddit-agent")]
#[command(
    about = "Fast, local-first, multi-account Agentic Reddit Operator in Rust",
    version = "0.1.0"
)]
pub struct Cli {
    #[arg(long, global = true, help = "Account alias to use")]
    pub account: Option<String>,

    #[arg(
        long,
        global = true,
        help = "Reddit client ID (can also be read from REDDIT_CLIENT_ID)"
    )]
    pub client_id: Option<String>,

    #[arg(
        long,
        global = true,
        help = "Reddit client secret (can also be read from REDDIT_CLIENT_SECRET)"
    )]
    pub client_secret: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage isolated Reddit accounts and authentication
    Account {
        #[command(subcommand)]
        subcommand: AccountSubcommand,
    },
    /// Inspect authenticated identity for active account
    Me,
    /// Read and modify supported account preferences
    Prefs {
        #[command(subcommand)]
        subcommand: PrefsSubcommand,
    },
    /// Query community rules and posting requirements
    Subreddit {
        #[command(subcommand)]
        subcommand: SubredditSubcommand,
    },
    /// Prepare and publish Reddit posts
    Post {
        #[command(subcommand)]
        subcommand: PostSubcommand,
    },
    /// Manage, review, and execute two-stage ActionPlans
    Action {
        #[command(subcommand)]
        subcommand: ActionSubcommand,
    },
    /// Diagnose and repair account or submission issues
    Issue {
        #[command(subcommand)]
        subcommand: IssueSubcommand,
    },
    /// Controlled browser operations
    Browser {
        #[command(subcommand)]
        subcommand: BrowserSubcommand,
    },
    /// Discover high-intent Reddit opportunities and draft helpful replies
    Opportunity {
        #[command(subcommand)]
        subcommand: OpportunitySubcommand,
    },
    /// Mine Reddit deep consensus to forge a production-grade Agent Skill (OmniSkill v6.2.0)
    #[command(name = "reddit-2-skill", alias = "r2s")]
    Reddit2Skill {
        #[arg(
            long,
            short,
            help = "Topic or problem context to research on Reddit (e.g. 'vibe coding errors')"
        )]
        topic: String,
        #[arg(long, help = "Comma-separated subreddits to prioritize")]
        subreddits: Option<String>,
        #[arg(
            long,
            default_value = "skills",
            help = "Base directory to write generated skill folder"
        )]
        output_dir: PathBuf,
    },
    /// Start Model Context Protocol (MCP) server over Stdio
    Mcp,
}

#[derive(Subcommand)]
pub enum OpportunitySubcommand {
    /// Discover high-intent opportunities from a subreddit
    Discover {
        #[arg(long, short, help = "Target subreddit (e.g. SaaS)")]
        subreddit: String,
        #[arg(long, default_value = "MyProduct", help = "Product or tool name")]
        product: String,
        #[arg(long, default_value = "Automates workflow with local-first security")]
        value_prop: String,
        #[arg(long, default_value = "10")]
        limit: usize,
        #[arg(long, default_value = "text", help = "Output format: text, json, md")]
        format: String,
        #[arg(long, help = "Optional file path to write output artifact")]
        output: Option<PathBuf>,
    },
    /// Draft an experience-grammar reply adhering to honest disclosure
    Draft {
        #[arg(long, help = "Thread title or question asked")]
        title: String,
        #[arg(long, default_value = "")]
        body: String,
        #[arg(long, default_value = "MyProduct")]
        product: String,
        #[arg(long, default_value = "Full disclosure: I work on MyProduct")]
        disclosure: String,
    },
}

#[derive(Subcommand)]
pub enum AccountSubcommand {
    /// Register a new account alias
    Add { alias: String },
    /// List all configured accounts
    List,
    /// Set the default active account
    Use { alias: String },
    /// Run health check diagnostics on an account
    Health { alias: Option<String> },
    /// Run OAuth2 interactive authorization flow
    Oauth {
        alias: String,
        #[arg(long, default_value = "8080")]
        port: u16,
        #[arg(long, help = "Space or comma-separated OAuth scopes")]
        scopes: Option<String>,
    },
    /// Open visible Chrome window for manual login
    Login {
        alias: String,
        #[arg(long, default_value = "180")]
        timeout_secs: u64,
    },
    /// Show identity for account
    Whoami { alias: Option<String> },
}

#[derive(Subcommand)]
pub enum PrefsSubcommand {
    /// Fetch current account preferences
    Get,
    /// Set supported preferences (key=value ...)
    Set {
        #[arg(required = true)]
        settings: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum SubredditSubcommand {
    /// Inspect official rules of a subreddit
    Rules { subreddit: String },
    /// Inspect post requirements (flair, title/body constraints)
    Requirements { subreddit: String },
}

#[derive(Subcommand)]
pub enum PostSubcommand {
    /// Prepare a submission ActionPlan with validation
    Prepare {
        #[arg(long, short)]
        subreddit: String,
        #[arg(long, short)]
        title: String,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        body_file: Option<PathBuf>,
        #[arg(long)]
        url: Option<String>,
        #[arg(long, default_value = "self")]
        kind: String,
        #[arg(long)]
        flair_id: Option<String>,
        #[arg(long, default_value_t = false)]
        nsfw: bool,
        #[arg(long, default_value_t = false)]
        spoiler: bool,
    },
}

#[derive(Subcommand)]
pub enum ActionSubcommand {
    /// List pending action plans
    List,
    /// Approve a pending action plan
    Approve { id: String },
    /// Execute an approved action plan
    Execute { id: String },
}

#[derive(Subcommand)]
pub enum IssueSubcommand {
    /// Run diagnosis on account state or errors
    Diagnose,
    /// Apply safe repair remediation
    Repair {
        #[arg(long)]
        issue_code: String,
    },
}

#[derive(Subcommand)]
pub enum BrowserSubcommand {
    /// Open browser to Reddit
    Open,
    /// Open browser to Reddit settings section
    Settings {
        #[arg(long, default_value = "feed")]
        section: String,
    },
}

pub struct AppContext {
    pub storage: Arc<Storage>,
    pub auth_manager: Arc<AuthManager>,
    pub account_manager: Arc<AccountManager>,
    pub audit_logger: Arc<AuditLogger>,
    pub router: Arc<CapabilityRouter>,
    pub action_manager: Arc<ActionManager>,
    pub issue_engine: Arc<IssueEngine>,
    pub browser_driver: Arc<BrowserDriver>,
    pub client_id: String,
    pub client_secret: String,
}

impl AppContext {
    pub fn init(client_id_opt: Option<String>, client_secret_opt: Option<String>) -> Self {
        let storage = Arc::new(
            Storage::open_default()
                .unwrap_or_else(|_| Storage::new_in_memory().expect("in-memory db fallback")),
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        let auth_manager = Arc::new(AuthManager::new_system(client.clone()));
        let account_manager = Arc::new(AccountManager::new(storage.clone(), auth_manager.clone()));
        let audit_logger = Arc::new(AuditLogger::new(storage.clone()));

        let client_id = client_id_opt
            .or_else(|| std::env::var("REDDIT_CLIENT_ID").ok())
            .unwrap_or_else(|| "default_client_id".to_string());

        let client_secret = client_secret_opt
            .or_else(|| std::env::var("REDDIT_CLIENT_SECRET").ok())
            .unwrap_or_default();

        let api_driver = Arc::new(RedditApiDriver::new(
            client,
            auth_manager.clone(),
            storage.clone(),
            client_id.clone(),
            client_secret.clone(),
        ));

        let browser_driver = Arc::new(BrowserDriver::new().with_storage(storage.clone()));
        let router = Arc::new(CapabilityRouter::new(api_driver, browser_driver.clone()));

        let action_manager = Arc::new(ActionManager::new(
            storage.clone(),
            router.clone(),
            account_manager.clone(),
            audit_logger.clone(),
        ));

        let issue_engine = Arc::new(IssueEngine::new(
            router.clone(),
            account_manager.clone(),
            action_manager.clone(),
            storage.clone(),
            client_id.clone(),
        ));

        Self {
            storage,
            auth_manager,
            account_manager,
            audit_logger,
            router,
            action_manager,
            issue_engine,
            browser_driver,
            client_id,
            client_secret,
        }
    }

    pub async fn run(self, cli: Cli) -> anyhow::Result<()> {
        let account_override = cli.account.as_deref();

        match cli.command {
            Commands::Account { subcommand } => match subcommand {
                AccountSubcommand::Add { alias } => {
                    let acc = self.account_manager.add_account(&alias)?;
                    println!(
                        "Added account '{}' (id: {}). Active: {}",
                        acc.alias, acc.id, acc.is_active
                    );
                }

                AccountSubcommand::List => {
                    let accounts = self.account_manager.list_accounts()?;
                    if accounts.is_empty() {
                        println!(
                            "No accounts registered yet. Use `reddit-agent account add <alias>`."
                        );
                    } else {
                        println!(
                            "{:<15} {:<20} {:<8} {:<10}",
                            "ALIAS", "REDDIT USER", "ACTIVE", "DRIVER"
                        );
                        println!("{:-<55}", "");
                        for a in accounts {
                            let user = a.reddit_username.as_deref().unwrap_or("<unverified>");
                            let active_str = if a.is_active { "YES" } else { "no" };
                            println!(
                                "{:<15} {:<20} {:<8} {:<10}",
                                a.alias, user, active_str, a.default_driver
                            );
                        }
                    }
                }

                AccountSubcommand::Use { alias } => {
                    self.account_manager.set_active_account(&alias)?;
                    println!("Default active account switched to '{}'", alias);
                }

                AccountSubcommand::Health { alias } => {
                    let ctx = self
                        .account_manager
                        .get_account_context(alias.as_deref().or(account_override))?;
                    let report = self.router.health_check(&ctx).await?;
                    println!("Account Health Report for '{}':", report.account);
                    println!("  Status: {}", report.status.to_uppercase());
                    if let Some(ref u) = report.reddit_username {
                        println!("  Reddit User: u/{}", u);
                    }
                    if report.issues.is_empty() {
                        println!("  No issues detected. System is healthy.");
                    } else {
                        println!("  Issues ({}) :", report.issues.len());
                        for issue in report.issues {
                            println!(
                                "    - [{}] {}: {}",
                                issue.capability, issue.code, issue.message
                            );
                            println!("      Remediation: {}", issue.remediation);
                        }
                    }
                }

                AccountSubcommand::Oauth {
                    alias,
                    port,
                    scopes,
                } => {
                    let ctx = self.account_manager.get_account_context(Some(&alias))?;

                    if self.client_id.trim().is_empty() || self.client_id == "default_client_id" {
                        anyhow::bail!(
                            "Reddit client ID is required for OAuth flow. Provide --client-id <ID> or set REDDIT_CLIENT_ID (create an app at https://www.reddit.com/prefs/apps)."
                        );
                    }

                    let scope_list = scopes.as_deref().unwrap_or("identity read submit edit account mysubreddits subscribe save history privatemessages");
                    let scope_vec: Vec<&str> = scope_list
                        .split(|c: char| c.is_whitespace() || c == ',')
                        .filter(|s| !s.is_empty())
                        .collect();
                    let redirect_uri = format!("http://127.0.0.1:{}/callback", port);
                    let state = uuid::Uuid::new_v4().to_string();

                    let auth_url = AuthManager::generate_authorize_url(
                        &self.client_id,
                        &redirect_uri,
                        &state,
                        &scope_vec,
                    );

                    println!("Opening Reddit authorization in browser...");
                    println!("Authorization URL:\n{}\n", auth_url);

                    // Attempt opening browser
                    #[cfg(target_os = "macos")]
                    let _ = std::process::Command::new("open").arg(&auth_url).spawn();
                    #[cfg(target_os = "linux")]
                    let _ = std::process::Command::new("xdg-open")
                        .arg(&auth_url)
                        .spawn();

                    println!(
                        "Waiting for OAuth callback on http://127.0.0.1:{}/callback ...",
                        port
                    );
                    let code =
                        AuthManager::wait_for_callback(port, &state, Duration::from_secs(120))
                            .await?;
                    println!("Callback received! Exchanging code for tokens...");

                    let tokens = self
                        .auth_manager
                        .exchange_code(&self.client_id, &self.client_secret, &code, &redirect_uri)
                        .await?;

                    if let Some(ref refresh) = tokens.refresh_token {
                        self.auth_manager.store_refresh_token(&alias, refresh)?;
                        println!("Securely stored refresh token in OS Keychain!");
                    }

                    // Query identity to link username
                    self.auth_manager
                        .set_cached_access_token(&alias, &tokens.access_token, tokens.expires_in)
                        .await;
                    if let Ok(id_val) = self
                        .router
                        .execute(&Capability::GetIdentity, &ctx, json!({}))
                        .await
                    {
                        if let Some(username) = id_val["identity"]["name"].as_str() {
                            let scopes_saved: Vec<String> =
                                scope_vec.into_iter().map(String::from).collect();
                            self.account_manager.link_identity(
                                &ctx.account.id,
                                username,
                                &scopes_saved,
                            )?;
                            println!(
                                "Linked account '{}' to Reddit username 'u/{}'!",
                                alias, username
                            );
                        }
                    }
                }

                AccountSubcommand::Login {
                    alias,
                    timeout_secs,
                } => {
                    let ctx = self.account_manager.get_account_context(Some(&alias))?;
                    println!("Launching visible Chrome browser for manual login...");
                    println!(
                        "Please sign in on the Reddit window. Waiting up to {} seconds...",
                        timeout_secs
                    );
                    let user = self
                        .browser_driver
                        .interactive_login(&ctx, Duration::from_secs(timeout_secs))
                        .await?;
                    println!("Login complete! Verified username: u/{}", user);
                }

                AccountSubcommand::Whoami { alias } => {
                    let ctx = self
                        .account_manager
                        .get_account_context(alias.as_deref().or(account_override))?;
                    let val = self
                        .router
                        .execute(&Capability::GetIdentity, &ctx, json!({}))
                        .await?;
                    println!("{}", serde_json::to_string_pretty(&val)?);
                }
            },

            Commands::Me => {
                let ctx = self.account_manager.get_account_context(account_override)?;
                let val = self
                    .router
                    .execute(&Capability::GetIdentity, &ctx, json!({}))
                    .await?;
                let identity = &val["identity"];
                println!("Account: {}", ctx.account.alias);
                println!(
                    "Reddit User: u/{}",
                    identity["name"].as_str().unwrap_or("unknown")
                );
                println!("Link Karma: {}", identity["link_karma"]);
                println!("Comment Karma: {}", identity["comment_karma"]);
                println!("Is Moderator: {}", identity["is_mod"]);
            }

            Commands::Prefs { subcommand } => match subcommand {
                PrefsSubcommand::Get => {
                    let ctx = self.account_manager.get_account_context(account_override)?;
                    let val = self
                        .router
                        .execute(&Capability::GetPreferences, &ctx, json!({}))
                        .await?;
                    println!("{}", serde_json::to_string_pretty(&val)?);
                }

                PrefsSubcommand::Set { settings } => {
                    let ctx = self.account_manager.get_account_context(account_override)?;
                    let mut patch_map = serde_json::Map::new();

                    for s in settings {
                        if let Some((k, v)) = s.split_once('=') {
                            let k = k.trim();
                            let v = v.trim();
                            if k.is_empty() {
                                anyhow::bail!(
                                    "Invalid preference key in '{}': Key cannot be empty.",
                                    s
                                );
                            }
                            let parsed_v = match v.to_lowercase().as_str() {
                                "true" => Value::Bool(true),
                                "false" => Value::Bool(false),
                                _ => Value::String(v.to_string()),
                            };
                            patch_map.insert(k.to_string(), parsed_v);
                        } else {
                            anyhow::bail!(
                                "Invalid preference format '{}': Must be in key=value format (e.g. enable_followers=true).",
                                s
                            );
                        }
                    }

                    if patch_map.is_empty() {
                        anyhow::bail!("No preferences specified to update.");
                    }

                    let patch_val = Value::Object(patch_map);
                    let plan = self.action_manager.prepare_action(
                        &ctx,
                        "set_preferences",
                        "account_preferences",
                        patch_val,
                        RiskLevel::Reversible,
                        vec![],
                        30,
                    )?;

                    println!("ActionPlan prepared (ID: {})", plan.id);
                    println!(
                        "{}",
                        CapabilityRouter::preview_action(
                            &ctx,
                            "Patch preferences",
                            "prefs",
                            &serde_json::to_string(&plan.payload_preview)?
                        )
                    );
                    println!(
                        "\nTo execute this change, run:\n  reddit-agent action execute {}",
                        plan.id
                    );
                }
            },

            Commands::Subreddit { subcommand } => match subcommand {
                SubredditSubcommand::Rules { subreddit } => {
                    let ctx = self.account_manager.get_account_context(account_override)?;
                    let rules = self
                        .router
                        .execute(
                            &Capability::GetSubredditRules,
                            &ctx,
                            json!({"subreddit": subreddit}),
                        )
                        .await?;
                    println!("{}", serde_json::to_string_pretty(&rules)?);
                }

                SubredditSubcommand::Requirements { subreddit } => {
                    let ctx = self.account_manager.get_account_context(account_override)?;
                    let reqs = self
                        .router
                        .execute(
                            &Capability::GetPostRequirements,
                            &ctx,
                            json!({"subreddit": subreddit}),
                        )
                        .await?;
                    println!("{}", serde_json::to_string_pretty(&reqs)?);
                }
            },

            Commands::Post { subcommand } => match subcommand {
                PostSubcommand::Prepare {
                    subreddit,
                    title,
                    body,
                    body_file,
                    url,
                    kind,
                    flair_id,
                    nsfw,
                    spoiler,
                } => {
                    let ctx = self.account_manager.get_account_context(account_override)?;
                    let text = if let Some(path) = body_file {
                        std::fs::read_to_string(path)?
                    } else {
                        body.unwrap_or_default()
                    };

                    ActionManager::validate_post_submission(
                        &subreddit,
                        &title,
                        &kind,
                        url.as_deref(),
                    )?;

                    let mut warnings = Vec::new();
                    if let Ok(reqs_val) = self
                        .router
                        .execute(
                            &Capability::GetPostRequirements,
                            &ctx,
                            json!({"subreddit": subreddit}),
                        )
                        .await
                    {
                        if let Ok(reqs) =
                            serde_json::from_value::<crate::reddit::PostRequirements>(reqs_val)
                        {
                            warnings = ActionManager::validate_post_requirements(
                                &reqs,
                                &title,
                                &text,
                                flair_id.as_deref(),
                            );
                        }
                    }

                    let payload = json!({
                        "subreddit": subreddit,
                        "title": title,
                        "text": text,
                        "url": url,
                        "kind": kind,
                        "flair_id": flair_id,
                        "nsfw": nsfw,
                        "spoiler": spoiler,
                    });

                    let plan = self.action_manager.prepare_action(
                        &ctx,
                        "create_post",
                        &format!("r/{}", subreddit),
                        payload,
                        RiskLevel::PublicWrite,
                        warnings.clone(),
                        60,
                    )?;

                    println!("=== ACTION PLAN PREPARED ===");
                    println!(
                        "{}",
                        CapabilityRouter::preview_action(
                            &ctx,
                            "Create post",
                            &format!("r/{}", subreddit),
                            &format!("Title: {}", title)
                        )
                    );
                    println!("Action ID:    {}", plan.id);
                    println!("Payload Hash: {}", plan.payload_hash);
                    println!("Risk Level:   {:?}", plan.risk_level);
                    println!("Expires At:   {}", plan.expires_at);

                    if !warnings.is_empty() {
                        println!("\nWARNINGS (Post Requirements Violations):");
                        for w in warnings {
                            println!("  - {}", w);
                        }
                    }

                    println!("\nNext Steps:");
                    println!("  1. Approve: `reddit-agent action approve {}`", plan.id);
                    println!("  2. Execute: `reddit-agent action execute {}`", plan.id);
                }
            },

            Commands::Action { subcommand } => match subcommand {
                ActionSubcommand::List => {
                    let actions = self.storage.list_pending_actions(None)?;
                    if actions.is_empty() {
                        println!("No pending actions.");
                    } else {
                        println!(
                            "{:<15} {:<15} {:<15} {:<12} {:<10}",
                            "ID", "ACTION", "TARGET", "STATUS", "RISK"
                        );
                        println!("{:-<70}", "");
                        for a in actions {
                            println!(
                                "{:<15} {:<15} {:<15} {:<12} {:<10}",
                                a.id, a.action_type, a.target, a.status, a.risk_level
                            );
                        }
                    }
                }

                ActionSubcommand::Approve { id } => {
                    let plan = self.action_manager.approve_action(&id)?;
                    println!(
                        "ActionPlan '{}' approved successfully! Status: {:?}",
                        plan.id, plan.status
                    );
                    println!(
                        "Run `reddit-agent action execute {}` to perform the operation.",
                        id
                    );
                }

                ActionSubcommand::Execute { id } => {
                    let plan = self.action_manager.get_action(&id)?;
                    if let Some(ref req_acc) = account_override {
                        if req_acc != &plan.account {
                            anyhow::bail!(
                                "Account context mismatch: Action '{}' is bound to account '{}', but '--account {}' was specified.",
                                id,
                                plan.account,
                                req_acc
                            );
                        }
                    }
                    println!("=== EXECUTING ACTION ===");
                    println!("Account:  {}", plan.account);
                    if let Some(ref u) = plan.reddit_username {
                        println!("User:     u/{}", u);
                    }
                    println!("Action:   {}", plan.action);
                    println!("Target:   {}", plan.target);
                    println!(
                        "Validating cryptographic hash and executing action '{}'...",
                        id
                    );
                    let res = self.action_manager.execute_action(&id).await?;
                    println!("Executed successfully!");
                    println!("{}", serde_json::to_string_pretty(&res)?);
                }
            },

            Commands::Issue { subcommand } => match subcommand {
                IssueSubcommand::Diagnose => {
                    let ctx = self.account_manager.get_account_context(account_override)?;
                    let diag = self.issue_engine.diagnose_account(&ctx).await?;
                    println!("Diagnosis for account '{}':", diag.account);
                    println!("  Category: {:?}", diag.category);
                    println!("  Code:     {}", diag.code);
                    println!("  Summary:  {}", diag.summary);
                    println!("  Human Action Mandatory: {}", diag.human_action_mandatory);
                    println!("  Remediation Code:       {}", diag.remediation_code);

                    let plan = self.issue_engine.plan_repair(&diag)?;
                    println!("\nRemediation Plan:");
                    println!("  Action: {}", plan.repair_action);
                    println!("  Details: {}", plan.remediation);
                    println!("  Safe Automatic: {}", plan.is_safe_automatic);
                }

                IssueSubcommand::Repair { issue_code } => {
                    let ctx = self.account_manager.get_account_context(account_override)?;
                    let diag = self.issue_engine.diagnose_account(&ctx).await?;
                    if diag.code != issue_code && diag.remediation_code != issue_code {
                        println!(
                            "Current diagnosed issue ({}) does not match requested code '{}'",
                            diag.code, issue_code
                        );
                    }
                    let plan = self.issue_engine.plan_repair(&diag)?;
                    let res = self.issue_engine.apply_safe_repair(&ctx, &plan).await?;
                    println!("Repair Status: {}", res.status);
                    println!("Details:       {}", res.details);
                }
            },

            Commands::Browser { subcommand } => match subcommand {
                BrowserSubcommand::Open => {
                    let ctx = self.account_manager.get_account_context(account_override)?;
                    println!("Launching isolated browser for '{}'...", ctx.account.alias);
                    let browser = self
                        .browser_driver
                        .launch_isolated_browser(&ctx.account.alias, false)
                        .await?;
                    let _page = browser.new_page("https://www.reddit.com").await?;
                    println!("Browser opened. Press Ctrl+C to close.");
                    tokio::signal::ctrl_c().await?;
                }

                BrowserSubcommand::Settings { section } => {
                    let ctx = self.account_manager.get_account_context(account_override)?;
                    let url = self.browser_driver.open_settings(&ctx, &section).await?;
                    println!("Navigated to {}", url);
                }
            },

            Commands::Opportunity { subcommand } => match subcommand {
                OpportunitySubcommand::Discover {
                    subreddit,
                    product,
                    value_prop,
                    limit,
                    format,
                    output,
                } => {
                    let product_ctx = ProductContext {
                        product_name: product,
                        value_prop,
                        icp_target: "Reddit community members seeking solutions".to_string(),
                        pain_points: vec![
                            "wasting time".to_string(),
                            "manual work".to_string(),
                            "flaky tools".to_string(),
                        ],
                        competitors: vec!["generic-tool".to_string()],
                        target_subreddits: vec![subreddit.clone()],
                        high_intent_phrases: vec![
                            "recommendation".to_string(),
                            "alternative".to_string(),
                            "what tool".to_string(),
                        ],
                        affiliation_disclosure: "Full disclosure: I am affiliated with this tool"
                            .to_string(),
                    };

                    let keyless = RedditKeylessDriver::new();
                    let threads = keyless
                        .discover_threads(
                            &[subreddit.clone()],
                            &product_ctx.high_intent_phrases,
                            limit,
                        )
                        .await;

                    let now = chrono::Utc::now().timestamp();
                    let ranked = OpportunityWorker::discover_opportunities(
                        &threads,
                        &product_ctx,
                        now,
                    );

                    let formatted_output = match format.to_lowercase().as_str() {
                        "json" => serde_json::to_string_pretty(&ranked)?,
                        "md" | "markdown" => {
                            let mut md = String::new();
                            md.push_str(&format!("# Reddit Opportunities: r/{}\n\n", subreddit));
                            md.push_str(&format!(
                                "*Found {} ranked opportunities for `{}`*\n\n",
                                ranked.len(),
                                product_ctx.product_name
                            ));
                            md.push_str("| # | Title | Subreddit | Score | Signals | Link |\n");
                            md.push_str("|---|---|---|---|---|---|\n");
                            for (i, opp) in ranked.iter().take(20).enumerate() {
                                let clean_title = opp.thread.title.replace('|', "\\|");
                                let signals_str = opp
                                    .detected_signals
                                    .iter()
                                    .map(|s| format!("{:?}", s))
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                md.push_str(&format!(
                                    "| {} | {} | r/{} | {:.2} | {} | [View Thread]({}) |\n",
                                    i + 1,
                                    clean_title,
                                    opp.thread.subreddit,
                                    opp.composite_score,
                                    signals_str,
                                    opp.thread.canonical_url()
                                ));
                            }
                            md.push_str("\n## Detailed Breakdown\n\n");
                            for (i, opp) in ranked.iter().take(10).enumerate() {
                                md.push_str(&format!("### {}. {}\n\n", i + 1, opp.thread.title));
                                md.push_str(&format!("- **URL:** {}\n", opp.thread.canonical_url()));
                                md.push_str(&format!(
                                    "- **Opportunity Reason:** {}\n",
                                    opp.opportunity_reason
                                ));
                                md.push_str(&format!(
                                    "- **Comments:** {} | **Composite Score:** {:.2}\n\n",
                                    opp.thread.num_comments, opp.composite_score
                                ));
                            }
                            md
                        }
                        _ => {
                            let mut text =
                                format!("\nFound {} ranked opportunities:\n\n", ranked.len());
                            for (i, opp) in ranked.iter().take(10).enumerate() {
                                text.push_str(&format!(
                                    "{}. [Score: {:.2}] {} (r/{})\n   URL:     {}\n   Why:     {}\n   Signals: {:?} | Comments: {}\n\n",
                                    i + 1,
                                    opp.composite_score,
                                    opp.thread.title,
                                    opp.thread.subreddit,
                                    opp.thread.canonical_url(),
                                    opp.opportunity_reason,
                                    opp.detected_signals,
                                    opp.thread.num_comments
                                ));
                            }
                            text
                        }
                    };

                    if let Some(out_path) = output {
                        std::fs::write(&out_path, &formatted_output)?;
                        println!("Artifact written successfully to: {}", out_path.display());
                    } else {
                        println!("{}", formatted_output);
                    }
                }

                OpportunitySubcommand::Draft {
                    title,
                    body,
                    product,
                    disclosure,
                } => {
                    let sample_thread = RedditThread {
                        id: "cli_draft_target".to_string(),
                        subreddit: "community".to_string(),
                        title: title.clone(),
                        body: body.clone(),
                        author: "peer_user".to_string(),
                        created_utc: chrono::Utc::now().timestamp(),
                        num_comments: 5,
                        is_locked: false,
                        is_archived: false,
                        url: "https://reddit.com/r/community/comments/cli_draft_target/".to_string(),
                    };

                    let sample_product = ProductContext {
                        product_name: product,
                        value_prop: "Helpful developer tool".to_string(),
                        icp_target: "Developers and community builders".to_string(),
                        pain_points: vec!["friction".to_string(), "manual work".to_string()],
                        competitors: vec![],
                        target_subreddits: vec!["community".to_string()],
                        high_intent_phrases: vec!["recommendation".to_string()],
                        affiliation_disclosure: disclosure,
                    };

                    let sample_opp = RankedOpportunity {
                        thread: sample_thread,
                        op_signal_score: 0.90,
                        product_fit_score: 0.80,
                        timing_score: 0.85,
                        composite_score: 0.85,
                        detected_signals: vec![crate::agents::opportunity::OpportunitySignal::RecommendationAsk],
                        opportunity_reason: "OP is looking for assistance".to_string(),
                    };

                    match OpportunityWorker::draft_reply(&sample_opp, &sample_product) {
                        Ok(draft) => {
                            println!("\n=== Experience Grammar Draft (Words: {}) ===", draft.word_count);
                            println!("{}", draft.draft_text);
                            println!("\nChecks passed:");
                            println!("- Experience grammar valid: {}", draft.experience_grammar_valid);
                            println!("- Product named:           {}", draft.product_named);
                            println!("- Affiliation disclosed:   {}", draft.affiliation_disclosed);
                            println!("- Referenced detail:       '{}'", draft.concrete_detail_referenced);
                        }
                        Err(err) => {
                            eprintln!("Draft generation failed: {}", err);
                        }
                    }
                }
            },

            Commands::Reddit2Skill {
                topic,
                subreddits,
                output_dir,
            } => {
                println!("🚀 Launching reddit-2-skill for topic: '{}'...", topic);
                let custom_subs: Option<Vec<String>> = subreddits.map(|s| {
                    s.split(',')
                        .map(|item| item.trim().trim_start_matches("r/").to_string())
                        .filter(|item| !item.is_empty())
                        .collect()
                });

                let engine = crate::agents::reddit_2_skill::Reddit2SkillEngine::new();
                let package = engine
                    .mine_and_forge_skill(&topic, custom_subs, Some(&output_dir))
                    .await
                    .map_err(|e| anyhow::anyhow!(e))?;

                println!("\n✅ Successfully forged Agent Skill: '{}'", package.skill_name);
                println!("- Display Name:       {}", package.display_name);
                println!("- Target Subreddits:  r/{}", package.target_subreddits.join(", r/"));
                println!("- Analyzed Threads:   {}", package.analyzed_threads_count);
                println!(
                    "- Output Directory:   {}",
                    output_dir.join(&package.skill_name).display()
                );
                println!("\nStructure created:");
                println!("  ├── SKILL.md                 (AgentSkills.io v1 & Poteto Mode standard)");
                println!("  ├── scripts/preflight_check.py");
                println!("  └── references/reddit_evidence.md");
            }

            Commands::Mcp => {
                let server = McpServer::new(
                    self.account_manager,
                    self.router,
                    self.action_manager,
                    self.issue_engine,
                );
                server.run_stdio_server().await?;
            }
        }

        Ok(())
    }
}
