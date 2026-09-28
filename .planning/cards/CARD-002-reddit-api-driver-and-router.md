# CARD-002: Reddit API Driver, Capability Enum, and Capability Router

## Objective
Implement the `Capability` enum, `Driver` trait (`supports`, `execute`, `health_check`), `CapabilityRouter`, and `RedditApiDriver` using a persistent `reqwest::Client` with rustls, User-Agent enforcement, and rate limit header tracking.

## Acceptance Criteria
1. Capability contract defines: `GetIdentity`, `GetPreferences`, `SetPreferences`, `GetSubreddit`, `GetSubredditRules`, `Search`, `CreatePost`, `EditPost`, `DeletePost`, `CreateComment`, `EditComment`, `DeleteComment`, `GetInbox`, `ManageSubscription`, `ManageSaved`, `ManageHidden`, `InspectAccountHealth`, `BrowserSettings`.
2. `RedditApiDriver` implements endpoints: `/api/v1/me`, `/api/v1/me/karma`, `/api/v1/scopes`, `/api/v1/me/prefs` (GET & PATCH).
3. `CapabilityRouter` routes API capabilities to `RedditApiDriver` and browser-only capabilities to `BrowserDriver`.
4. Rate limit headers (`x-ratelimit-remaining`, `x-ratelimit-reset`) are recorded and respected.
5. Unit tests with mocked HTTP responses verify:
   - Identity fetching and JSON deserialization.
   - Preference reading and schema validation.
   - Header redaction for Authorization headers in tracing/logs.
