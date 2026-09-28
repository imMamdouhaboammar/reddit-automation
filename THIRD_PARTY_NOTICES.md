# Third-Party Notices and Acknowledgments

This document contains license, copyright, and provenance information for third-party libraries, dependencies, and architectural reference implementations used or referenced in `reddit-agent`.

---

## 1. Architectural & Research References

During the design and research phases of this project, several open-source Reddit automation and Model Context Protocol (MCP) implementations were analyzed for protocol behaviors, scope segmentation, and API edge cases. **No source code from these projects was copied or incorporated into this repository.**

| Project / Repository | Upstream Author / Owner | License | Relationship & Usage |
| :--- | :--- | :--- | :--- |
| [`quinnjr/reddit-mcp`](https://github.com/quinnjr/reddit-mcp) | Quinn | MIT | Behavioral & architectural reference only. Analyzed for Reddit OAuth callback patterns and scope segmentation. No source code incorporated. |
| [`daanrongen/reddit-mcp`](https://github.com/daanrongen/reddit-mcp) | Daan Rongen | MIT | Behavioral reference only. Analyzed for Reddit API query bounds and parameter validation patterns. No source code incorporated. |
| [`brettadams0/reddit-mcp`](https://github.com/brettadams0/reddit-mcp) | Brett Adams | MIT | Behavioral reference only. Analyzed for tool naming conventions and error-handling ergonomics in MCP servers. No source code incorporated. |
| [`roux`](https://github.com/ferristseng/roux) | Ferris Tseng | MIT | Evaluated crate. Inspected endpoint URI patterns; rejected direct adoption due to reliance on username/password auth and lack of driver abstraction. No source code incorporated. |

---

## 2. Direct Dependencies (Cargo Crates)

All direct dependencies are linked through standard Cargo dependency management; no third-party source files are vendored or copied into this repository.

| Package | Version | Declared License | Upstream Repository | Role & Usage |
| :--- | :--- | :--- | :--- | :--- |
| **`tokio`** | 1.53.1 | MIT | https://github.com/tokio-rs/tokio | Asynchronous runtime, task scheduling, timers |
| **`reqwest`** | 0.12.28 | MIT OR Apache-2.0 | https://github.com/seanmonstar/reqwest | HTTP client with rustls-tls and connection pooling |
| **`oauth2`** | 5.0.0 | MIT OR Apache-2.0 | https://github.com/ramosbugs/oauth2-rs | RFC 6749 OAuth 2.0 PKCE and token management |
| **`rusqlite`** | 0.32.1 | MIT | https://github.com/rusqlite/rusqlite | Embedded SQLite for accounts, pending actions, audit log |
| **`keyring`** | 3.6.3 | MIT OR Apache-2.0 | https://github.com/hwchen/keyring-rs | Native OS keychain integration for secure token isolation |
| **`chromiumoxide`** | 0.9.1 | MIT OR Apache-2.0 | https://github.com/mattsse/chromiumoxide | Async Chrome DevTools Protocol client for browser driver |
| **`clap`** | 4.6.7 | MIT OR Apache-2.0 | https://github.com/clap-rs/clap | Command-line argument parsing and help generation |
| **`serde`** | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde | Serialization/deserialization framework |
| **`serde_json`** | 1.0.151 | MIT OR Apache-2.0 | https://github.com/serde-rs/json | JSON serialization for Reddit API and MCP payloads |
| **`thiserror`** | 2.0.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror | Strongly typed domain error derivation |
| **`anyhow`** | 1.0.104 | MIT OR Apache-2.0 | https://github.com/dtolnay/anyhow | Flexible application error handling |
| **`tracing`** | 0.1.44 | MIT | https://github.com/tokio-rs/tracing | Structured diagnostic instrumentation |
| **`tracing-subscriber`**| 0.3.23 | MIT | https://github.com/tokio-rs/tracing | Structured log filtering and formatting |
| **`url`** | 2.5.8 | MIT OR Apache-2.0 | https://github.com/servo/rust-url | RFC 3986 URL parsing and validation |
| **`sha2`** | 0.10.9 | MIT OR Apache-2.0 | https://github.com/RustCrypto/hashes | SHA-256 cryptographic hashing for action plan integrity |
| **`hex`** | 0.4.3 | MIT OR Apache-2.0 | https://github.com/KokaKiwi/rust-hex | Hexadecimal encoding/decoding |
| **`chrono`** | 0.4.45 | MIT OR Apache-2.0 | https://github.com/chronotope/chrono | Date and time handling |
| **`uuid`** | 1.26.1 | Apache-2.0 OR MIT | https://github.com/uuid-rs/uuid | Universally unique identifiers (v4) |
| **`futures-util`** | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs | Asynchronous stream and future utilities |
| **`urlencoding`** | 2.1.3 | MIT | https://github.com/kornelski/rust_urlencoding | URL percent-encoding and decoding |
| **`async-trait`** | 0.1.92 | MIT OR Apache-2.0 | https://github.com/dtolnay/async-trait | Async trait method support |
| **`tempfile`** | 3.27.0 | MIT OR Apache-2.0 | https://github.com/Stebalien/tempfile | Dev-dependency: temporary file management in tests |
| **`wiremock`** | 0.6.5 | MIT/Apache-2.0 | https://github.com/LukeMathWalker/wiremock-rs | Dev-dependency: HTTP mock server for integration tests |

---

## 3. Direct Dependencies (Node / Bun Packages)

The TypeScript wrapper tools and tests utilize the following Node/Bun packages:

| Package | Version | Declared License | Upstream Repository | Role & Usage |
| :--- | :--- | :--- | :--- | :--- |
| **`commander`** | 13.1.0 | MIT | https://github.com/tj/commander.js | TypeScript CLI command definition |
| **`dotenv`** | 16.4.7 | BSD-2-Clause | https://github.com/motdotla/dotenv | Environment variable configuration loader |
| **`zod`** | 3.24.2 | MIT | https://github.com/colinhacks/zod | Schema validation and typing |
| **`tsup`** | 8.4.0 | MIT | https://github.com/egoist/tsup | Dev-dependency: TypeScript bundler |
| **`typescript`**| 5.8.2 | Apache-2.0 | https://github.com/microsoft/TypeScript | Dev-dependency: TypeScript language compiler |
| **`vitest`** | 3.0.7 | MIT | https://github.com/vitest-dev/vitest | Dev-dependency: Unit testing framework |

---

## 4. Transitive Dependencies & Audit Findings

A complete automated licensing audit of all 255 transitive dependencies resolved by `Cargo.lock` and `bun.lock` was performed:

- **MIT / Apache-2.0 Dual License:** 180 crates
- **MIT License:** 40 crates
- **Apache-2.0 License:** 15 crates
- **Unicode-3.0 / Unicode-DFS-2016:** 18 crates
- **BSD-2-Clause / BSD-3-Clause:** 3 crates
- **ISC License:** 3 crates
- **Unlicense / CC0-1.0:** 3 crates
- **Tri-Licensed Crates:** `r-efi` is tri-licensed under `MIT OR Apache-2.0 OR LGPL-2.1-or-later`. This project incorporates `r-efi` exclusively under the permissive `MIT OR Apache-2.0` terms.
- **Copyleft / SSPL Status:** Zero GPL, AGPL, or SSPL copyleft obligations are incurred by any dependency used in this software.

Third-party components remain subject to their respective licenses.
