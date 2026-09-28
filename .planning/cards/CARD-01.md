# CARD-01: Domain Types and Reddit Client Abstraction

- **Card**: Domain types, schemas, and Reddit API client abstraction with rate-limiting and mock adapter
- **Why**: Solid type-safe foundation and mockable Reddit interface to allow deterministic agent execution without live network spam.
- **Depends on**: None
- **Owns**: `src/types/`, `src/reddit/`, `src/config/`
- **Must not change**: Existing documentation invariants in `docs/CONSTITUTION.md`
- **Behavior/invariant**:
  - `DRY_RUN=true` strictly respected.
  - User-Agent strictly formatted.
  - Reddit client supports read methods (`fetchSubredditPosts`, `searchPosts`, `fetchPostComments`).
  - Mock client returns realistic synthetic fixtures for testing.
- **Implementation constraints**: Bun runtime, TypeScript strict mode, Zod validation.
- **Acceptance evidence**: Unit test passes for schema validation and mock Reddit client data retrieval.
- **Rollback/recovery**: Revert files in `src/types/`, `src/reddit/`, `src/config/`.
- **Integration handoff**: Expose `RedditClient` and types for agent workers.
- **Parallel-safe with**: CARD-02 specification.
