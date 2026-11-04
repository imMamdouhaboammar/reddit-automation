# Playbook: Deep Reddit Scouring to OmniSkill Synthesis (`reddit-2-skill`)

> **Role:** Skill Architect & Knowledge Distiller  
> **Primary Tool:** `reddit_2_skill` (CLI: `reddit-agent r2s`)  
> **Core Objective:** Mine real practitioner intelligence, undocumented traps, and battle-tested workflows across Reddit, and synthesize a production-grade Agent Skill compliant with `imMamdouhaboammar/omni-skill v6.2.0` and Poteto Mode.

---

## 1. Concept & Invariants

```
   [ Topic / Problem Context ]
                │
                ▼ 1. Subreddit Auto-Inference
   ┌────────────────────────────────────────────────────────┐
   │ Infer 3-5 high-signal practitioner subreddits          │
   │ (e.g. "Shopify headless" -> r/shopify, r/ecommerce)    │
   └────────────────────────┬───────────────────────────────┘
                            │
                            ▼ 2. Multi-Tier Deep Crawl
   ┌────────────────────────────────────────────────────────┐
   │ RedditKeylessDriver executes:                          │
   │ - Public listings (hot, top month)                     │
   │ - Shreddit web component search partials               │
   │ - Arctic Shift archive fallback                        │
   │ - Noise-word token overlap relevance filtering         │
   └────────────────────────┬───────────────────────────────┘
                            │
                            ▼ 3. Empirical Distillation
   ┌────────────────────────────────────────────────────────┐
   │ Extract:                                               │
   │ - Core Invariants (Rules where breaking causes bugs)   │
   │ - 4-Step Operational Loop                              │
   │ - Real Thread URLs & Evidence Dossier                  │
   │ - Executable Preflight Script                          │
   └────────────────────────┬───────────────────────────────┘
                            │
                            ▼ 4. OmniSkill Packaging
   ┌────────────────────────────────────────────────────────┐
   │ Generate Tripartite Package:                           │
   │ skills/<skill-name>/                                   │
   │ ├── SKILL.md                 (AgentSkills.io v1)       │
   │ ├── scripts/preflight_check.py                         │
   │ └── references/reddit_evidence.md                      │
   └────────────────────────────────────────────────────────┘
```

---

## 2. Invocation via MCP Tool

```json
{
  "name": "reddit_2_skill",
  "arguments": {
    "topic": "FastAPI async database connection leaks in production",
    "subreddits": ["FastAPI", "Python", "webdev"],
    "output_dir": "skills"
  }
}
```

### Generated Artifact Deliverables:
1. `skills/fastapi-async-database/SKILL.md`:
   - Frontmatter conforming to AgentSkills.io v1.
   - Core Invariants (e.g. always close connection pool on lifespan shutdown).
   - 4-Step Operational Loop.
2. `skills/fastapi-async-database/scripts/preflight_check.py`:
   - Executable diagnostic checking connection pool parameters.
3. `skills/fastapi-async-database/references/reddit_evidence.md`:
   - Clickable Reddit URLs of all analyzed practitioner discussions.

---

## 3. Invocation via CLI

```bash
# Full command
reddit-agent reddit-2-skill --topic "Vibe coding bugs and fix loops" --output-dir skills

# Short alias
reddit-agent r2s --topic "Next.js 15 App Router caching issues" --output-dir skills
```
