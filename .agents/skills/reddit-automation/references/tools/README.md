# Reddit Automation Tools Catalog

Complete technical reference for all 28 agentic MCP tools provided by the `reddit-automation` engine.

---

## Tool Categories

### 1. Opportunity Discovery & Community Growth (Keyless Supported)
These tools operate with keyless scraping fallbacks (Public Shreddit + Arctic Shift archive) and require zero authentication credentials.

| Tool | Description | Documentation |
|------|-------------|---------------|
| `reddit_digger` | Deep candidate source gatherer: maps where to search & aggregates threads | [reddit_digger.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_digger.md) |
| `reddit_miner` | Deep content analyzer: sanitizes text, classifies intent signals & ranks opportunities | [reddit_miner.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_miner.md) |
| `reddit_discover_opportunities` | Unified pipeline: runs digger then miner in one call (legacy alias) | [reddit_discover_opportunities.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_discover_opportunities.md) |
| `reddit_draft_response` | Draft 25-55 word Experience Grammar replies with mandatory disclosure | [reddit_draft_response.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_draft_response.md) |
| `reddit_audit_community_ratio` | Audit karma & history against Reddit's 90/10 content-to-promo rule | [reddit_audit_community_ratio.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_audit_community_ratio.md) |
| `reddit_2_skill` | Mine Reddit consensus & engineer production Agent Skill packages | [reddit_2_skill.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_2_skill.md) |

---

### 2. Verified Publishing Pipeline (Two-Stage ActionPlan)
All mutating operations (post, comment, preference updates) enforce a two-stage cryptographic plan with SHA-256 payload verification. Direct blind publishing is strictly blocked.

| Tool | Description | Documentation |
|------|-------------|---------------|
| `reddit_prepare_post` | Validate & prepare post ActionPlan with SHA-256 hash | [reddit_prepare_post.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_prepare_post.md) |
| `reddit_approve_action` | Explicitly approve pending ActionPlan | [reddit_approve_action.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_approve_action.md) |
| `reddit_publish_prepared_post` | Publish verified and approved post ActionPlan | [reddit_publish_prepared_post.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_publish_prepared_post.md) |
| `reddit_prepare_comment` | Validate & prepare comment ActionPlan with hash | [reddit_prepare_comment.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_prepare_comment.md) |
| `reddit_publish_prepared_comment` | Publish verified and approved comment ActionPlan | [reddit_publish_prepared_comment.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_publish_prepared_comment.md) |

---

### 3. Account & Identity Management
Manage multi-account profiles, OAuth validity, rate limits, and Reddit user preferences.

| Tool | Description | Documentation |
|------|-------------|---------------|
| `reddit_accounts` | List configured account aliases and active default | [reddit_accounts.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_accounts.md) |
| `reddit_whoami` | Live verified Reddit profile, karma breakdown & identity | [reddit_whoami.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_whoami.md) |
| `reddit_account_health` | Deep deterministic token, scope & rate-limit health check | [reddit_account_health.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_account_health.md) |
| `reddit_get_preferences` | Read account settings and NSFW/search preferences | [reddit_get_preferences.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_get_preferences.md) |
| `reddit_prepare_preference_change` | Generate two-stage ActionPlan to patch preferences | [reddit_prepare_preference_change.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_prepare_preference_change.md) |

---

### 4. Content & Community Exploration
Read-only queries for subreddit metadata, AutoMod post rules, and discussion trees.

| Tool | Description | Documentation |
|------|-------------|---------------|
| `reddit_get_subreddit` | Subscriber statistics, active users & community metadata | [reddit_get_subreddit.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_get_subreddit.md) |
| `reddit_get_subreddit_rules` | Subreddit rules, descriptions & violation reasons | [reddit_get_subreddit_rules.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_get_subreddit_rules.md) |
| `reddit_get_post_requirements` | Strict AutoMod submission rules (title regex, body length, flairs) | [reddit_get_post_requirements.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_get_post_requirements.md) |
| `reddit_search` | Search submissions across Reddit or target subreddits | [reddit_search.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_search.md) |
| `reddit_get_post` | Fetch complete submission details and body | [reddit_get_post.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_get_post.md) |
| `reddit_get_comments` | Fetch nested discussion comments tree for a post | [reddit_get_comments.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_get_comments.md) |

---

### 5. Content Lifecycle & Inbox Management
Edit, delete, and monitor ongoing interactions on published content.

| Tool | Description | Documentation |
|------|-------------|---------------|
| `reddit_edit_own_content` | Edit self-text or comment body of owned content | [reddit_edit_own_content.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_edit_own_content.md) |
| `reddit_delete_own_content` | Permanently delete post or comment | [reddit_delete_own_content.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_delete_own_content.md) |
| `reddit_get_inbox` | Fetch unread messages, mentions & comment replies | [reddit_get_inbox.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_get_inbox.md) |

---

### 6. Diagnostics & Automated Repair
Deterministic troubleshooting engine for failure isolation and automated remediation.

| Tool | Description | Documentation |
|------|-------------|---------------|
| `reddit_diagnose_issue` | Root-cause analysis on API or submission failures | [reddit_diagnose_issue.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_diagnose_issue.md) |
| `reddit_plan_repair` | Generate step-by-step repair plan with automated/human classification | [reddit_plan_repair.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_plan_repair.md) |
| `reddit_apply_safe_repair` | Safely execute non-destructive repair actions (backoff, token refresh) | [reddit_apply_safe_repair.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_apply_safe_repair.md) |

---

### 7. Dynamic Routing & Multi-Agent Orchestration (OmniSkill v6.2.0)
Autonomous intent classification, execution DAG generation, and wave-by-wave Tool-Agent coordination.

| Tool | Description | Documentation |
|------|-------------|---------------|
| `reddit_agent_route` | Classifies natural language (AR/EN) and compiles execution DAG | [reddit_agent_route.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_agent_route.md) |
| `reddit_agent_orchestrate` | Executes goal wave-by-wave across specialized Tool-Agents | [reddit_agent_orchestrate.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_agent_orchestrate.md) |

---

### 8. Stealth Browser Engine & Account Takeover (Obscura & agent-browser)
Anti-detection stealth browsing, encrypted OS Keychain session persistence, human hesitation, momentum scrolling, and compact semantic snapshots.

| Tool | Description | Documentation |
|------|-------------|---------------|
| `reddit_account_login` | Interactive stealth login & takeover; saves cookies to OS Keychain | [reddit_account_login.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_account_login.md) |
| `reddit_takeover_account` | Alias for account takeover with custom starting URL | [reddit_account_login.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_account_login.md) |
| `reddit_account_cookies` | Inspect OS Keychain session metadata (zero raw token leakage) | [reddit_account_cookies.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_account_cookies.md) |
| `reddit_human_browse` | Stealth browsing with human behavior simulation & semantic snapshots | [reddit_human_browse.md](file:///Users/mamdouhaboammar/Documents/M-Research/reddit-automation%20/skills/reddit-automation/references/tools/reddit_human_browse.md) |
