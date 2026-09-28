# CARD-004: Account Control, Subscriptions, Saved/Hidden, Inbox, and Deterministic Account Health

## Objective
Implement account preferences management (fetching and patching supported preference keys), subreddit subscriptions, saved/hidden item management, inbox & unread notifications fetching, and deterministic `AccountHealth` evaluation.

## Acceptance Criteria
1. Preferences management: GET `/api/v1/me/prefs` and PATCH `/api/v1/me/prefs` with schema validation of allowed keys.
2. Subscription management: GET `/subreddits/mine/subscriber` and POST `/api/subscribe` (`sub`/`unsub`).
3. Saved and hidden content: `/api/save`, `/api/unsave`, `/api/hide`, `/api/unhide`.
4. Inbox reading: `/message/inbox.json`, `/message/unread.json`, and `/api/read_message`.
5. Deterministic `AccountHealth`:
   - Checks OAuth token validity & expiration timestamp.
   - Checks required scopes presence against requested capability.
   - Checks API endpoint reachability.
   - Checks rate-limit status (remaining requests and reset delay).
   - Checks browser profile session status.
   - Returns structured `AccountHealthReport` with status (`healthy`, `degraded`, `unhealthy`) and actionable issue codes.
