# Tool Reference: `reddit_publish_prepared_post`

> **Category:** Execution & Publishing  
> **Risk Level:** High (Network Mutation)  
> **Primary Use:** Execute an approved post ActionPlan through Reddit's OAuth API.

---

## 1. Description

Validates that the `ActionPlan` is in `approved` status, verifies the cryptographic hash against the stored payload to detect tampering, and publishes the submission via Reddit's OAuth2 API. Records the published URL and logs an audit record in local SQLite.

---

## 2. Input Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `action_id` | string | **Yes** | Approved ActionPlan identifier (e.g. `"act_8a7b6c5d4e3f"`) |

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
  "status": "executed",
  "post_id": "t3_1k8abc",
  "url": "https://reddit.com/r/SaaS/comments/1k8abc/lessons_learned_scaling/",
  "published_at_utc": 1700000100
}
```

---

## 4. Error Semantics
- If status is `pending_approval`: Fails with `ActionNotApproved`.
- If payload was modified: Fails closed with `ActionTampered`.
- If already executed: Fails closed with `ActionAlreadyExecuted`.
