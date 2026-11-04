# Tool Reference: `reddit_prepare_comment`

> **Category:** Two-Stage Publishing & Action Plans  
> **Risk Level:** Mutating (Dry-run by default)  
> **Primary Use:** Validate and prepare a two-stage ActionPlan to reply to a thread or comment.

---

## 1. Description

Validates comment markdown formatting and length boundaries, prevents duplicate identical comments, and creates a signed `ActionPlan` with status `pending_approval`.

---

## 2. Input Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `account` | string | No | Account alias (defaults to active account) |
| `thing_id` | string | **Yes** | Fullname of parent post (`t3_xxx`) or comment (`t1_xxx`) |
| `text` | string | **Yes** | Markdown body of the comment |

---

## 3. Example Request & Response

### Request:
```json
{
  "account": "personal",
  "thing_id": "t3_1h3xyz",
  "text": "I ran into this exact issue when working on..."
}
```

### Response:
```json
{
  "action_id": "act_comment_5544",
  "hash": "4b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a",
  "status": "pending_approval",
  "preview": "Account: personal\nAction: CreateComment\nTarget: t3_1h3xyz\nDetails: 48 words",
  "risk_level": "Medium"
}
```

---

## 4. Agent Instructions
- Present the text and preview to the user.
- Await approval before calling `reddit_approve_action`.
