# Tool Reference: `reddit_discover_opportunities`

> **Category:** Autonomous Discovery & Intent Scoring  
> **Host Requirements:** None (operates keyless or authenticated)  
> **Primary Use:** Discover high-intent Reddit threads where users genuinely need what you build.

---

## 1. Description

Scans target subreddits using multi-tier scraping (Public Shreddit JSON, site search, and Arctic Shift archive fallback). Sanitizes untrusted content to neutralize prompt injection, scores threads against 4 intent signals, applies a relevance floor, and bounds viral engagement bonus ($0.25$ cap) so off-topic viral posts never outrank relevant discussions.

---

## 2. Input Parameters

| Parameter | Type | Required | Description | Example |
|---|---|---|---|---|
| `product_name` | string | **Yes** | Official name of your product or tool | `"RedditAgent"` |
| `value_prop` | string | **Yes** | One-line core value proposition | `"Local-first autonomous Reddit operator"` |
| `icp_target` | string | No | Ideal customer profile | `"Founders and community builders"` |
| `target_subreddits` | array[string] | **Yes** | 3–15 communities to search | `["SaaS", "indiehackers"]` |
| `pain_points` | array[string] | No | Specific friction points solved | `["wasting hours", "getting banned"]` |
| `competitors` | array[string] | No | Competitors users complain about | `["spam-blaster"]` |
| `high_intent_phrases`| array[string] | No | Phrases used by buyers | `["tool recommendation", "alternative"]` |
| `affiliation_disclosure` | string | No | Mandatory affiliation disclaimer | `"Full disclosure: I work on RedditAgent"` |
| `limit_per_sub` | integer | No | Max threads to fetch per sub (default: 15) | `10` |

---

## 3. Example Request

```json
{
  "product_name": "RedditAgent",
  "value_prop": "Local-first multi-account Reddit operator with MCP",
  "target_subreddits": ["SaaS", "webdev"],
  "pain_points": ["spammy bots", "wasting hours on reddit"],
  "high_intent_phrases": ["reddit marketing tool recommendation"],
  "affiliation_disclosure": "Full disclosure: I work on RedditAgent",
  "limit_per_sub": 10
}
```

---

## 4. Example Response

```json
[
  {
    "thread": {
      "id": "1h3xyz",
      "subreddit": "SaaS",
      "title": "Need a tool recommendation for Reddit outreach without spam",
      "body": "I have been trying to find customers on Reddit...",
      "author": "indie_dev",
      "created_utc": 1700000000,
      "num_comments": 8,
      "is_locked": false,
      "is_archived": false,
      "url": "https://reddit.com/r/SaaS/comments/1h3xyz/"
    },
    "op_signal_score": 0.90,
    "product_fit_score": 0.85,
    "timing_score": 0.80,
    "composite_score": 0.86,
    "detected_signals": ["RecommendationAsk", "UrgentWorkflow"],
    "opportunity_reason": "OP is actively asking for a tool recommendation in r/SaaS where RedditAgent directly solves the problem."
  }
]
```

---

## 5. Agent Instructions
- Pass the output directly to `reddit_draft_response`.
- Never invent product details or fake quotes.
