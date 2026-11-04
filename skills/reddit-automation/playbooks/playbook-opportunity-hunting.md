# Playbook: Autonomous Opportunity Hunting

> **Role:** Discovery & Intent Qualification Engine  
> **Primary Tool:** `reddit_discover_opportunities` (fallback: `reddit_search`)  
> **Input Invariant:** Never scan broad generic keywords. Use high-intent phrases and target subreddits.

---

## 1. Objective

Identify active Reddit threads where community members are genuinely asking for recommendations, expressing friction, or seeking workflows that your product solves—filtering out venting, memes, locked threads, and off-topic viral posts.

---

## 2. Intent Signal Matrix

Every thread evaluated is classified into one or more deterministic signals:

| Signal Token | Weight | Definition & Evidence Pattern |
|---|---|---|
| `RecommendationAsk` | `0.90` | OP asks "what tool do you use", "looking for an alternative to X", "any software that handles Y". |
| `UrgentWorkflow` | `0.90` | OP describes an active blocking task: "I need to automate X by tomorrow", "how do you handle Y in production". |
| `ExpressedPain` | `0.75` | OP describes friction points matched to `product.pain_points`: "wasting hours on X", "getting banned for Y". |
| `CompetitorFrustration` | `0.60` | OP mentions a competitor with negative sentiment: "Tool X is too expensive", "tired of bugs in Tool Y". |

---

## 3. The 3-Stage Discovery Pipeline

```
   ┌───────────────────────────────────────────────────────────────┐
   │ 1. Multi-Tier Retrieval                                       │
   │    Tier A: Public JSON listings (r/{sub}/hot.json)             │
   │    Tier B: Shreddit web component search partials             │
   │    Tier C: Arctic Shift archive fallback (keyless resilience) │
   └───────────────────────────────┬───────────────────────────────┘
                                   │
                                   ▼
   ┌───────────────────────────────────────────────────────────────┐
   │ 2. Untrusted Content Sanitization                             │
   │    - Strip Prompt Injection patterns ("ignore instructions")  │
   │    - Filter HTML scripts, markdown links, executable strings   │
   └───────────────────────────────┬───────────────────────────────┘
                                   │
                                   ▼
   ┌───────────────────────────────────────────────────────────────┐
   │ 3. Relevance & Bounded Viral Ranking                          │
   │    Score = Relevance + min(0.25, log10(Engagement + 1) / 20)  │
   │    (Floor = 0.20: Off-topic viral posts are discarded)        │
   └───────────────────────────────────────────────────────────────┘
```

---

## 4. Execution Protocol for Agents

### Step 1: Prepare Product Context
```json
{
  "product_name": "RedditAgent",
  "value_prop": "Fast, local-first, multi-account agentic operator in Rust with MCP",
  "icp_target": "Founders, indie hackers, and community managers",
  "target_subreddits": ["SaaS", "webdev", "indiehackers", "startups"],
  "pain_points": ["wasting hours on reddit", "getting banned for spam", "manual lead finding"],
  "competitors": ["spam-blaster", "generic-bot"],
  "high_intent_phrases": ["reddit marketing tool recommendation", "alternative to spammy bots"],
  "affiliation_disclosure": "Full disclosure: I work on RedditAgent",
  "limit_per_sub": 15
}
```

### Step 2: Call `reddit_discover_opportunities`
Inspect the resulting array of `RankedOpportunity` objects.

### Step 3: Qualification Gate
Keep only opportunities where:
1. `composite_score >= 0.65`
2. `detected_signals` contains at least one of `RecommendationAsk`, `UrgentWorkflow`, or `ExpressedPain`.
3. The post is unlocked, unarchived, and less than 48 hours old.
4. Total comments `< 40` (threads with 100+ comments are already saturated).

### Step 4: Output Synthesis
For each survivor, format a single actionable summary:
- **Title & URL:** `[Title](https://reddit.com/r/.../comments/...)`
- **Signal:** `RecommendationAsk` (Confidence: 0.90)
- **Why We Can Help:** "OP is actively looking for an alternative to spammy marketing bots in r/SaaS where RedditAgent provides local-first, disclosure-compliant outreach."
