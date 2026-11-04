# Tool: reddit_get_subreddit

Fetch metadata, active subscriber statistics, description, and restrictions for a community.

## Purpose & Scope
Inspects a target subreddit's public parameters:
- Subscriber count and active concurrent users.
- Subreddit type: `public`, `restricted`, `private`, or `archived`.
- NSFW/18+ flag.
- Submission type allowed: `any`, `link`, `self`.
- Over-18 requirement.

Works via authenticated OAuth or public anonymous fallback driver.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `subreddit` | string | Yes | Target community name without the `r/` prefix. |
| `account` | string | No | Account alias (optional). |

## Return Schema
```json
{
  "name": "SaaS",
  "display_name": "r/SaaS",
  "title": "Software as a Service (SaaS)",
  "subscribers": 145000,
  "active_users": 280,
  "submission_type": "any",
  "over18": false,
  "is_quarantined": false,
  "created_utc": 1332800000,
  "description": "Discussion, strategies, and feedback for SaaS founders and builders."
}
```

## Agent Usage Pattern
Always inspect community statistics before evaluating campaign opportunities or creating submissions.

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "6",
  "method": "tools/call",
  "params": {
    "name": "reddit_get_subreddit",
    "arguments": {
      "subreddit": "SaaS"
    }
  }
}
```
