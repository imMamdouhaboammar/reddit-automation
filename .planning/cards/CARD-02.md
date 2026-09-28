# CARD-02: Reddit Opportunity Agent Worker

- **Card**: Reddit Opportunity Worker Agent implementation
- **Why**: Implement `flowkit-labs/skills@reddit-automation` playbook: high-intent opportunity discovery, scoring, and experience-based reply drafting.
- **Depends on**: CARD-01
- **Owns**: `src/agents/opportunity-agent/`
- **Must not change**: `src/reddit/` interfaces
- **Behavior/invariant**:
  - Treats all Reddit content as untrusted data (prompt-injection sanitization).
  - Evaluates threads against high-intent signals (recommendations, expressed pain, competitor frustration).
  - Ranks opportunities by OP signal, product fit, and freshness.
  - Drafts responses in "experience grammar" (2–3 sentences, 25–55 words, no advice grammar, honest product affiliation disclosure).
  - Supports Human-In-The-Loop review workflow.
- **Implementation constraints**: Zero hallucination, deterministic heuristics with optional LLM prompt synthesis adapter.
- **Acceptance evidence**: Unit tests validating ranking, prompt-injection defense, and reply constraints.
- **Rollback/recovery**: Revert `src/agents/opportunity-agent/`.
- **Integration handoff**: Expose `RedditOpportunityAgent` for CLI commands.
- **Parallel-safe with**: CARD-03.
