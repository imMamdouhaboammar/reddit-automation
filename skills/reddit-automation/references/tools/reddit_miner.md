# Tool: reddit_miner (reddit-miner)

Extract content, inspect discussion depth, classify buying intent, and rank Reddit opportunities.

## Purpose & Scope
`reddit_miner` (or `reddit-miner`) is the deep extraction and semantic inspection engine. It receives candidate sources from `reddit_digger` (or specific thread IDs/URLs) and:
- Extracts complete post text and submission details.
- Sanitizes prompt injection attempts (`"ignore instructions"`, `<script>`, shell injection).
- Evaluates OP intent signals:
  - `RecommendationAsk` ($0.90$): Actively shopping for solutions.
  - `UrgentWorkflow` ($0.90$): Immediate operational blocker.
  - `ExpressedPain` ($0.75$): Pain points matching product capabilities.
  - `CompetitorFrustration` ($0.60$): Complaints regarding existing market tools.
- Computes product fit score, timing freshness, and composite rank:
  $$\text{Composite} = (\text{Signal} \times 0.45) + (\text{ProductFit} \times 0.35) + (\text{Timing} \times 0.20)$$
- Produces ranked opportunities with canonical Reddit URLs ready for response drafting.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sources` | array of objects | Yes | Array of `DiggerSource` objects (or `RedditThread` objects) returned by `reddit_digger`. |
| `product` | object | Yes | `ProductContext` specifying `product_name`, `pain_points`, `competitors`, `high_intent_phrases`, and `affiliation_disclosure`. |

## Return Schema (`MinerResult`)
```json
{
  "analyzed_count": 8,
  "opportunities": [
    {
      "thread": {
        "id": "1h4abc",
        "subreddit": "SaaS",
        "title": "Need recommendations for customer outreach tools without spam",
        "body": "We have tried multiple tools but keep hitting rate limits...",
        "author": "founder_x",
        "created_utc": 1735000000,
        "num_comments": 19,
        "is_locked": false,
        "is_archived": false,
        "url": "https://reddit.com/r/SaaS/comments/1h4abc/need_recommendations_for_customer_outreach_tools/",
        "canonical_url": "https://reddit.com/r/SaaS/comments/1h4abc/need_recommendations_for_customer_outreach_tools/"
      },
      "op_signal_score": 0.90,
      "product_fit_score": 0.85,
      "timing_score": 0.88,
      "composite_score": 0.87,
      "detected_signals": ["RecommendationAsk", "ExpressedPain"],
      "opportunity_reason": "OP is actively asking for a tool recommendation in r/SaaS where AgenticReddit directly solves the problem."
    }
  ]
}
```

## Agent Usage Pattern
Pipe the output of `reddit_digger` into `reddit_miner`:

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "miner_1",
  "method": "tools/call",
  "params": {
    "name": "reddit_miner",
    "arguments": {
      "sources": digger_result.sources,
      "product": {
        "product_name": "AgenticReddit",
        "value_prop": "Local-first authentic Reddit outreach",
        "icp_target": "Founders and builders",
        "pain_points": ["spammy scrapers", "rate limits"],
        "competitors": ["spam-blaster"],
        "target_subreddits": ["SaaS"],
        "high_intent_phrases": ["tool recommendation"],
        "affiliation_disclosure": "Full disclosure: I built AgenticReddit"
      }
    }
  }
}
```

## Native CLI Command
```bash
cargo run --bin reddit-agent -- opportunity mine --subreddit SaaS --product "AgenticReddit" --limit 10
```
