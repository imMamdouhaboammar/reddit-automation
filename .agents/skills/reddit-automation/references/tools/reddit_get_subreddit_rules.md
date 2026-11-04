# Tool: reddit_get_subreddit_rules

Retrieve official community rules and guidelines for a subreddit.

## Purpose & Scope
Fetches the formal moderation rules defined in `r/<subname>/about/rules`. This includes rule titles, detailed violation criteria, short descriptions, and report reasons. Essential for pre-flight compliance checks before proposing any post or response.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `subreddit` | string | Yes | Subreddit name without `r/` prefix. |
| `account` | string | No | Account alias (optional). |

## Return Schema
```json
{
  "subreddit": "startups",
  "rules": [
    {
      "priority": 1,
      "short_name": "No direct self-promotion in main feed",
      "description": "Keep self-promotional pitches, links, and launches inside the designated weekly Share Your Startup megathread.",
      "violation_reason": "Self-promotion outside sticky",
      "created_utc": 1500000000
    },
    {
      "priority": 2,
      "short_name": "Provide context and actionable insights",
      "description": "Vague questions or requests for ideas without prior research are removed.",
      "violation_reason": "Low-effort post",
      "created_utc": 1500000000
    }
  ],
  "site_rules_flow": true
}
```

## Agent Usage Pattern
Whenever drafting a post or comment, check rules first to avoid immediate AutoModerator shadowbans.

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "7",
  "method": "tools/call",
  "params": {
    "name": "reddit_get_subreddit_rules",
    "arguments": {
      "subreddit": "startups"
    }
  }
}
```
