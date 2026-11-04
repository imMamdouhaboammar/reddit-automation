# Tool: reddit_get_post

Fetch complete submission details and body content for a specific post.

## Purpose & Scope
Retrieves single post object metadata, body markdown, score, upvote ratio, author, flair, lock status, and moderation state. Accepts standard base36 IDs (`1h3xyz`) or fullnames (`t3_1h3xyz`).

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `post_id` | string | Yes | Post ID or fullname (e.g. `1h3xyz` or `t3_1h3xyz`). |
| `subreddit` | string | No | Subreddit name (speeds up lookup if known). |
| `account` | string | No | Account alias (optional). |

## Return Schema
```json
{
  "id": "1h3xyz",
  "name": "t3_1h3xyz",
  "subreddit": "SaaS",
  "title": "What tools do you use for customer telemetry?",
  "selftext": "We are hitting scaling limits with our current analytics stack...",
  "author": "founder_99",
  "score": 48,
  "upvote_ratio": 0.94,
  "num_comments": 23,
  "permalink": "/r/SaaS/comments/1h3xyz/what_tools_do_you_use_for_customer_telemetry/",
  "canonical_url": "https://www.reddit.com/r/SaaS/comments/1h3xyz/what_tools_do_you_use_for_customer_telemetry/",
  "is_locked": false,
  "is_archived": false,
  "created_utc": 1735000000
}
```

## Agent Usage Pattern
Run this when you need the complete un-truncated text of an identified opportunity post before formulating an answer.

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "10",
  "method": "tools/call",
  "params": {
    "name": "reddit_get_post",
    "arguments": {
      "post_id": "1h3xyz",
      "subreddit": "SaaS"
    }
  }
}
```
