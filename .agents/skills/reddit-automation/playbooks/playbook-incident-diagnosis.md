# Playbook: Incident Diagnosis, Shadowban & Challenge Recovery

> **Role:** Incident Diagnostician & Self-Healing SRE  
> **Primary Tools:** `reddit_diagnose_issue`, `reddit_plan_repair`, `reddit_apply_safe_repair`, `reddit_account_health`  
> **Core Invariant:** Never retry blind API calls on rejection. Always diagnose root cause first.

---

## 1. Incident Taxonomy

When an action or request fails, the `IssueEngine` classifies the root cause:

| Issue Code | Trigger Signature | Severity | Automation Action |
|---|---|---|---|
| `POST_REQUIREMENT_VIOLATION` | Missing flair, invalid title regex, body length violation | Low | Auto-remediate parameters against rules |
| `RATE_LIMIT_EXCEEDED` | HTTP 429, header `x-ratelimit-remaining: 0` | Medium | Jittered exponential backoff |
| `CAPTCHA_CHALLENGE` | Cloudflare turnstile, Reddit robot challenge | High | **Halt automation; require human interactive login** |
| `ACCOUNT_SHADOWBANNED` | Profile 404 in incognito, comments auto-filtered | Critical | Quarantine account; halt all activity |
| `AUTH_SCOPE_MISSING` | HTTP 403, missing OAuth scope | Medium | Trigger interactive OAuth refresh |

---

## 2. Diagnosis & Repair Workflow

```
   [ Error Encountered: Submission Rejected / 403 / 429 ]
                            │
                            ▼
   ┌────────────────────────────────────────────────────────┐
   │ 1. reddit_diagnose_issue(account, error, payload)      │
   │    Inspects account tokens, sub rules, and headers     │
   └────────────────────────┬───────────────────────────────┘
                            │
                            ▼
   ┌────────────────────────────────────────────────────────┐
   │ 2. reddit_plan_repair(diagnosis)                       │
   │    Generates deterministic RepairPlan                  │
   └────────────────────────┬───────────────────────────────┘
                            │
              ┌─────────────┴─────────────┐
              │                           │
         Safe Automated            Human Checkpoint
              │                           │
              ▼                           ▼
   ┌──────────────────────┐    ┌──────────────────────┐
   │ reddit_apply_safe_   │    │ HALT AUTOMATION!     │
   │ repair               │    │ Prompt human to run: │
   │ (e.g. patch prefs,   │    │ reddit-agent account │
   │  wait backoff)       │    │ login --alias <name> │
   └──────────────────────┘    └──────────────────────┘
```

---

## 3. Human Checkpoint Protocol (CAPTCHA / Security Checkpoint)

If the diagnostician reports `requires_human: true`:
1. **Never attempt to bypass CAPTCHA programmatically.** Doing so violates platform safety and triggers permanent IP bans.
2. The agent outputs the exact human command:
   ```bash
   reddit-agent account login --alias <alias>
   ```
3. A visible Chromium browser opens with the user's isolated profile.
4. The user completes the challenge, clicks save, and closes the browser.
5. The agent re-runs `reddit_account_health` to verify recovery before resuming work.
