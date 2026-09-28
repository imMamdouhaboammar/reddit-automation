# ADR-001: Toolchain, Runtime, and CLI Architecture Selection

## Status
Accepted

## Context
The project is an autonomous Reddit agentic automation CLI tool. It requires fast execution, strict type-safety, rate-limited and dry-run protected HTTP client operations, structured configuration parsing, and comprehensive testability without polluting or spamming live Reddit communities during development or CI.

## Decision
1. **Runtime & Package Management**: Use **Bun** exclusively as the JavaScript/TypeScript runtime and package manager.
2. **CLI Framework**: Adopt **Commander.js** for command hierarchy, option flags (`--dry-run`, `--live`, `--subreddit`, `--limit`), and help generation.
3. **Bundler**: Use **tsup** to bundle TypeScript into clean ESM deliverables in `dist/`.
4. **Test Suite**: Use **Vitest** for fast unit and integration tests with Reddit API mocking.
5. **Schema & Config**: Use **Zod** for environment variables and API payload validation.
6. **Safety Default**: Enforce `DRY_RUN=true` by default across all configuration layers.

## Consequences
- Fast developer loop with zero-transpilation overhead during development via `bun run`.
- Single source of truth for runtime validation prevents runtime crashes caused by malformed API responses.
- Guarantees zero accidental mutations to live Reddit during local testing or CI.
