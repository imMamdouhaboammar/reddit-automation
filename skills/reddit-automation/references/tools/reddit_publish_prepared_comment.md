# Tool Reference: `reddit_publish_prepared_comment`

> **Category:** Execution & Publishing  
> **Risk Level:** High (Network Mutation)  
> **Primary Use:** Execute an approved comment ActionPlan to publish the reply on Reddit.

---

## 1. Description

Publishes an approved comment to Reddit's `/api/comment` endpoint. Verifies cryptographic hash integrity, applies rate limit backoff headers, and records the comment fullname in the local audit log.

---

## 2. Input Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `action_id` | string | **Yes** | Approved ActionPlan identifier (e.g. `"act_comment_5544"`) |

---

## 3. Example Request & Response

### Request:
```json
{
  "action_id": "act_comment_5544"
}
```

### Response:
```json
{
  "action_id": "act_comment_5544",
  "status": "executed",
  "comment_id": "t1_987abc",
  "parent_id": "t3_1h3xyz",
  "url": "https://reddit.com/r/SaaS/comments/1h3xyz/comment/987abc/",
  "published_at_utc": 1700000150
}
```

---

## 4. Error Semantics
- If unapproved: Fails with `ActionNotApproved`.
- If tampered: Fails closed with `ActionTampered`.
