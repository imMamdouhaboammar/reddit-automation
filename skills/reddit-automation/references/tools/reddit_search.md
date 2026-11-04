# Tool: reddit_search

Search Reddit submissions within a target subreddit or across all of Reddit.

## Purpose & Scope
Executes keyword or query searches against Reddit. Automatically leverages keyless multi-tier fallback (public Shreddit search -> Arctic Shift archive) if no authenticated account is active or if OAuth quotas are constrained. Returns full metadata, score, author, permalink, and selftext.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | Yes | Search query terms (supports boolean logic `AND`, `OR`, `NOT`). |
| `subreddit` | string | No | Subreddit name (or `"all"`). Default: `"all"`. |
| `limit` | integer | No | Maximum number of posts to return (default: 25, max: 100). |
| `account` | string | No | Optional account alias. |

## Return Schema
```json
{
  "query": "vibe coding bugs",
  "subreddit": "all",
  "results_count": 2,
  "threads": [
    {
      "id": "1j0abc",
      "subreddit": "ExperiencedDevs",
      "title": "How are you dealing with junior engineers shipping vibe-coded PRs?",
      "selftext": "We've had three incidents this week where AI-generated code...",
      "author": "lead_architect",
      "score": 342,
      "num_comments": 158,
      "created_utc": 1740000000,
      "permalink": "/r/ExperiencedDevs/comments/1j0abc/how_are_you_dealing_with_junior_engineers/",
      "canonical_url": "https://www.reddit.com/r/ExperiencedDevs/comments/1j0abc/how_are_you_dealing_with_junior_engineers/"
    }
  ]
}
```

## Agent Usage Pattern
Use for ad-hoc research, finding precedent discussions, checking for duplicate questions, or exploring niche opinions.

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "9",
  "method": "tools/call",
  "params": {
    "name": "reddit_search",
    "arguments": {
      "query": "Claude code review quality",
      "subreddit": "programming",
      "limit": 10
    }
  }
}
```
