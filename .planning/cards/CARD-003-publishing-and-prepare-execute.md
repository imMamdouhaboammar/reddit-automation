# CARD-003: Content Publishing, Subreddit Rules, Post Requirements, 2-Stage Prepare/Execute Model

## Objective
Implement Subreddit metadata, rules, and post requirements queries, together with a two-stage ActionPlan preparation and execution model with risk classifications and SHA-256 payload tampering validation.

## Acceptance Criteria
1. Subreddit queries: `about.json`, `rules.json`, and `/api/v1/{subreddit}/post_requirements`.
2. Action risk classification: `READ`, `REVERSIBLE`, `PUBLIC_WRITE`, `DESTRUCTIVE`, `SECURITY_SENSITIVE`.
3. Side-effect preparation: `prepare_post` and `prepare_comment` produce structured `ActionPlan` with SHA-256 hash.
4. Preview enforcement: Previews clearly show `Account`, `Reddit user`, `Action`, `Subreddit`, and payload.
5. Execution verification: `execute` validates that the account, action, and payload hash match the approved plan.
6. Execution calls Reddit endpoints: `/api/submit` (self/link), `/api/comment`, `/api/editusertext`, `/api/del`, `/api/save`, `/api/unsave`.
7. Unit tests verify:
   - ActionPlan payload hash verification fails closed if tampered.
   - Approval gate prevents unapproved execution.
   - Audit event is logged with account alias, reddit username, action, and target.
