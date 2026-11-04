# reddit_account_cookies

Inspect metadata for Reddit session cookies securely stored in the OS Keychain without leaking raw cookie values.

## Overview
- **Keyless & Encrypted**: Interrogates the OS Keychain (`reddit-agent` service, `browser_cookies:{alias}`) to check session status.
- **Zero Token Leakage**: Returns metadata only (cookie count, verified username, timestamp, and cookie names like `reddit_session`, `token_v2`), never exposing raw values.
- **Fail-Closed Verification**: Allows agents and operators to confirm login health without spinning up a browser process.

---

## Tool Signature

```json
{
  "name": "reddit_account_cookies",
  "description": "Inspect metadata for account session cookies stored in OS Keychain (cookie count, verified username, timestamp) with zero raw token exposure.",
  "inputSchema": {
    "type": "object",
    "required": ["account"],
    "properties": {
      "account": {
        "type": "string",
        "description": "Account alias to inspect."
      }
    }
  }
}
```

---

## Example Usage

### Request
```json
{
  "account": "personal"
}
```

### Response
```json
{
  "account": "personal",
  "has_cookies": true,
  "verified_username": "founder_mamdouh",
  "captured_at": "2026-11-04T12:00:00Z",
  "cookie_count": 6,
  "keychain_storage": "Encrypted in OS Keychain ('reddit-agent')",
  "cookie_names": [
    "reddit_session",
    "token_v2",
    "loid",
    "session_tracker",
    "edgebucket",
    "recent_srs"
  ]
}
```
