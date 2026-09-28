# CARD-006: Agent Issue Diagnosis and Safe Deterministic Repair Engine

## Objective
Implement agent-facing issue diagnosis and deterministic repair capabilities (`reddit_diagnose_issue`, `reddit_plan_repair`, `reddit_apply_safe_repair`) handling OAuth, scopes, session expiration, post rejection/removal, and security challenges.

## Acceptance Criteria
1. Diagnostic engine identifies deterministic issue codes:
   - `OAUTH_REVOKED`: OAuth refresh token rejected; remediation plans reauthorization.
   - `MISSING_SCOPE`: Operation requires scope not currently granted; plans re-consent flow.
   - `BROWSER_SESSION_EXPIRED`: Browser session unauthenticated; plans interactive login.
   - `PREFERENCE_DRIFT`: Current preference differs from desired target; plans API PATCH.
   - `POST_REJECTED`: API rejection or rules mismatch; returns evidence and correction requirements.
   - `POST_REMOVED`: Post removed by AutoMod/moderator; extracts removal reason.
   - `RATE_LIMITED`: Calculates reset duration and plans safe backoff retry.
   - `SECURITY_CHALLENGE`: Account locked or identity challenge detected; halts and creates human checkpoint.
2. `reddit_plan_repair` creates structured `RepairPlan` with estimated risk level and execution parameters.
3. `reddit_apply_safe_repair` executes safe, reversible remediations automatically; requires human checkpoint for security-sensitive flows.
4. Unit tests verify:
   - Accurate issue code mapping from mock errors and responses.
   - Security challenges halt automated execution and flag mandatory human action.
