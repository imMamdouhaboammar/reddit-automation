# Tool: reddit_get_comments

Fetch the comment hierarchy and top replies for a specific submission.

## Purpose & Scope
Returns the nested discussion tree of a Reddit post up to depth limit. Enables agents to:
- Identify top-upvoted advice already given by the community.
- Avoid duplicate suggestions.
- Discover direct questions from the OP within the comment thread.
- Find parent comment IDs (`t1_...`) for drafting targeted replies.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `post_id` | string | Yes | Post ID or fullname (`1h3xyz` or `t3_1h3xyz`). |
| `subreddit` | string | No | Subreddit name (optional). |
| `account` | string | No | Account alias (optional). |

## Return Schema
```json
{
  "post_id": "1h3xyz",
  "total_comments": 14,
  "comments": [
    {
      "id": "c1a2b3",
      "name": "t1_c1a2b3",
      "author": "experienced_cto",
      "body": "Have you considered OpenTelemetry collectors?",
      "score": 19,
      "created_utc": 1735003600,
      "replies": [
        {
          "id": "c4d5e6",
          "name": "t1_c4d5e6",
          "author": "founder_99",
          "body": "Yes, but our team found the configuration overhead high.",
          "score": 3,
          "created_utc": 1735005000,
          "replies": []
        }
      ]
    }
  ]
}
```

## Agent Usage Pattern
Fetch existing comments before drafting a reply. Check if the OP has clarified their problem in the replies (`c4d5e6`).

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "11",
  "method": "tools/call",
  "params": {
    "name": "reddit_get_comments",
    "arguments": {
      "post_id": "1h3xyz",
      "subreddit": "SaaS"
    }
  }
}
```
