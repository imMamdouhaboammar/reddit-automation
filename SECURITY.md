# Security Policy

Security and credential isolation are core tenets of `reddit-agent`. We appreciate responsible disclosure of security issues and handle reports with high urgency.

---

## Supported Versions

Only the latest release branch of `reddit-agent` receives active security fixes.

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1.0 | :x:                |

---

## Reporting a Vulnerability

If you discover a security vulnerability or potential credential leak mechanism in this repository:

1. **Do NOT open a public GitHub issue.**
2. **Use GitHub Private Vulnerability Reporting:**
   - Navigate to the **Security** tab of the official GitHub repository.
   - Click **Report a vulnerability** to submit a private security advisory directly to the maintainers.
3. If Private Vulnerability Reporting is unavailable, open a minimal issue requesting private contact instructions without disclosing vulnerability details.

### What to Include in Your Report

- Description of the issue and potential security impact.
- Steps to reproduce the issue (including sample commands, inputs, or proof of concept).
- Any proposed remediation or patch.

We will acknowledge receipt within 48 hours and coordinate a coordinated disclosure schedule.

---

## What NEVER to Disclose

When reporting bugs, troubleshooting issues, or submitting logs in public channels, **NEVER** expose:

- Reddit API Client Secrets or IDs
- OAuth Access Tokens or Refresh Tokens
- Reddit Account Passwords
- Session cookies or auth cookies
- Browser user-data profile directories or local SQLite database dumps
- Any real credentials or authorization tokens

Any public issues or PRs containing real secrets will be immediately edited or removed and the affected credentials should be considered compromised and rotated immediately.
