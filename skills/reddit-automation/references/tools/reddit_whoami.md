# Tool: reddit_whoami

Query identity, karma breakdown, and verified Reddit username for a given account (or the active default account).

## Purpose & Scope
Returns the live Reddit profile associated with an account configuration. Verifies OAuth connectivity with Reddit's `/api/v1/me` endpoint, providing current comment karma, link karma, account age, and moderation status.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `account` | string | No | Optional account alias. If omitted, the default account is queried. |

## Return Schema
```json
{
  "alias": "personal_dev",
  "username": "CodeExplorer",
  "comment_karma": 1420,
  "link_karma": 380,
  "total_karma": 1800,
  "created_utc": 1609459200,
  "is_gold": false,
  "is_mod": false,
  "has_verified_email": true
}
```

## Agent Usage Pattern
Use when starting a session to confirm identity and establish baseline karma before computing engagement ratios or planning community submissions.

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "2",
  "method": "tools/call",
  "params": {
    "name": "reddit_whoami",
    "arguments": {
      "account": "personal_dev"
    }
  }
}
```

## Safety Rules
- Read-only call.
- If Reddit returns a 401 Unauthorized, routes automatically to `reddit_diagnose_issue`.
