# Contributing to reddit-agent

We welcome contributions from the community! Whether you are fixing a bug, adding a new driver, optimizing performance, or improving documentation, your help is appreciated.

`reddit-agent` is an owner-controlled, source-available project. We strive to maintain high engineering standards, fast local-first performance, and safe Reddit automation.

---

## Code of Conduct

All contributors and participants are expected to adhere to our [Code of Conduct](file:///Users/mamdouhaboammar/Documents/antigravity/happy-raman/CODE_OF_CONDUCT.md).

---

## Development Environment Setup

This project uses **Rust** (2021 edition) for core runtime systems and drivers, and **Bun** for TypeScript CLI wrapper tools and unit tests.

### Prerequisites

- **Rust toolchain:** Rust 1.80+ (`cargo`, `rustc`, `rustfmt`, `clippy`).
- **Bun runtime:** Bun 1.1+ (used for TS tests and package execution).
- **Chrome / Chromium:** Required for the browser-based driver.

### Quick Verification

Clone the repository and verify your local build:

```bash
# Build Rust crate
cargo build

# Run Rust unit and integration tests
cargo test

# Run TypeScript wrapper tests
bun test

# Run Rust linter and formatting checks
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

---

## Coding Guidelines

1. **Safety First:**
   - **Never log or persist sensitive credentials** (OAuth tokens, refresh tokens, passwords, cookies).
   - Ensure `dry-run` protections are preserved: no live mutations should occur unless explicitly confirmed.
2. **Deterministic Architecture:**
   - Keep drivers modular and testable (`src/drivers/`).
   - Use strongly typed domain errors (`thiserror`).
   - Validate all API inputs and responses using strict models.
3. **Tests Required:**
   - Any new feature or bug fix must include corresponding automated tests (`cargo test` / `bun test`).
   - Red-green verification: verify tests fail without your change and pass with it.

---

## Pull Request Process

1. **Fork & Branch:**
   - Fork the repository on GitHub.
   - Create a dedicated feature branch (`git checkout -b feat/your-feature-name` or `fix/issue-description`).
2. **Format & Lint:**
   - Run `cargo fmt` to format Rust files.
   - Run `cargo clippy --all-targets` to catch lint issues.
   - Run `cargo test` and `bun test` to ensure all tests pass.
3. **Commit Messages:**
   - Follow [Conventional Commits](https://www.conventionalcommits.org/) (e.g., `feat:`, `fix:`, `docs:`, `chore:`).
4. **Submit PR:**
   - Open a Pull Request against the `master` branch.
   - Provide a clear description of the problem solved and test results.
   - Confirm the CLA acceptance checkbox in the PR template.

---

## Contributor License Agreement (CLA)

To ensure the project can be safely maintained, distributed, and dual-licensed for enterprise users while honoring community contributions:

> **Important Licensing Notice:**
>
> Submitting a pull request does not transfer ownership of the contributor's copyright unless expressly agreed.
>
> Contributions are provided under the [Contributor License Agreement (CLA.md)](file:///Users/mamdouhaboammar/Documents/antigravity/happy-raman/CLA.md) and may be included in both publicly licensed and commercially licensed versions of the project.

When you open a PR, simply verify and check the CLA acknowledgment box in the PR description template.

---

## Security Vulnerabilities

Please **do not** open public GitHub issues for security vulnerabilities or credential disclosures. Follow the instructions in [`SECURITY.md`](file:///Users/mamdouhaboammar/Documents/antigravity/happy-raman/SECURITY.md) to report security issues privately.
