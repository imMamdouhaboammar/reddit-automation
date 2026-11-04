# Tool: reddit_edit_own_content

Edit the text of your own existing Reddit post or comment.

## Purpose & Scope
Updates the body markdown of content authored by the active account. Applies to:
- Self-posts (`t3_...`): edits the `selftext`.
- Comments (`t1_...`): edits the comment body.

Link posts (`kind: "link"`) cannot have their URLs edited.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `thing_id` | string | Yes | Fullname of content to edit (`t3_xxx` or `t1_xxx`). |
| `text` | string | Yes | New markdown body text. |
| `account` | string | No | Account alias that authored the content. |

## Return Schema
```json
{
  "thing_id": "t1_c1a2b3",
  "status": "updated",
  "edited_utc": 1735008000,
  "body": "Updated comment text with additional benchmark numbers."
}
```

## Agent Usage Pattern
Use when amending an earlier comment with corrections, clarifications, or requested disclosures.

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "12",
  "method": "tools/call",
  "params": {
    "name": "reddit_edit_own_content",
    "arguments": {
      "thing_id": "t1_c1a2b3",
      "text": "Clarification: we observed this specifically on Node 20. Full disclosure: I work on Acme telemetry."
    }
  }
}
```

## Safety Rules
- Mutating action.
- Validates that the active account is indeed the author of the target `thing_id`.
