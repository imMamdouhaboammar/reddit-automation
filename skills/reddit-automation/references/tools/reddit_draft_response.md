# Tool Reference: `reddit_draft_response`

> **Category:** Response Generation & Compliance  
> **Host Requirements:** None  
> **Primary Use:** Draft helpful, 25-55 word replies adhering to Experience Grammar with mandatory affiliation disclosure.

---

## 1. Description

Generates a draft response for a `RankedOpportunity`. Enforces the linguistic contract of Experience Grammar ("I ran into this exact issue...", "What worked for me was..."), cites a concrete detail from OP's post, embeds an honest hedge ("at least in my case..."), evaluates the 3-part product naming gate, and mandates affiliation disclosure if the product is named.

---

## 2. Input Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `opportunity` | object | **Yes** | `RankedOpportunity` object from `reddit_discover_opportunities` |
| `product` | object | **Yes** | `ProductContext` with product details and disclosure |

---

## 3. Example Request

```json
{
  "opportunity": {
    "thread": {
      "id": "1h3xyz",
      "subreddit": "SaaS",
      "title": "Need a tool recommendation for Reddit outreach without spam",
      "body": "Does anyone know a good tool?",
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
    "detected_signals": ["RecommendationAsk"],
    "opportunity_reason": "OP is actively asking for a tool recommendation"
  },
  "product": {
    "product_name": "RedditAgent",
    "value_prop": "Local-first Reddit operator",
    "icp_target": "Founders",
    "pain_points": ["spammy bots"],
    "competitors": [],
    "target_subreddits": ["SaaS"],
    "high_intent_phrases": ["tool recommendation"],
    "affiliation_disclosure": "Full disclosure: I work on RedditAgent"
  }
}
```

---

## 4. Example Response

```json
{
  "thread_id": "1h3xyz",
  "draft_text": "I ran into this exact issue when working on Need a tool recommendation — what worked for me was automating the triage so only high-signal threads surfaced. Full disclosure: I work on RedditAgent, so I'm biased, but keeping replies under 50 words completely stopped moderator flags, at least in my case.",
  "word_count": 51,
  "experience_grammar_valid": true,
  "product_named": true,
  "affiliation_disclosed": true,
  "concrete_detail_referenced": "Need a tool recommendation",
  "review_status": "PendingHumanReview"
}
```

---

## 5. Agent Instructions
- Present the draft to the user for review.
- Never publish directly without going through the two-stage ActionPlan pipeline (`reddit_prepare_comment` -> `reddit_approve_action`).
