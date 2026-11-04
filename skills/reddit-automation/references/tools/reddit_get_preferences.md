# Tool: reddit_get_preferences

Read current Reddit account settings and preferences.

## Purpose & Scope
Retrieves Reddit user preferences via `/api/v1/me/prefs`. Useful for checking NSFW visibility flags, default comment sorting, email notification preferences, and search indexation settings.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `account` | string | No | Account alias (defaults to active account). |

## Return Schema
```json
{
  "account": "personal_dev",
  "preferences": {
    "over_18": true,
    "search_include_over_18": true,
    "default_comment_sort": "confidence",
    "email_digests": false,
    "show_presence": false
  }
}
```

## Agent Usage Pattern
Call when troubleshooting why certain subreddits (e.g. 18+ or private) cannot be searched or accessed.

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "4",
  "method": "tools/call",
  "params": {
    "name": "reddit_get_preferences",
    "arguments": {
      "account": "personal_dev"
    }
  }
}
```
