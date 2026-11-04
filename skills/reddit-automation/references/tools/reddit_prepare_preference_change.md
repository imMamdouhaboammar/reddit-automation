# Tool: reddit_prepare_preference_change

Prepare a two-stage ActionPlan to patch Reddit account preferences.

## Purpose & Scope
Generates a cryptographic `ActionPlan` (`act_pref_...`) containing a diff of the requested preference modifications. Enforces two-stage mutation: changes are never applied directly without human or supervisor approval via `reddit_approve_action`.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `account` | string | No | Account alias (defaults to active account). |
| `preferences` | object | Yes | Key-value map of preferences to update. |

## Return Schema
```json
{
  "action_id": "act_pref_e9a1b2c3",
  "account": "personal_dev",
  "kind": "patch_preferences",
  "risk_level": "medium",
  "status": "pending_approval",
  "diff": {
    "over_18": {
      "old": false,
      "new": true
    }
  },
  "content_hash": "a1b2c3d4e5f6...",
  "expires_at_utc": 1709459200
}
```

## Agent Usage Pattern
1. Call `reddit_prepare_preference_change` with desired patch.
2. Review the diff and SHA-256 hash.
3. Present confirmation to supervisor or execute `reddit_approve_action`.
4. Apply the change via safe repair or action dispatch.

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "5",
  "method": "tools/call",
  "params": {
    "name": "reddit_prepare_preference_change",
    "arguments": {
      "account": "personal_dev",
      "preferences": {
        "over_18": true
      }
    }
  }
}
```
