# Tool: reddit_get_post_requirements

Fetch strict posting requirements (title regex, body length limits, required flair, blacklisted domains) for a subreddit.

## Purpose & Scope
Inspects Reddit's structured submission restrictions (`/api/v1/subreddit/post_requirements`). Prevents rejected posts by uncovering programmatic AutoMod criteria before submission:
- Minimum/maximum title length.
- Title regex patterns (e.g. `[Ask]`, `[Discussion]` tags).
- Required link or post flairs.
- Domain allowlists/blocklists.
- Mandatory body text constraints.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `subreddit` | string | Yes | Subreddit name without `r/` prefix. |
| `account` | string | No | Account alias (optional). |

## Return Schema
```json
{
  "subreddit": "webdev",
  "title_min_length": 15,
  "title_max_length": 300,
  "title_regex": "^\\[.+\\].*",
  "body_min_length": 50,
  "body_required": true,
  "flair_required": true,
  "available_flairs": [
    {
      "id": "flair_1a2b3c",
      "text": "Discussion",
      "text_editable": false
    },
    {
      "id": "flair_4d5e6f",
      "text": "Question",
      "text_editable": false
    }
  ],
  "domain_blacklist": ["bit.ly", "tinyurl.com"]
}
```

## Agent Usage Pattern
Run this tool before calling `reddit_prepare_post` to ensure the title, flair, and body length conform exactly to subreddit validation gates.

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "8",
  "method": "tools/call",
  "params": {
    "name": "reddit_get_post_requirements",
    "arguments": {
      "subreddit": "webdev"
    }
  }
}
```
