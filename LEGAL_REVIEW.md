# Legal Review & Sanity Check

This document outlines key legal questions, structural considerations, and edge cases flagged for human review by the repository owner and legal counsel prior to public release.

---

## 1. Agency / Client Work under Commons Clause Condition v1.0

- **Context:** The Commons Clause restricts selling "a product or service whose value derives, entirely or substantially, from the functionality of the Software."
- **Clarification Added in FAQ:** In `LICENSE-FAQ.md`, we clarified that an agency or consultant may use the software internally as a tool while performing broader marketing or community consulting services for clients (including operating authorized Reddit accounts), provided they do not sell licenses, sell forks, charge primarily for access to hosted software instances, or offer the software itself as a commercial product.
- **Review Question for Counsel:** Is this non-binding explanatory FAQ guidance sufficient to provide commercial comfort to agency users without compromising the enforceable boundary against commercial repackagers?

---

## 2. Relicensing Scope in the Contributor License Agreement (`CLA.md`)

- **Context:** `CLA.md` implements an Apache-style, non-exclusive copyright and patent license from contributors to the Project Owner. It explicitly permits the Project Owner to distribute contributions under the public source-available license, commercial licenses, or future licensing models, while allowing contributors to retain copyright ownership of their original contributions.
- **Review Question for Counsel:** Does the license grant in Section 3 adequately preserve the owner's legal flexibility to grant proprietary commercial licenses and enterprise exceptions without requiring full copyright assignment from external community contributors?

---

## 3. Project Name & Trademark Protection

- **Context:** `TRADEMARKS.md` reserves the project names (`reddit-agent`, `reddit-agentic-cli`) and official visual identity, permitting only factual compatibility references ("compatible with...", "forked from...").
- **Review Question for Counsel:** Should the owner proceed with formal trademark registration for `reddit-agent` in key jurisdictions (e.g., USPTO / EUIPO) to prevent bad-faith third-party filings, given the commercial resale restrictions?

---

## 4. Jurisdiction & Governing Law for Commercial Licensing Agreements

- **Context:** `COMMERCIAL-LICENSE.md` directs commercial inquiries to a GitHub issue template to establish custom, bilaterally negotiated enterprise contracts.
- **Review Question for Counsel:** Which jurisdiction and governing law clause should be standardized as the default for separately negotiated commercial agreements (e.g., Delaware, England & Wales, or the owner's local jurisdiction)?
