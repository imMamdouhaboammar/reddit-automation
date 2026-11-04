---
name: reddit-automation
displayName: "👽 Reddit Automation — Autonomous Opportunity Discovery, Experience Drafting & Verified Publishing"
description: >
  Fully agentic Reddit automation operator built on reddit-agent's high-performance
  Rust engine and Model Context Protocol (MCP). Replaces manual copy-pasting with
  multi-tier keyless thread discovery (Shreddit + Arctic Shift fallback), semantic
  relevance filtering, prompt-injection sanitization, experience grammar drafting
  (25-55 words with mandatory affiliation disclosure), 90/10 community ratio auditing,
  and cryptographic two-stage approval gates (SHA-256) for secure, ban-proof engagement.
emoji: "👽"
homepage: https://github.com/imMamdouhaboammar/reddit-agent
license: MIT
metadata:
  openclaw:
    emoji: "👽"
    requires:
      bins:
        - reddit-agent
        - cargo
        - bun
    primaryEnv: REDDIT_CLIENT_ID
    tags:
      - reddit
      - marketing
      - lead-generation
      - automation
      - mcp
      - agentic
      - growth
---

# 👽 Reddit Automation (Fully Autonomous Agentic Operator)

*Engineered as the autonomous evolution of `flowkit-labs/skills` powered by [`reddit-agent`](https://github.com/imMamdouhaboammar/reddit-agent).*

**Automate high-intent community discovery, experience-grammar drafting, and disclosure-compliant engagement on Reddit with zero spam, zero astroturfing, and cryptographic fail-closed safety.**

Unlike manual prompt scripts that require tedious copy-pasting, this skill provides a full end-to-end autonomous agent loop backed by a native Rust engine, local OS Keychain credential encryption, headless Chrome DevTools Protocol (CDP), and standard Model Context Protocol (MCP) server tools.

---

## Autonomous Capabilities & Architecture

```
   ┌───────────────────────────────────────────────────────────────┐
   │                  Autonomous AI Agent Client                   │
   │           (Claude Desktop, Cursor, Antigravity, CLI)          │
   └───────────────────────────────┬───────────────────────────────┘
                                   │ JSON-RPC (MCP Tools)
                                   ▼
   ┌───────────────────────────────────────────────────────────────┐
   │             reddit-agent Native Operator (Rust Core)          │
   │                                                               │
   │  ┌───────────────────────┐         ┌───────────────────────┐  │
   │  │  Opportunity Worker   │         │  Community Builder    │  │
   │  │  - Intent Scoring     │         │  - 90/10 Ratio Audit  │  │
   │  │  - Prompt Sanitizer   │         │  - Karma Health Check │  │
   │  │  - Experience Grammar │         │  - Crisis Mitigation  │  │
   │  └───────────┬───────────┘         └───────────┬───────────┘  │
   │              │                                 │              │
   │  ┌───────────▼─────────────────────────────────▼───────────┐  │
   │  │              Multi-Tier Scraping & Discovery            │  │
   │  │  1. Public JSON Listings (r/{sub}/hot.json)             │  │
   │  │  2. Shreddit Web Component Partials                     │  │
   │  │  3. Arctic Shift Archive Fallback (Keyless & Resilient) │  │
   │  │  4. OAuth2 Authenticated API Driver                     │  │
   │  └───────────────────────────┬─────────────────────────────┘  │
   │                              │                                │
   │  ┌───────────────────────────▼─────────────────────────────┐  │
   │  │     Fail-Closed Cryptographic Gate (ActionPlan)         │  │
   │  │     - Two-stage Dry-Run: Hash = SHA-256(payload)        │  │
   │  │     - Explicit Human Approval Before Network Mutation   │  │
   │  └─────────────────────────────────────────────────────────┘  │
   └───────────────────────────────────────────────────────────────┘
```

---

## Quick Trigger Phrases

- `"Find Reddit opportunities for my product this week"`
- `"Scan r/SaaS and r/startups for people asking for my tool"`
- `"Draft experience-grammar replies for top Reddit threads"`
- `"Audit our Reddit account health and 90/10 karma ratio"`
- `"Execute approved Reddit replies safely"`

---

## The 4-Phase Autonomous Agent Loop

### Phase 1: Onboard Product Context (Executed Once)

The agent requests or reads `ProductContext` from `.reddit-agent/product.json` or user inputs:
- `product_name`: Official tool/service name.
- `value_prop`: One-line core value proposition.
- `icp_target`: Ideal customer profile and buyer persona.
- `pain_points`: Specific friction points your product removes.
- `competitors`: Competitors buyers commonly express frustration with.
- `target_subreddits`: 5–15 focused communities where target users discuss workflows.
- `high_intent_phrases`: Exact search phrases reflecting urgent purchase/tool needs.
- `affiliation_disclosure`: Honest affiliation statement (e.g., `"Full disclosure: I am building X"`).

---

### Phase 2: Autonomous Opportunity Discovery & Scoring

Call MCP tool: `reddit_discover_opportunities`

```json
{
  "product_name": "MyTool",
  "value_prop": "Local-first automated testing operator",
  "icp_target": "Founders and engineers",
  "target_subreddits": ["SaaS", "webdev", "indiehackers"],
  "pain_points": ["flaky tests", "slow CI pipelines", "complex mock setups"],
  "competitors": ["legacy-runner", "manual-qa"],
  "high_intent_phrases": ["test runner recommendation", "how to speed up vitest"],
  "affiliation_disclosure": "Full disclosure: I work on MyTool",
  "limit_per_sub": 15
}
```

#### Deterministic Discovery & Scoring Rules:
1. **Multi-Source Scraping:** Attempts public JSON and Shreddit listings first. Automatically recovers missing or 403-blocked threads via Arctic Shift archive.
2. **Untrusted Data Sanitization:** Runs regex and string sanitization over all OP titles and bodies to neutralize prompt injection attacks (`"ignore previous instructions"`, `<script>`, etc.).
3. **Intent Signal Classification:**
   - `RecommendationAsk` ($0.90$ signal): OP actively asking for software recommendations.
   - `UrgentWorkflow` ($0.90$ signal): OP blocked by immediate operational obstacle.
   - `ExpressedPain` ($0.75$ signal): OP venting friction that product resolves.
   - `CompetitorFrustration` ($0.60$ signal): OP frustrated with alternative tool.
4. **Relevance Floor & Bounded Viral Engagement:**
   $$\text{Rank} = \text{Relevance} + \min\left(0.25, \frac{\log_{10}(\text{Score} + \text{Comments} + 1)}{20}\right)$$
   Guarantees that viral off-topic threads (memes, relationship drama) can never outrank on-topic threads.

---

### Phase 3: Autonomous Experience-Grammar Drafting

Call MCP tool: `reddit_draft_response`

```json
{
  "opportunity": { ... }, // RankedOpportunity from Phase 2
  "product": { ... }       // ProductContext
}
```

#### Strict Anti-Slop & Quality Gates:
- **Experience Grammar (Mandatory):**
  - ✅ **Allowed:** `"I ran into this exact issue..."`, `"What worked for me was..."`, `"In my experience..."`, `"What fixed it for me..."`.
  - ❌ **Forbidden:** `"You should..."`, `"You need to..."`, `"The best way is..."`, `"I'd just..."`, `"Step 1: ..."` (advice/support grammar).
- **Length Constraint:** Exactly **25 to 55 words** (2–3 punchy sentences). Long replies read as artificial marketing copy.
- **Concrete Anchor:** Must react directly to a specific detail the OP mentioned.
- **One Hedge:** Include at least one honest hedge (`"at least in my case..."`).
- **Product Naming Gate & Honest Disclosure:**
  - Only name the product if OP is explicitly shopping for a solution **AND** product fit score $\ge 0.65$.
  - If named, the affiliation disclaimer must be stated in the exact same sentence.
  - If the gate fails, draft a pure-value answer without mentioning the product name.

---

### Phase 4: Tamper-Proof Cryptographic Action Plan & Publication

To guarantee security and eliminate accidental spam:

1. **Prepare Action Plan:** Call MCP tool `reddit_prepare_comment`
   - Generates a pending action with a SHA-256 cryptographic signature:
     $$\text{Hash} = \text{SHA256}(\text{account} + \text{thing\_id} + \text{text} + \text{salt})$$
2. **Review & Human Approval Gate:** Call MCP tool `reddit_approve_action`
   - The user or supervising workflow inspects the exact diff and confirms approval.
   - Unapproved actions fail closed and cannot mutate state.
3. **Execute & Publish:** Call MCP tool `reddit_publish_prepared_comment`
   - Verifies the cryptographic hash matches the original un-tampered plan.
   - Publishes via authenticated Reddit OAuth2 driver with strict rate-limit backoff.
4. **Account Ratio Audit:** Call MCP tool `reddit_audit_community_ratio`
   - Validates that the account maintains at least a 90% organic value-to-promotion ratio.

---

## Command Line Equivalents

```bash
# Start MCP server for IDE / AI Agent integration
reddit-agent mcp

# List all configured accounts and verify health
reddit-agent account list
reddit-agent account health

# Prepare a post with dry-run validation
reddit-agent post prepare --subreddit SaaS --title "Experience scaling local agents" --body "..."

# Review and approve action plans
reddit-agent action list
reddit-agent action approve <ACTION_ID>
reddit-agent action execute <ACTION_ID>
```

---

## Ethical Standards & Content Policy

1. **Honest Attribution:** Never impersonate third-party consumers. Disclose your affiliation whenever a product or service is mentioned.
2. **No Astroturfing:** Coordinated sockpuppet upvoting, fake grassroots consensus, and vote rings are strictly banned.
3. **Community Respect:** Obey subreddit rules and wiki guidelines. If a community prohibits product mentions entirely, stay in 100% pure help mode without product naming.
4. **Data Isolation:** All session tokens, keyrings, and Chrome profiles remain local on the user's physical machine. No data is exfiltrated to external telemetry servers.
