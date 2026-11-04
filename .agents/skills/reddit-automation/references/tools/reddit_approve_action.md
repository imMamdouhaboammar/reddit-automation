# Tool Reference: `reddit_approve_action`

> **Category:** Safety & Approval Gate  
> **Risk Level:** State transition  
> **Primary Use:** Approve a pending ActionPlan before execution.

---

## 1. Description

Transitions an `ActionPlan` from `pending_approval` to `approved` state. Confirms the cryptographic integrity of the action hash and marks it eligible for execution.

---

## 2. Input Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `action_id` | string | **Yes** | ActionPlan identifier (e.g. `"act_8a7b6c5d4e3f"`) |

---

## 3. Example Request & Response

### Request:
```json
{
  "action_id": "act_8a7b6c5d4e3f"
}
```

### Response:
```json
{
  "action_id": "act_8a7b6c5d4e3f",
  "status": "approved",
  "message": "Action plan approved successfully. Ready for execution."
}
```

---

## 4. Agent Instructions
- Only invoke when the human user or workflow policy has explicitly consented to publishing.
- Follow immediately with `reddit_publish_prepared_post` or `reddit_publish_prepared_comment`.
