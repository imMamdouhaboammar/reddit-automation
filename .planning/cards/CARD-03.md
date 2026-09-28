# CARD-03: Reddit Community Builder Agent Worker

- **Card**: Reddit Community Builder Worker Agent implementation
- **Why**: Implement `marketing-reddit-community-builder` playbook: 90/10 value-to-promo ratio, subreddit culture & sentiment analysis, educational content strategies, and brand reputation monitoring.
- **Depends on**: CARD-01
- **Owns**: `src/agents/community-builder-agent/`
- **Must not change**: `src/reddit/` interfaces
- **Behavior/invariant**:
  - Enforces 90/10 rule (90% pure value-add, max 10% promotional).
  - Performs subreddit sentiment analysis (positive/neutral/negative balance).
  - Generates value-first educational content outlines and AMA question/talking-point guides.
  - Monitors brand mentions and drafts authentic, non-defensive remediation responses.
- **Implementation constraints**: Deterministic heuristic rules with structured output schemas.
- **Acceptance evidence**: Unit tests verifying 90/10 ratio calculation, sentiment breakdown, and educational content generation.
- **Rollback/recovery**: Revert `src/agents/community-builder-agent/`.
- **Integration handoff**: Expose `RedditCommunityBuilderAgent` for CLI commands.
- **Parallel-safe with**: CARD-02.
