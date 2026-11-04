---
name: vibe-coding-bugs-and
version: "1.0.0"
displayName: "🛡️ vibe coding bugs and Operator (Reddit-Synthesized)"
description: >
  Empirical engineering skill synthesized from deep Reddit consensus across r/vibecoding, r/ChatGPTCoding, r/webdev, r/programming.
  Addresses real-world failure modes, undocumented traps, and battle-tested solutions
  for 'Vibe Coding bugs and memory leaks' adhering to OmniSkill v6.2.0 and Poteto Mode standards.
argument-hint: 'vibe-coding-bugs-and audit | vibe-coding-bugs-and execute'
allowed-tools: Bash, Read, Write
metadata:
  openclaw:
    emoji: "🛡️"
    requires:
      bins: [bash, python3]
    tags:
      - reddit-2-skill
      - omni-skill
      - poteto-mode
      - vibe-coding-bugs-and
---

# 🛡️ vibe coding bugs and Operator (Reddit-Synthesized)

*Engineered by `reddit-2-skill` using real practitioner intelligence mined from r/vibecoding, r/ChatGPTCoding, r/webdev, r/programming.*

## Core Invariants (Fail-Closed Gates)

1. **Single Source of Truth:** Never improvise configurations or assumptions without empirical validation.
2. **Red-Green Falsification:** A fix is not verified until tested against the exact failure mode reported by users.
3. **Direct Technical Payload:** Zero affordance narration, zero conversational fluff — execute deterministic checks directly.

## 4-Step Operational Loop

### Step 1: Preflight & Context Discovery
- Inspect current environment, dependencies, and configuration states.
- Execute `scripts/preflight_check.py` to identify silent regressions.

### Step 2: Anti-Pattern Detection
- Check against known failure modes documented in `references/reddit_evidence.md`.
- Flag and block recursive trial-and-error edits.

### Step 3: Targeted Remediation
- Apply minimal, isolated modifications strictly bounded to the identified root cause.
- Respect community-validated workarounds.

### Step 4: Verification & Durable Codification
- Run full test suite and confirm green status.
- Codify learnings to prevent future recurrence.

