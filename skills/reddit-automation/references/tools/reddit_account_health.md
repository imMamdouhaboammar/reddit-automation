# Tool: reddit_account_health

Run deterministic health diagnostics on an account (OAuth tokens, scopes, rate-limit state, browser session freshness).

## Purpose & Scope
Performs deep pre-flight verification on a Reddit account before launching autonomous posting workflows. Tests:
1. Token expiration timestamp.
2. Required OAuth scopes (`identity`, `read`, `submit`, `privatemessages`, `edit`).
3. Reddit rate-limit remaining budget.
4. Cookie/browser session freshness if Playwright fallback driver is enabled.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `account` | string | Yes | Account alias to check. |

## Return Schema
```json
{
  "account": "personal_dev",
  "status": "healthy",
  "oauth": {
    "valid": true,
    "expires_in_seconds": 3200,
    "granted_scopes": ["identity", "read", "submit", "edit"]
  },
  "rate_limits": {
    "remaining_requests": 590,
    "reset_seconds": 580
  },
  "browser_session": {
    "available": false,
    "state": "not_configured"
  },
  "warnings": []
}
```

## Agent Usage Pattern
Run prior to any batch or scheduled autonomous task. If `status` is `"degraded"` or `"error"`, pass the result directly into `reddit_plan_repair`.

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "3",
  "method": "tools/call",
  "params": {
    "name": "reddit_account_health",
    "arguments": {
      "account": "personal_dev"
    }
  }
}
```

## Safety Rules
- Read-only diagnostics.
- Non-destructive.
