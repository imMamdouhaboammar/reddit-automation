# Reddit Agentic Automation CLI Tool — Codebase Constitution

**System Version:** 0.0.1  
**Author:** Mamdouh Aboammar  
**License:** MIT  
**Toolchain:** Bun · TypeScript · Commander · tsup · Vitest  

$$\text{User Intent} \xrightarrow{\text{Commander CLI}} \text{Agentic Orchestration} \xrightarrow{\text{Policy \& Guardrails}} \text{Reddit API Client (Dry-Run by Default)} \xrightarrow{\text{Audit Log \& State}}$$

---

## 1. System Identity & Mission

The **Reddit Agentic Automation CLI Tool** (`reddit-agent`) is an autonomous, agentic command-line system designed to inspect, synthesize, monitor, and automate interactions with Reddit (subreddits, threads, comments, user insights) through intelligent agent workflows while strictly enforcing Reddit API terms of service, platform safety boundaries, rate limits, and fail-closed security.

---

## 2. Workspace Topology & Boundaries

```
.
├── .fable/                      # Fable lifecycle state and routing persistence
├── .planning/                   # Agentic workflow SOT (state.yaml) and cards
├── docs/                        # Engineering documentation
│   ├── CONSTITUTION.md          # Canonical system invariants & architecture
│   ├── agent-rules.yaml         # Machine-readable agent constraints
│   ├── adr/                     # Architecture Decision Records
│   │   ├── ADR-000-index.md
│   │   └── ADR-001-toolchain.md
│   └── solutions/               # Durable Compound Engineering learnings
├── src/                         # Application source code (created during execution)
│   ├── cli/                     # Commander CLI command registry & options
│   ├── core/                    # Agentic engine, workflow drivers, triage logic
│   ├── reddit/                  # Reddit API client, rate limiter, OAuth2 auth
│   ├── config/                  # Environment and configuration schema (Zod)
│   └── utils/                   # Logging, security sanitization, formatting
├── tests/                       # Vitest unit and integration test suites
│   ├── unit/                    # Fast mock-based tests
│   └── integration/             # Isolated harness tests
├── package.json                 # Bun package manifest
├── tsconfig.json                # TypeScript compiler config
└── .env.example                 # Safe credential template (DRY_RUN=true)
```

---

## 3. Technology Stack & Toolchain Contracts

1. **Runtime & Package Manager**: **Bun** is mandatory for all package installation, script execution, lockfile generation, and local development. (`npm` and `yarn` are prohibited).
2. **Language**: TypeScript (strict mode enabled, zero unchecked `any`).
3. **CLI Framework**: Commander.js with modular command definitions and explicit `--dry-run` flags.
4. **Build & Bundler**: `tsup` targeting ESM with declaration maps.
5. **Testing**: `Vitest` with mock-first Reddit adapters (no live network mutations during automated tests).
6. **Schema Validation**: `Zod` for runtime environment and API response validation.

---

## 4. Non-Negotiable Invariants (Constitutional Canon)

### MUST-NEVER
- **MUST-NEVER** expose API keys, client secrets, passwords, or refresh tokens in code, tests, logs, stdout, or git commits.
- **MUST-NEVER** execute live Reddit mutations (submitting posts, comments, votes, moderation actions) when `DRY_RUN=true` or `--dry-run` is active.
- **MUST-NEVER** bypass Reddit API rate limits, use spoofed/fake User-Agents, or perform scraping that violates Reddit's API Developer Terms.
- **MUST-NEVER** swallow errors silently; all exceptions must be caught, typed, and mapped to user-actionable remediation messages.
- **MUST-NEVER** make speculative edits when consecutive failures $\ge 2$ (Fable Circuit Breaker must trip to `OPEN`).

### MUST-ALWAYS
- **MUST-ALWAYS** default `DRY_RUN=true` unless explicitly overridden with `--live` and user confirmation.
- **MUST-ALWAYS** format `User-Agent` strictly following Reddit guidelines: `<platform>:<app ID>:<version> (by /u/<reddit_username>)`.
- **MUST-ALWAYS** respect `x-ratelimit-remaining`, `x-ratelimit-reset`, and HTTP 429 using exponential backoff with randomized jitter.
- **MUST-ALWAYS** validate input arguments and external API payloads using Zod schemas before processing.
- **MUST-ALWAYS** follow the 3-Pass Verification protocol (Unit, Integration, AppSec/Taste) before marking any task complete.

### MUST-WHEN
- **MUST-WHEN** processing Reddit markdown or user inputs, sanitize text to prevent prompt injection or markdown rendering exploits.
- **MUST-WHEN** an LLM agent produces a suggested response/action, compute a confidence score ($pACS \ge 70$) before recommending or queueing the action.
- **MUST-WHEN** creating or modifying interfaces, perform an AST blast-radius check across all importers.

---

## 5. Reddit API & Platform Guardrails

1. **Authentication Boundary**: Authentication must use Reddit OAuth2 (`script` or `installed` app flows) with token caching and proactive expiration checks.
2. **Rate Limit Governor**: A token-bucket or leaky-bucket governor inside `src/reddit/` must throttle requests to Reddit's current limits (e.g., max 60-100 requests per minute per OAuth client).
3. **Dry-Run Audit Trail**: Every command that would mutate Reddit state must output a structured preview of the mutation (target subreddit, title, body, action) and exit cleanly without network side effects.

---

## 6. Agentic Automation & Decision Boundaries

1. **Triage-First Pipeline**: Ingestion $\rightarrow$ LLM Classification/Triage $\rightarrow$ Confidence Scoring $\rightarrow$ Action Proposal $\rightarrow$ Human Gate / Auto-Execute (if threshold met).
2. **Quality Gates**:
   - **L0 Anti-Skip**: Artifact or action payload exists and conforms to schema.
   - **L1 Verification**: Semantic goal achieved against test harness.
   - **L1.5 pACS**: Confidence scoring across Faithfulness, Completeness, Logic ($\min(F, C, L) \ge 70$).
   - **L2 Adversarial Review**: Review for hallucination or Reddit TOS violations.
3. **Circuit Breaker**: Trips to `OPEN` if Reddit API returns repeated errors (401/403/429) or if LLM reasoning loops.

---

## 7. Code Change Protocol (CCP)

Before modifying any file in the workspace:
1. **Understand Intent**: Verify requirements against `docs/CONSTITUTION.md` and active work cards.
2. **Ripple Effect Analysis**: Identify all upstream/downstream call sites.
3. **Change Plan**: Draft minimal, bounded change preserving all existing invariants.
4. **Falsification**: Write failing test first, verify failure, implement fix, verify pass.

---

## 8. Compound Engineering Codification

Every resolved ambiguity, edge-case failure, or Reddit API quirk must be documented into `docs/solutions/` as a durable project learning to prevent recurring mistakes across sessions.
