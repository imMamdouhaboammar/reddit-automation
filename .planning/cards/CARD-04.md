# CARD-04: Commander CLI Command Wiring

- **Card**: CLI command registry wiring agent workers
- **Why**: Expose user-facing CLI commands for discovery, reply drafting, community auditing, and brand monitoring with dry-run safety.
- **Depends on**: CARD-01, CARD-02, CARD-03
- **Owns**: `src/cli/`, `src/index.ts`
- **Must not change**: Agent worker business logic
- **Behavior/invariant**:
  - `--dry-run` enabled by default.
  - Pretty terminal output with structured formatting.
  - Clean error handling and exit codes.
  - Interactive or non-interactive flag support.
- **Implementation constraints**: Commander.js, ESM export, executable binary.
- **Acceptance evidence**: CLI executes `--help` and mock runs for each command successfully.
- **Rollback/recovery**: Revert `src/cli/` and `src/index.ts`.
- **Integration handoff**: CLI binary ready for distribution.
- **Parallel-safe with**: CARD-05.
