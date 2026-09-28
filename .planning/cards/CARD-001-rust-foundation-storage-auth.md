# CARD-001: Rust Foundation, SQLite Storage, OS Keyring, and Multi-Account Auth

## Objective
Establish the primary Rust Cargo project, rusqlite database storage schema, OS keyring token isolation with safe memory caching, multi-account identity management, and OAuth authorization-code exchange with a localhost callback listener.

## Acceptance Criteria
1. Cargo project compiles cleanly with `tokio`, `rusqlite`, `keyring`, `oauth2`, `serde`, `thiserror`, `tracing`.
2. SQLite schema initializes tables: `accounts`, `oauth_metadata`, `browser_profiles`, `pending_actions`, `audit_log`.
3. Credentials stored in OS Keyring with zero tokens in SQLite, logs, or debug output.
4. Multi-account registry supports adding accounts, switching active account, listing accounts with status.
5. Unit tests verify:
   - Account context resolution and deterministic default selection.
   - Token redaction invariant: no tokens or secrets serialized to string or JSON.
   - Keyring isolation: account tokens do not cross account boundaries.
