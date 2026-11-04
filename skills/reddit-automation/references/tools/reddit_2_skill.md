# Tool Reference: `reddit_2_skill`

> **Category:** Autonomous Skill Synthesis & Knowledge Mining  
> **Host Requirements:** None (keyless crawling engine)  
> **Primary Use:** Mine deep Reddit consensus on any topic and synthesize an Agent Skill conforming to OmniSkill v6.2.0 and Poteto Mode.

---

## 1. Description

Takes a user topic or domain problem, infers high-signal practitioner subreddits, executes keyless multi-tier crawling, extracts empirical failure modes and peer-reviewed workarounds, and synthesizes a full Tripartite Agent Skill (`SKILL.md`, `scripts/`, `references/`) with executable preflight checks and evidence dossiers.

---

## 2. Input Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `topic` | string | **Yes** | Domain problem or subject (e.g. "Shopify headless checkout dropoff") |
| `subreddits` | array[string] | No | Optional list of subreddits to prioritize |
| `output_dir` | string | No | Base directory to write the generated skill (default: "skills") |

---

## 3. Example Request & Response

### Request:
```json
{
  "topic": "Vibe coding hallucinations and recursive bug loops",
  "subreddits": ["vibecoding", "ChatGPTCoding", "webdev"],
  "output_dir": "skills"
}
```

### Response:
```json
{
  "skill_name": "vibe-coding-hallucinations",
  "display_name": "🛡️ vibe coding hallucinations Operator (Reddit-Synthesized)",
  "description": "Empirical engineering skill synthesized from deep Reddit consensus across r/vibecoding, r/ChatGPTCoding, r/webdev.",
  "target_subreddits": ["vibecoding", "ChatGPTCoding", "webdev"],
  "analyzed_threads_count": 18,
  "output_dir": "skills/vibe-coding-hallucinations"
}
```

---

## 4. Agent Instructions
- Point the user to the newly forged skill directory.
- Explain the Core Invariants and 4-step loop synthesized from the Reddit evidence.
