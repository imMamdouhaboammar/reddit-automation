# Reddit Operator End-to-End Behavioral Findings & Defect Journal

This document tracks all defects, anomalies, and behavioral failures discovered during skeptical, adversarial, end-to-end testing of `reddit-agent`.

---

## Finding BUG-001: Interactive Browser Login Reports False Success on Timeout
- **ID:** BUG-001
- **Severity:** P1
- **Scenario:** `reddit-agent account login <alias> --timeout-secs 5` without logging in.
- **Expected:** The command must fail with an authentication timeout error and a non-zero exit code.
- **Actual:** The command printed `Login complete! Verified username: u/logged_in_user` and exited with status 0.
- **Reproduction:**
  ```bash
  reddit-agent account add test-acc
  reddit-agent account login test-acc --timeout-secs 5
  echo $? # observed: 0, stdout: "Login complete! Verified username: u/logged_in_user"
  ```
- **Root Cause:** In `src/drivers/browser.rs`, `interactive_login` defaulted `user_display` to `"logged_in_user"` when `verified_user` was `None`, and returned `Ok(user_display)`.
- **Fix:** If `verified_user` is `None` after `start.elapsed() >= timeout_duration`, return `Err(DriverError::HumanCheckpointRequired("Interactive login timed out before user authentication was completed".to_string()))`.
- **Regression Test:** `tests/regression_tests.rs::test_interactive_login_timeout_fails_closed`
- **Real-World Verification:** Tested with 500ms timeout; returned `Err(HumanCheckpointRequired)` and exited with code 1.
- **Status:** Resolved / Verified

---

## Finding BUG-002: ActionPlan Hash Did Not Cryptographically Bind Account and Target
- **ID:** BUG-002
- **Severity:** P0
- **Scenario:** Modifying `account_id`, `action_type`, or `target` in `pending_actions` SQLite table before executing.
- **Expected:** Modifying the target or account of an approved action must fail closed with `PayloadTampered`.
- **Actual:** Only `payload_json` was hashed. Modifying `target` or `account_id` was ignored by the tamper check.
- **Reproduction:**
  ```bash
  OUTPUT=$(reddit-agent post prepare -s test -t "Title" --body "Body")
  ACTION_ID=$(echo "$OUTPUT" | grep "Action ID:" | awk '{print $3}')
  reddit-agent action approve "$ACTION_ID"
  sqlite3 ~/.config/reddit-agent/state.db "UPDATE pending_actions SET target='r/tampered' WHERE id='$ACTION_ID';"
  reddit-agent action execute "$ACTION_ID" # hash check previously passed!
  ```
- **Root Cause:** `compute_payload_hash` in `src/actions.rs` only hashed `payload_json`. Furthermore, `save_pending_action` in `src/storage.rs` omitted `target`, `account_id`, `action_type`, and `driver` from its `ON CONFLICT DO UPDATE` clause.
- **Fix:** Added `compute_action_hash(account_id, action, target, payload)` which cryptographically binds all 4 parameters via SHA-256. Updated `prepare_action` and `execute_action` to verify this comprehensive signature. Updated SQLite `save_pending_action` to include all mutable fields in `ON CONFLICT`.
- **Regression Test:** `tests/regression_tests.rs::test_action_tampering_target_or_account_fails_closed`
- **Real-World Verification:** Tampered target in SQLite rejected with `Error: Payload tampered! Hash mismatch`.
- **Status:** Resolved / Verified

---

## Finding BUG-003: MCP Server Bypasses Approval Gate for Posts and Comments
- **ID:** BUG-003
- **Severity:** P0
- **Scenario:** Calling `reddit_publish_prepared_post` or `reddit_publish_prepared_comment` via MCP without prior approval.
- **Expected:** Unapproved actions cannot be published. MCP clients must explicitly approve actions.
- **Actual:** `reddit_publish_prepared_post` and `reddit_publish_prepared_comment` automatically approved unapproved actions (`self.action_manager.approve_action(action_id)`), subverting the human-in-the-loop approval requirement.
- **Root Cause:** In `src/mcp.rs`, `handle_tool_call` auto-approved actions internally, and MCP lacked a `reddit_approve_action` tool.
- **Fix:** Removed auto-approval from publish tools. Added `reddit_approve_action` to the MCP tool catalog. Now unapproved plans return `Action requires approval before execution`.
- **Regression Test:** `tests/regression_tests.rs::test_mcp_unapproved_publish_rejected`
- **Real-World Verification:** Calling publish on pending action via MCP now fails closed with `Action requires approval before execution`.
- **Status:** Resolved / Verified

---

## Finding BUG-004: Post Preparation Accepted Invalid Titles, Kinds, URLs, and Subreddit Names
- **ID:** BUG-004
- **Severity:** P2
- **Scenario:** Preparing posts with empty/whitespace titles, titles > 300 chars, invalid kinds, missing/invalid URLs, or invalid subreddit names.
- **Expected:** Validation errors should reject invalid submissions during `post prepare`.
- **Actual:** All invalid submissions created ActionPlans and exited with status 0.
- **Reproduction:**
  ```bash
  reddit-agent post prepare -s test -t "" --body "Hello" # Exit 0!
  reddit-agent post prepare -s test -t "Link" --kind link # Exit 0 without URL!
  reddit-agent post prepare -s "invalid sub!" -t "Title" # Exit 0!
  ```
- **Root Cause:** `src/cli.rs` and `src/actions.rs` lacked intrinsic post field validation before plan creation.
- **Fix:** Added `ActionManager::validate_post_submission` checking:
  1. Subreddit name format (3-21 chars, alphanumeric + underscores only)
  2. Title (non-empty trimmed, 1..=300 chars)
  3. Kind (must be `"self"` or `"link"`)
  4. URL (required for `"link"`, must parse as valid HTTP/HTTPS)
  Integrated validation into both CLI `PostSubcommand::Prepare` and MCP `reddit_prepare_post`.
- **Regression Test:** `tests/regression_tests.rs::test_post_prepare_validation_boundaries`, `test_mcp_prepare_post_input_validation`
- **Real-World Verification:** Executing with empty title, invalid subreddit, or link without URL exits 1 with actionable errors.
- **Status:** Resolved / Verified

---

## Finding BUG-005: Reversible Actions Could Be Re-Executed Indefinitely; Executed Actions Returned Misleading Error
- **ID:** BUG-005
- **Severity:** P1
- **Scenario:** Calling `action execute <id>` on an already-executed action plan.
- **Expected:** Fail with `InvalidState("Action has already been executed")`.
- **Actual:** For reversible actions (`RiskLevel::Reversible`), execution ran again indefinitely because `requires_approval()` was false and `status == Executed` was not checked. For approved actions, it returned `ApprovalRequired` instead of `InvalidState`.
- **Root Cause:** `execute_action` in `src/actions.rs` did not check `plan.status == ActionStatus::Executed`.
- **Fix:** Added explicit check: `if plan.status == ActionStatus::Executed { return Err(ActionError::InvalidState("Action has already been executed".to_string())); }`. Also check `plan.status == ActionStatus::Rejected`.
- **Regression Test:** `tests/regression_tests.rs::test_executed_action_cannot_be_reexecuted`
- **Real-World Verification:** Verified in regression suite; returns `InvalidState("Action has already been executed")`.
- **Status:** Resolved / Verified

---

## Finding BUG-006: CLI `account oauth` Ran with Missing/Default Client ID and Hung
- **ID:** BUG-006
- **Severity:** P2
- **Scenario:** Running `reddit-agent account oauth <alias>` without `--client-id` or `REDDIT_CLIENT_ID`.
- **Expected:** Actionable error instructing the user to configure `REDDIT_CLIENT_ID` or `--client-id`.
- **Actual:** Opened browser to `client_id=default_client_id` (failing with "YOU BROKE REDDIT" on Reddit) and hung for 120s on localhost listener.
- **Root Cause:** `src/cli.rs` did not validate that `self.client_id` was set and not equal to `"default_client_id"` before launching the OAuth flow.
- **Fix:** Added validation in `AccountSubcommand::Oauth`: returns `anyhow::bail!("Reddit client ID is required for OAuth flow. Provide --client-id <ID> or set REDDIT_CLIENT_ID (create an app at https://www.reddit.com/prefs/apps).")`.
- **Real-World Verification:** Running `reddit-agent account oauth test-acc` immediately exits with code 1 and prints the exact remediation instructions.
- **Status:** Resolved / Verified

---

## Finding BUG-007: State Storage and Browser Profiles Ignored `XDG_CONFIG_HOME` and `REDDIT_AGENT_HOME`
- **ID:** BUG-007
- **Severity:** P2
- **Scenario:** Running `reddit-agent` with `XDG_CONFIG_HOME` or `REDDIT_AGENT_HOME` set.
- **Expected:** Database and browser profiles should be placed in the overridden directory to allow isolated clean-room testing.
- **Actual:** Hardcoded to `$HOME/.config/reddit-agent`.
- **Root Cause:** `Storage::default_path` and `BrowserDriver::profile_dir_for_account` only inspected `$HOME`.
- **Fix:** Added `Storage::config_dir()` checking `REDDIT_AGENT_HOME`, then `XDG_CONFIG_HOME/reddit-agent`, falling back to `$HOME/.config/reddit-agent`.
- **Regression Test:** `tests/regression_tests.rs::test_storage_and_profile_paths_respect_xdg_and_reddit_home`
- **Real-World Verification:** Verified in tests and CLI runs using `REDDIT_AGENT_HOME`.
- **Status:** Resolved / Verified

---

## Finding BUG-008: `prefs set` Silently Dropped Arguments Without `=`
- **ID:** BUG-008
- **Severity:** P3
- **Scenario:** `reddit-agent prefs set invalid_arg` without an equals sign.
- **Expected:** Clear error indicating settings must be formatted as `key=value`.
- **Actual:** Silently ignored, resulting in an empty patch `{}` and preparing an empty ActionPlan.
- **Root Cause:** `src/cli.rs` ignored non-matching strings in `for s in settings`.
- **Fix:** Validates that each argument contains `key=value` format with non-empty key, and errors if empty.
- **Real-World Verification:** `reddit-agent prefs set invalid_setting` now exits 1 with: `Error: Invalid preference format 'invalid_setting': Must be in key=value format (e.g. enable_followers=true).`
- **Status:** Resolved / Verified

---

## Finding BUG-009: `action execute` Lacked Pre-Execution Confirmation and Ignored `--account` Mismatch
- **ID:** BUG-009
- **Severity:** P2
- **Scenario:** Running `reddit-agent --account account_b action execute <id_of_account_a>`.
- **Expected:** Execution must reject if the explicitly requested account context does not match the action plan's bound account. Execution must also print the target account alias, Reddit username, and action preview before executing.
- **Actual:** Ignored `--account` override silently; printed only action ID.
- **Root Cause:** `src/cli.rs` did not validate `--account` override against `plan.account` and did not display action context before executing.
- **Fix:** Validates account match and prints execution preview (account alias, Reddit user, action, target) before proceeding.
- **Real-World Verification:** Displays `=== EXECUTING ACTION ===` header with account details and rejects account mismatches.
- **Status:** Resolved / Verified
