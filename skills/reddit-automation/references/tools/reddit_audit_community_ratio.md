# Tool Reference: `reddit_audit_community_ratio`

> **Category:** Reputation & Account Health  
> **Host Requirements:** Configured account or karma breakdown  
> **Primary Use:** Audit account history against Reddit's 90/10 content-to-promotion ratio.

---

## 1. Description

Calculates the percentage of organic, value-adding contributions versus promotional submissions. If the value ratio falls below 90%, it returns a promotional freeze advisory to protect the account from moderation bans.

---

## 2. Input Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `account` | string | No | Account alias (defaults to active account) |
| `history` | array[object] | No | Optional explicit list of recent engagement records |

---

## 3. Example Request & Response

### Request:
```json
{
  "account": "personal"
}
```

### Response:
```json
{
  "account": "personal",
  "reddit_username": "founder_bob",
  "link_karma": 120,
  "comment_karma": 1840,
  "audit": {
    "total_engagements": 45,
    "value_add_count": 42,
    "promotional_count": 3,
    "value_percentage": 93.3,
    "is_compliant": true,
    "advisory": "Account ratio is healthy and compliant (93.3% value-add). Safe to engage."
  }
}
```

---

## 4. Agent Instructions
- If `is_compliant == false`: Stop all promotional drafts and suggest answering questions in pure-value mode.
