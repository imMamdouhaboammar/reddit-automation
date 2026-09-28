# CARD-005: BrowserDriver with Chromiumoxide, Isolated Profile Per-Account, Interactive Login, Session Verification

## Objective
Implement `BrowserDriver` via `chromiumoxide` to handle UI-only operations and human interactive checkpoints with strictly isolated per-account profile directories.

## Acceptance Criteria
1. Profile isolation: Profiles stored at `~/.config/reddit-agent/browser/<account_alias>/` — zero cookie sharing between accounts.
2. Interactive login: `reddit-agent account login <account_alias>` opens a visible Chromium window to Reddit, halts for human authentication, verifies logged-in username against the account record, and stores profile path in SQLite.
3. Lazy initialization: Chromium is only launched on demand for UI-only actions, never for API-supported actions.
4. Session verification: Checks if Reddit UI displays the authenticated username without requesting passwords or stealing cookies.
5. UI navigation & settings: Safe navigation to Reddit settings and support/appeal pages, stopping at security checkpoints (password change, 2FA, identity verification) for human intervention.
6. Error handling: UI breakage returns `BrowserCapabilityBroken` with structured diagnostics rather than blind retries.
