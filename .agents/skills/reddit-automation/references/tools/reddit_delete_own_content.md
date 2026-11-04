# Tool: reddit_delete_own_content

Permanently delete your own post or comment on Reddit.

## Purpose & Scope
Removes a submission or comment published by the active account. Once deleted, the text is replaced by `[deleted]` on Reddit.

High-risk destructive action.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Fullname of the content to delete (e.g. `t3_xxx` or `t1_xxx`). |
| `account` | string | No | Account alias that authored the content. |

## Return Schema
```json
{
  "id": "t1_c1a2b3",
  "status": "deleted",
  "deleted_utc": 1735009000
}
```

## Agent Usage Pattern
Use when a comment or post violates subreddit rules, receives negative community sentiment, or was posted in error.

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "13",
  "method": "tools/call",
  "params": {
    "name": "reddit_delete_own_content",
    "arguments": {
      "id": "t1_c1a2b3",
      "account": "personal_dev"
    }
  }
}
```

## Safety Rules
- Irreversible destructive operation.
- The agent must always verify author identity matches active account before invocation.
