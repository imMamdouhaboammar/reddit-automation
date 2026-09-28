# Licensing FAQ

> **Notice:** This document provides practical, explanatory guidance regarding the intended interpretation of the governing license terms (`LICENSE`: Apache License 2.0 with the Commons Clause License Condition v1.0). This document is provided for convenience and does not replace, supersede, or modify the legal terms set forth in `LICENSE`. For any ambiguity or edge cases, please refer to `LEGAL_REVIEW.md` or contact the project owner.

---

## 1. What is the licensing model of this project?

This project is released as **source-available** software under the **Apache License 2.0** combined with the **Commons Clause License Condition v1.0**.

- **Classification:** Source Available · Community Developed · Commercial Resale Restricted.
- **Not OSI Open Source:** This software is not OSI-approved Open Source (FOSS). The Commons Clause adds a commercial resale restriction that is outside the Open Source Definition.
- **Core Principle:** The source code is visible, inspectable, forkable, modifiable, and freely usable for personal, internal business, and professional service purposes. However, third parties are not permitted to commercially sell the software, sell rebranded or modified forks, or package it into a paid managed clone whose primary value is this software.

---

## 2. What are the permitted uses?

You are freely permitted and encouraged to:
- **Personal Use:** Run, study, tinker with, and customize the tool for your own personal Reddit accounts and automation workflows.
- **Internal Business Use:** Install and run the software within your company, team, or organization for internal operations.
- **Study & Research:** Read the source code, inspect architecture decisions, and learn from the implementation.
- **Modify Code:** Modify the code locally or in your own environments to suit your specific internal requirements.
- **Integrate:** Connect your own applications, LLM pipelines, orchestration frameworks, and Model Context Protocol (MCP) clients to the tool.
- **Fork for Community Development:** Create public or private forks on GitHub to experiment, develop bug fixes, or build extensions.
- **Contribute:** Submit issues, documentation improvements, drivers, and pull requests under the project's Contributor License Agreement (`CLA.md`).

---

## 3. Can businesses and companies use this software?

**Yes.** Internal business use is fully permitted.

Examples of permitted business use:
- A company installs `reddit-agent` on employee workstations or internal servers to manage official company Reddit communications.
- A growth or marketing team uses the CLI or MCP server to monitor brand mentions, draft responses, and analyze community sentiment.
- An engineering team integrates `reddit-agent` into their internal CI/CD or workflow automation scripts.
- A company maintains a private internal fork with custom internal drivers tailored to their internal infrastructure.
- An AI development team connects their proprietary LLM agents to the `reddit-agent` MCP server.

**The Commons Clause restriction is about commercializing the software itself — not merely using the software while conducting legitimate business.**

---

## 4. How does this apply to agencies and consultants doing client work?

We recognize that agencies and independent consultants often operate tools on behalf of clients. 

**Intended Interpretation:**
- An agency or consultant **may** use `reddit-agent` internally as a productivity tool while providing broader marketing, community management, or social consulting services to clients (including operating client Reddit accounts that the agency is authorized to manage).
- An agency or consultant **may not**:
  - Sell licenses or sublicenses to this software to their clients or third parties.
  - Sell a modified or rebranded fork of this software.
  - Charge customers primarily for access to a hosted instance of this software (e.g., "Pay $99/mo for access to our Reddit automation dashboard" where the dashboard's substantial value is `reddit-agent`).
  - Package and market substantially the same software as their own standalone commercial Reddit automation product or service.

---

## 5. What is strictly forbidden under the public license?

Under the Commons Clause condition, you may not "Sell" the software. The following activities require a separate commercial license from the project owner:
- **Selling the software:** Charging a fee to download, license, or obtain the software or its compiled binaries.
- **Selling modified or rebranded forks:** Creating a fork (e.g., renaming the project or adding minor features) and selling it.
- **Commercial managed services:** Hosting the software and charging subscription fees where the substantial value provided is the software itself.
- **White-label resale:** Rebranding the software under your company name and distributing or selling it as your proprietary product.
- **Removing ownership notices:** Stripping copyright, NOTICE, or license headers from original project files.
- **Impersonating the official project:** Marketing a fork or derived service in a way that misleads users into believing it is the official or endorsed release.

---

## 6. What kind of forks are allowed?

- **Community & Development Forks:** You are welcome to fork the repository on GitHub, create feature branches, test changes, and submit pull requests.
- **Experimental Forks:** You can fork to experiment with new drivers, storage backends, or agent models.
- **Internal Corporate Forks:** A company may fork the repository privately to integrate with proprietary internal tools, provided the fork is never sold or redistributed as a commercial product.
- **Prohibited Forks:** Any fork created to circumvent the commercial restriction, rebrand the product, and sell it commercially.

---

## 7. Do I have to contribute my changes back (copyleft)?

**No.** This is not a copyleft license (like GPL or AGPL). You are not legally required to publish or contribute back changes you make for private or internal use. However, community contributions that benefit the wider ecosystem are warmly welcomed!

---

## 8. What if I want to build a commercial product powered by this tool?

If you want to incorporate `reddit-agent` into a commercial product, sell hosted access, or distribute a white-label solution, you will need a separate commercial license granted directly by the project owner.

Please see [`COMMERCIAL-LICENSE.md`](file:///Users/mamdouhaboammar/Documents/antigravity/happy-raman/COMMERCIAL-LICENSE.md) for details on requesting a commercial licensing agreement.
