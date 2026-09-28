# CARD-007: MCP Server Agent Surface, CLI Admin Surface, 3-Pass Verification

## Objective
Implement the MCP stdio server with all safe agent capabilities, the administrative CLI subcommands, end-to-end integration tests, and run 3-Pass Verification (Unit, Integration, Security/Taste).

## Acceptance Criteria
1. MCP stdio server exposes tools:
   - `reddit_accounts`: List configured account aliases with active indicator.
   - `reddit_whoami`: Query identity and karma for specified or active account.
   - `reddit_account_health`: Run deterministic health checks.
   - `reddit_get_preferences`: Read current Reddit preferences.
   - `reddit_prepare_preference_change`: Prepare reversible preference patch ActionPlan.
   - `reddit_get_subreddit`: Get community info and subscriber count.
   - `reddit_get_subreddit_rules`: Fetch subreddit rules.
   - `reddit_get_post_requirements`: Fetch posting requirements.
   - `reddit_search`: Search submissions.
   - `reddit_get_post`: Retrieve post details.
   - `reddit_get_comments`: Retrieve comment tree.
   - `reddit_prepare_post`: Validate and create ActionPlan for submission.
   - `reddit_publish_prepared_post`: Execute approved post ActionPlan.
   - `reddit_prepare_comment`: Validate and create ActionPlan for comment/reply.
   - `reddit_publish_prepared_comment`: Execute approved comment ActionPlan.
   - `reddit_edit_own_content`: Edit post/comment text.
   - `reddit_delete_own_content`: Delete post/comment.
   - `reddit_get_inbox`: Fetch unread messages.
   - `reddit_diagnose_issue`: Diagnose account/posting issue.
   - `reddit_plan_repair`: Generate repair plan.
   - `reddit_apply_safe_repair`: Execute safe automated repair.
2. CLI subcommands provide administrative control:
   - `account`: `add`, `login`, `oauth`, `health`, `list`, `use`, `whoami`.
   - `me`: Query identity.
   - `prefs`: `get`, `set`.
   - `subreddit`: `rules`, `requirements`.
   - `post`: `prepare`.
   - `action`: `list`, `approve`, `execute`.
   - `issue`: `diagnose`, `repair`.
   - `browser`: `open`, `settings`.
   - `mcp`: Launch MCP stdio server.
3. 3-Pass Verification:
   - Pass 1 (Unit): Full test coverage of domain models, hashing, auth, storage, router, issues.
   - Pass 2 (Integration): Mocked API roundtrips and CLI command dry runs.
   - Pass 3 (Security & Taste): Keyring isolation, zero secret leakage in logs/output, clean human-readable CLI outputs.
