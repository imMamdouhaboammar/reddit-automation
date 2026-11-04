# reddit_account_login / reddit_takeover_account

Interactive user account login and takeover using the Obscura anti-detection stealth browser engine.

## Overview
- **Engine**: Obscura stealth browser (masks `navigator.webdriver`, spoofs `window.chrome`, randomizes Canvas/WebGL renderer to Apple Metal M-Series, applies anti-bot launch flags).
- **Session Persistence**: Encrypts and saves session cookies directly into the OS Keychain (`reddit-agent` service, `browser_cookies:{alias}` account).
- **Session Continuity**: Pre-injects existing Keychain cookies before page navigation to automatically resume logged-in sessions without human friction.
- **Fail-Closed Boundary**: Pauses at human checkpoints for password, 2FA, or passkey challenges; halts automation if challenges are detected.

---

## Tool Signature

```json
{
  "name": "reddit_account_login",
  "description": "Interactive Reddit account login & takeover using Obscura stealth browser. Opens visible anti-detection window, restores existing Keychain session if present, waits for authentication, and securely stores session cookies encrypted in OS Keychain.",
  "inputSchema": {
    "type": "object",
    "required": ["account"],
    "properties": {
      "account": {
        "type": "string",
        "description": "Configured account alias to log in."
      },
      "url": {
        "type": "string",
        "description": "Optional starting URL (defaults to https://www.reddit.com/login)."
      },
      "timeout_secs": {
        "type": "integer",
        "description": "Timeout in seconds to wait for user authentication (default: 180)."
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
  "account": "personal",
  "timeout_secs": 180
}
```

### Response
```json
{
  "account": "personal",
  "verified_username": "founder_mamdouh",
  "cookies_captured": 8,
  "keychain_saved": true,
  "profile_path": "/Users/.../.config/reddit-agent/browser/personal",
  "initial_snapshot": {
    "title": "Reddit - Dive into anything",
    "url": "https://www.reddit.com/",
    "element_count": 42,
    "tree_text": "[Page: Reddit - Dive into anything] (URL: https://www.reddit.com/)\n  - @e1 [LINK] \"Home\" -> /r/all\n  - @e2 [INPUT] \"Search Reddit\"...",
    "elements": [...]
  }
}
```

---

## Security Guarantees
- Raw session cookie tokens are never printed to terminal or returned in plain text MCP responses.
- Encrypted strictly via OS Keychain (`keyring::Entry("reddit-agent", "browser_cookies:<alias>")`).
