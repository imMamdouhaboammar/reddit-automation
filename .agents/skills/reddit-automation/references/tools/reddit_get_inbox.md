# Tool: reddit_get_inbox

Fetch unread messages, post replies, and moderation notices from the Reddit inbox.

## Purpose & Scope
Monitors incoming interactions directed at the account:
- Direct messages (PMs).
- Comment replies and mentions (`u/username`).
- AutoModerator notifications or removal alerts.

Supports filter categories: `unread`, `messages`, `inbox`.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `account` | string | No | Account alias (defaults to active account). |
| `filter` | string | No | Message category: `"unread"`, `"messages"`, or `"inbox"`. Default: `"unread"`. |

## Return Schema
```json
{
  "account": "personal_dev",
  "unread_count": 2,
  "messages": [
    {
      "id": "m1a2b3",
      "author": "AutoModerator",
      "subject": "Your post in r/startups was approved",
      "body": "Your submission has been reviewed by the moderation team.",
      "created_utc": 1735002000,
      "was_comment": false
    },
    {
      "id": "m4c5d6",
      "author": "dev_interested",
      "subject": "comment reply: What tools do you use?",
      "body": "Thanks for the tip on memory profiling! Did you benchmark this on ARM64?",
      "created_utc": 1735004000,
      "was_comment": true
    }
  ]
}
```

## Agent Usage Pattern
Periodically invoke during active campaigns to detect questions, AutoMod removals, or moderator inquiries.

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "14",
  "method": "tools/call",
  "params": {
    "name": "reddit_get_inbox",
    "arguments": {
      "filter": "unread"
    }
  }
}
```
