# reddit-agent

[![Source Available](https://img.shields.io/badge/License-Source--Available-blue.svg)](file:///Users/mamdouhaboammar/Documents/antigravity/happy-raman/LICENSE)
[![Build Status](https://img.shields.io/badge/Rust-2021-orange.svg)](file:///Users/mamdouhaboammar/Documents/antigravity/happy-raman/Cargo.toml)
[![Runtime](https://img.shields.io/badge/Bun-1.1%2B-black.svg)](file:///Users/mamdouhaboammar/Documents/antigravity/happy-raman/package.json)

**Fast, local-first, multi-account Agentic Reddit Operator built in Rust with Model Context Protocol (MCP) support.**

Built in public · Community contributions welcome · Self-hosted · Local-first · Source Available · Commercial resale restricted.

---

## Important Notice: Source-Available Classification

> **This project is source-available software.**
> 
> **It is not OSI-approved Open Source software.**
> 
> The source may be viewed, used, modified, forked, and contributed to under the repository license, but commercial resale of the software itself or a substantially equivalent derived offering is restricted.

For full details on permitted personal and business uses, please review [`LICENSE`](file:///Users/mamdouhaboammar/Documents/antigravity/happy-raman/LICENSE) and [`LICENSE-FAQ.md`](file:///Users/mamdouhaboammar/Documents/antigravity/happy-raman/LICENSE-FAQ.md).

---

## Highlights

- **Multi-Account Isolation:** Register, manage, and switch between multiple Reddit profiles with strict data and browser separation.
- **Secure Keychain Storage:** Authentication tokens and refresh credentials are encrypted via native OS keychains (macOS Keychain, Linux Secret Service, Windows Credential Manager).
- **Hybrid Driver Architecture:**
  - **API Driver:** Ultra-fast async HTTP/OAuth2 driver with strict rate-limit backoff and jitter.
  - **Browser Driver:** Chrome DevTools Protocol (CDP) driver with isolated browser profiles for human-in-the-loop 2FA/login and Reddit UI interaction.
- **Fail-Closed Safety & Dry-Run by Default:** All mutating actions default to `--dry-run` and generate cryptographically signed (SHA-256) action plans requiring explicit approval before execution.
- **Autonomous Agent Workers:**
  - **Opportunity Hunter:** Scans subreddits for relevant discussions, computes opportunity scores ($0-100$), and drafts disclosure-compliant responses.
  - **Community Builder:** Evaluates subreddit engagement against healthy community ratios (90/10 content-to-promotion rule) and monitors sentiment.
  - **Issue Diagnostician:** Analyzes post rejections, shadowbans, and CAPTCHA/security challenges with actionable remediation steps.
- **Model Context Protocol (MCP) Server:** Connects directly to AI platforms like Claude Desktop, Cursor, and custom agentic frameworks over standard JSON-RPC.

---

## Architecture Overview

```
                      ┌────────────────────────────────────────┐
                      │             AI Client / LLM            │
                      │  (Claude Desktop, Cursor, Custom Agent)│
                      └──────────────────┬─────────────────────┘
                                         │ JSON-RPC (MCP)
                                         ▼
                      ┌────────────────────────────────────────┐
                      │           reddit-agent (MCP)           │
                      └──────────────────┬─────────────────────┘
                                         │
                 ┌───────────────────────┴───────────────────────┐
                 ▼                                               ▼
    ┌───────────────────────────┐                 ┌───────────────────────────┐
    │     Fast API Driver       │                 │   Isolated Browser Driver │
    │   (OAuth2 / Rustls / HTTP)│                 │   (CDP / Chrome Profiles) │
    └────────────┬──────────────┘                 └─────────────┬─────────────┘
                 │                                              │
                 ▼                                              ▼
    ┌─────────────────────────────────────────────────────────────────────────┐
    │                       Reddit Platform API & Web                         │
    └─────────────────────────────────────────────────────────────────────────┘
```

---

## Prerequisites

- **Rust:** 1.80+ (edition 2021)
- **Bun:** 1.1+ (for TypeScript CLI wrapper and tests)
- **Google Chrome / Chromium:** For the browser driver

---

## Quickstart

### 1. Build and Test

```bash
# Clone the repository
git clone https://github.com/imMamdouhaboammar/reddit-agent.git
cd reddit-agent

# Run tests
cargo test
bun test

# Build release binary
cargo build --release
```

### 2. Configure Environment

Copy the example environment file:

```bash
cp .env.example .env
```

Configure your Reddit API credentials and ensure `DRY_RUN=true` during initial testing.

### 3. Usage Examples

```bash
# List registered accounts
cargo run -- account list

# Scan for relevant opportunities in a subreddit (dry-run)
cargo run -- agent opportunity --subreddit rust --keywords "web framework"

# Start the Model Context Protocol (MCP) server
cargo run -- mcp
```

---

## Community & Contributions

Community contributions are welcome! You can report bugs, suggest features, optimize performance, or add new drivers.

- Read [`CONTRIBUTING.md`](file:///Users/mamdouhaboammar/Documents/antigravity/happy-raman/CONTRIBUTING.md) for local setup, linting, and testing instructions.
- All contributions are made under the [Contributor License Agreement (CLA.md)](file:///Users/mamdouhaboammar/Documents/antigravity/happy-raman/CLA.md). Contributors retain their copyright ownership while granting the project owner a non-exclusive license to maintain and distribute the work.
- Please review our [`CODE_OF_CONDUCT.md`](file:///Users/mamdouhaboammar/Documents/antigravity/happy-raman/CODE_OF_CONDUCT.md).

---

## Commercial Licensing

Organizations that wish to use `reddit-agent` beyond the permissions of the public source-available license (e.g., OEM bundling, white-label distribution, or offering a paid hosted managed service) can request a separate commercial license agreement.

Please see [`COMMERCIAL-LICENSE.md`](COMMERCIAL-LICENSE.md) or open an inquiry using the [Commercial Licensing Issue Template](https://github.com/imMamdouhaboammar/reddit-agent/issues/new?template=commercial-license.yml).

---

## Security

Security vulnerabilities and credential exposure risks should be disclosed privately. Please refer to [`SECURITY.md`](file:///Users/mamdouhaboammar/Documents/antigravity/happy-raman/SECURITY.md) for reporting procedures.

---

## Trademarks

The project name `reddit-agent`, `reddit-agentic-cli`, logos, and official release identities are protected marks. See [`TRADEMARKS.md`](file:///Users/mamdouhaboammar/Documents/antigravity/happy-raman/TRADEMARKS.md) for our trademark policy and permitted factual statements.

---

## License

This project is source-available under **Apache License 2.0 with the Commons Clause License Condition v1.0**.

You may use, study, modify, fork, and contribute to the software under those terms, including personal and internal business use.

Commercial resale, white-label resale, or offering substantially the same software as a paid product or managed service is restricted.

See:
- [`LICENSE`](file:///Users/mamdouhaboammar/Documents/antigravity/happy-raman/LICENSE)
- [`LICENSE-FAQ.md`](file:///Users/mamdouhaboammar/Documents/antigravity/happy-raman/LICENSE-FAQ.md)
- [`COMMERCIAL-LICENSE.md`](file:///Users/mamdouhaboammar/Documents/antigravity/happy-raman/COMMERCIAL-LICENSE.md)
- [`THIRD_PARTY_NOTICES.md`](file:///Users/mamdouhaboammar/Documents/antigravity/happy-raman/THIRD_PARTY_NOTICES.md)
