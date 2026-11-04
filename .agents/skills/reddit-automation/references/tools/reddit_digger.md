# Tool: reddit_digger (reddit-digger)

Deeply determine search targets and aggregate candidate sources from Reddit.

## Purpose & Scope
`reddit_digger` (or `reddit-digger`) is the deep candidate discovery and source mapping engine. It determines:
- Exactly which subreddits to target for a given topic or product.
- Which high-intent query variations and search phrases to dispatch.
- Aggregates candidate threads deeply using keyless multi-tier scraping (Public Shreddit components + Arctic Shift archive fallback).
- Gathers rich thread metadata, scores, comment counts, author handles, permalinks, canonical URLs, and preview snippets.

Does not perform full-body classification or intent ranking; that is delegated to `reddit_miner`.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `target_subreddits` | array of strings | Yes | List of subreddits to deeply scan (e.g. `["SaaS", "startups"]`). |
| `product_name` | string | No | Optional name of product or topic context. |
| `value_prop` | string | No | One-line value proposition or problem context. |
| `high_intent_phrases` | array of strings | No | Search phrases or buyer questions (e.g. `["tool recommendation", "alternative to X"]`). |
| `limit_per_sub` | integer | No | Max candidate threads to collect per subreddit (default: 15). |

## Return Schema (`DiggerResult`)
```json
{
  "target_subreddits": ["SaaS", "startups"],
  "queries_executed": ["recommendation", "alternative to"],
  "total_sources": 8,
  "sources": [
    {
      "id": "1h4abc",
      "subreddit": "SaaS",
      "title": "Need recommendations for customer outreach tools without spam",
      "author": "founder_x",
      "score": 42,
      "num_comments": 19,
      "created_utc": 1735000000,
      "permalink": "/r/SaaS/comments/1h4abc/need_recommendations_for_customer_outreach_tools/",
      "canonical_url": "https://reddit.com/r/SaaS/comments/1h4abc/need_recommendations_for_customer_outreach_tools/",
      "body_snippet": "We have tried multiple tools but keep hitting rate limits...",
      "full_body": "We have tried multiple tools but keep hitting rate limits..."
    }
  ]
}
```

## Agent Usage Pattern
Run `reddit_digger` as Phase 1 of opportunity hunting. Pass the resulting `sources` directly into `reddit_miner` for Phase 2 deep inspection.

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "digger_1",
  "method": "tools/call",
  "params": {
    "name": "reddit_digger",
    "arguments": {
      "product_name": "AgenticReddit",
      "target_subreddits": ["SaaS", "ExperiencedDevs"],
      "high_intent_phrases": ["how do you find customers", "tool recommendations"],
      "limit_per_sub": 10
    }
  }
}
```

## Native CLI Command
```bash
cargo run --bin reddit-agent -- opportunity dig --subreddit SaaS --query "recommendation" --limit 15
```
