# Tool Reference: `reddit_prepare_post`

> **Category:** Two-Stage Publishing & Action Plans  
> **Risk Level:** Mutating (Dry-run by default)  
> **Primary Use:** Validate and prepare an ActionPlan to submit a new Reddit post with a cryptographic SHA-256 hash.

---

## 1. Description

Validates a submission against community guidelines and post requirements (title regex, body length, required flairs), checks for duplicates, and creates a signed, tamper-proof `ActionPlan` in local SQLite storage. Does NOT publish to Reddit until approved.

---

## 2. Input Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `account` | string | No | Account alias (defaults to active account) |
| `subreddit` | string | **Yes** | Target subreddit (without `r/` prefix) |
| `title` | string | **Yes** | Post title |
| `text` | string | No | Selftext markdown body (for text posts) |
| `url` | string | No | URL (for link posts) |
| `kind` | string | No | Post kind: `"self"` or `"link"` (default: `"self"`) |
| `flair_id` | string | No | Flair template UUID |
| `nsfw` | boolean | No | NSFW flag |
| `spoiler` | boolean | No | Spoiler flag |

---

## 3. Example Request & Response

### Request:
```json
{
  "account": "personal",
  "subreddit": "SaaS",
  "title": "Lessons learned scaling local-first agentic operators",
  "text": "Over the past 6 months we experimented with local SQLite...",
  "kind": "self"
}
```

### Response:
```json
{
  "action_id": "act_8a7b6c5d4e3f",
  "hash": "7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069",
  "status": "pending_approval",
  "preview": "Account: personal\nAction: CreatePost\nTarget: r/SaaS\nTitle: Lessons learned scaling...",
  "risk_level": "Medium"
}
```

---

## 4. Agent Instructions
- Present the `action_id` and `preview` to the user.
- Await confirmation before invoking `reddit_approve_action`.
