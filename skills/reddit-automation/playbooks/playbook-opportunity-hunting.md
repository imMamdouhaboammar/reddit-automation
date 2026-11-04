# Playbook: Autonomous Opportunity Hunting (Digger & Miner Pipeline)

> **Role:** Deep Source Candidate Aggregator (`reddit_digger`) & Content Intent Miner (`reddit_miner`)  
> **Primary Tools:** `reddit_digger`, `reddit_miner` (or composite `reddit_discover_opportunities`)  
> **Input Invariant:** Never scan broad generic keywords. Use high-intent phrases and target subreddits.

---

## 1. Objective

Systematically identify active Reddit threads where community members are genuinely asking for recommendations, expressing friction, or seeking workflows that your product solves—filtering out venting, memes, locked threads, and off-topic viral posts.

---

## 2. The Two-Stage Architecture (Digger & Miner)

The opportunity hunting process is architecturally decoupled into two specialist tools:

```
   ┌───────────────────────────────────────────────────────────────┐
   │ 1. reddit-digger (Deep Source Aggregation)                     │
   │    - Maps target subreddits and high-intent query terms       │
   │    - Multi-tier retrieval (Shreddit + Arctic Shift archive)   │
   │    - Gathers candidate threads, metrics, and canonical URLs   │
   │    Output: DiggerResult { sources: [DiggerSource] }           │
   └───────────────────────────────┬───────────────────────────────┘
                                   │
                                   ▼
   ┌───────────────────────────────────────────────────────────────┐
   │ 2. reddit-miner (Content Extraction & Intent Classification)   │
   │    - Sanitizes untrusted text against prompt injection        │
   │    - Evaluates intent signals (RecommendationAsk, Pain, etc.) │
   │    - Computes product fit score & composite rank              │
   │    Output: MinerResult { opportunities: [RankedOpportunity] } │
   └───────────────────────────────────────────────────────────────┘
```

---

## 3. Intent Signal Matrix

Every thread mined by `reddit_miner` is classified into one or more deterministic signals:

| Signal Token | Weight | Definition & Evidence Pattern |
|---|---|---|
| `RecommendationAsk` | `0.90` | OP asks "what tool do you use", "looking for an alternative to X", "any software that handles Y". |
| `UrgentWorkflow` | `0.90` | OP describes an active blocking task: "I need to automate X by tomorrow", "how do you handle Y in production". |
| `ExpressedPain` | `0.75` | OP describes friction points matched to `product.pain_points`: "wasting hours on X", "getting banned for Y". |
| `CompetitorFrustration` | `0.60` | OP mentions a competitor with negative sentiment: "Tool X is too expensive", "tired of bugs in Tool Y". |

---

## 4. Execution Protocol for Agents

### Step 1: Run `reddit_digger` (Deep Candidate Discovery)
Determine target subreddits and query phrases:
```json
{
  "product_name": "RedditAgent",
  "target_subreddits": ["SaaS", "webdev", "indiehackers", "startups"],
  "high_intent_phrases": ["reddit marketing tool recommendation", "alternative to spammy bots"],
  "limit_per_sub": 15
}
```
`reddit_digger` gathers all candidate sources with canonical URLs without loading heavy comment trees.

### Step 2: Run `reddit_miner` (Content Extraction & Semantic Analysis)
Pass the `sources` array from `reddit_digger` and your `product` context:
```json
{
  "sources": digger_result.sources,
  "product": {
    "product_name": "RedditAgent",
    "value_prop": "Fast, local-first, multi-account agentic operator in Rust with MCP",
    "icp_target": "Founders, indie hackers, and community managers",
    "target_subreddits": ["SaaS", "webdev", "indiehackers", "startups"],
    "pain_points": ["wasting hours on reddit", "getting banned for spam", "manual lead finding"],
    "competitors": ["spam-blaster", "generic-bot"],
    "high_intent_phrases": ["reddit marketing tool recommendation", "alternative to spammy bots"],
    "affiliation_disclosure": "Full disclosure: I work on RedditAgent"
  }
}
```

*(Note: Agents seeking single-turn execution may also call the composite pipeline `reddit_discover_opportunities`)*

### Step 3: Qualification Gate
Filter the opportunities returned by `reddit_miner`:
1. `composite_score >= 0.65`
2. `detected_signals` contains at least one of `RecommendationAsk`, `UrgentWorkflow`, or `ExpressedPain`.
3. The post is unlocked, unarchived, and less than 48 hours old.
4. Total comments `< 40` (threads with 100+ comments are already saturated).

### Step 4: Output Synthesis
For each survivor, format a single actionable summary:
- **Title & URL:** `[Title](canonical_url)`
- **Signal:** `RecommendationAsk` (Confidence: 0.90)
- **Why We Can Help:** "OP is actively looking for an alternative to spammy marketing bots in r/SaaS where RedditAgent provides local-first, disclosure-compliant outreach."
