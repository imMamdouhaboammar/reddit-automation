# Playbook: Cryptographic Two-Stage Action Publishing

> **Role:** Operations & Execution Gatekeeper  
> **Primary Tools:** `reddit_prepare_comment`, `reddit_approve_action`, `reddit_publish_prepared_comment`  
> **Core Invariant:** All mutating actions default to dry-run. No network mutation occurs without a cryptographically signed approval.

---

## 1. Safety Architecture

To eliminate runaway bot loops, unauthorized spam, and compromised token misuse, `reddit-agent` enforces a two-stage dry-run pipeline backed by SHA-256 signatures:

$$\text{ActionHash} = \text{SHA256}(\text{account} + \text{action\_type} + \text{target\_id} + \text{payload} + \text{created\_at})$$

```
   [ Agent / LLM ]
         │
         ▼ 1. Prepare (Dry-Run)
   ┌────────────────────────────────────────────────────────┐
   │ reddit_prepare_comment(thing_id, text)                 │
   │ -> Validates post requirements & length                │
   │ -> Computes SHA-256 ActionHash                         │
   │ -> Stores status: PendingApproval in local SQLite      │
   └────────────────────────┬───────────────────────────────┘
                            │
                            ▼ 2. Human / Supervisor Gate
   ┌────────────────────────────────────────────────────────┐
   │ reddit_approve_action(action_id)                       │
   │ -> Verifies hash integrity                             │
   │ -> Requires explicit human or policy approval          │
   │ -> Updates status: Approved                            │
   └────────────────────────┬───────────────────────────────┘
                            │
                            ▼ 3. Authenticated Execution
   ┌────────────────────────────────────────────────────────┐
   │ reddit_publish_prepared_comment(action_id)             │
   │ -> Fails closed if ActionHash was tampered with        │
   │ -> Fails closed if status != Approved                  │
   │ -> Dispatches to Reddit OAuth API with exponential     │
   │    jittered backoff                                    │
   │ -> Records AuditLog entry in local SQLite              │
   └────────────────────────────────────────────────────────┘
```

---

## 2. Step-by-Step Execution Guide

### Step 1: Prepare the Action Plan
Call `reddit_prepare_comment`:
```json
{
  "account": "personal",
  "thing_id": "t3_1h3xyz",
  "text": "I ran into this exact issue when working on..."
}
```
**Response received:**
```json
{
  "action_id": "act_9f8e7d6c5b4a",
  "hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "status": "pending_approval",
  "preview": "Account: personal\nAction: CreateComment\nTarget: t3_1h3xyz\nDetails: 53 words",
  "risk_level": "Medium"
}
```

### Step 2: Present Preview to Human / Supervisor
Display the exact preview and word count to the user:
```markdown
> [!IMPORTANT]
> **Action Pending Approval:** `act_9f8e7d6c5b4a`  
> **Target:** `r/SaaS` post `t3_1h3xyz`  
> **Text Preview:** "I ran into this exact issue..." (53 words)  
> **Cryptographic Signature:** `e3b0c4...`
```

### Step 3: Approve Action Plan
Once confirmed, call `reddit_approve_action`:
```json
{
  "action_id": "act_9f8e7d6c5b4a"
}
```

### Step 4: Execute & Publish
Call `reddit_publish_prepared_comment`:
```json
{
  "action_id": "act_9f8e7d6c5b4a"
}
```

---

## 3. Failure Semantics & Tamper Resistance

1. **Re-execution Prevention:** Once an action reaches `executed`, re-executing it immediately fails closed.
2. **Payload Tampering Detection:** If any field (`thing_id`, `text`, `account`) is altered in storage between preparation and execution, the SHA-256 hash check fails with `ActionTampered` and the action is revoked.
3. **Account Isolation:** An action prepared under account alias `alice` cannot be executed by account alias `bob`.
