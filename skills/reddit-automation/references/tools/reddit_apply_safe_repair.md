# Tool: reddit_apply_safe_repair

Apply safe automated repairs from a deterministic repair plan.

## Purpose & Scope
Executes the non-destructive remediation steps generated in a `RepairPlan` (e.g. executing wait cooldowns, requesting OAuth token refresh, updating post flairs, or patching account preferences). 

Fails safely if a step requires human intervention (e.g. interactive re-auth or manual captcha solving), returning a clear blocker description.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `repair_plan` | object | Yes | `RepairPlan` returned by `reddit_plan_repair`. |
| `account` | string | No | Account alias (optional). |

## Return Schema
```json
{
  "plan_id": "repair_plan_102938",
  "status": "completed",
  "steps_executed": [
    {
      "step_number": 1,
      "kind": "BackoffWait",
      "result": "success",
      "elapsed_seconds": 180
    },
    {
      "step_number": 2,
      "kind": "RecheckAccountHealth",
      "result": "success",
      "details": "Rate limit quota reset to 600 requests."
    }
  ],
  "is_repaired": true
}
```

## Agent Usage Pattern
Call after `reddit_plan_repair` when `is_fully_automated` is true. If false or halted, notify the user with the exact manual steps needed.

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "17",
  "method": "tools/call",
  "params": {
    "name": "reddit_apply_safe_repair",
    "arguments": {
      "account": "personal_dev",
      "repair_plan": {
        "plan_id": "repair_plan_102938",
        "is_fully_automated": true,
        "steps": [
          { "step_number": 1, "kind": "BackoffWait", "duration_seconds": 180 }
        ]
      }
    }
  }
}
```

## Safety Rules
- Halts immediately if an unexpected HTTP error is encountered.
- Never attempts unauthorized password resets or captcha bypasses.
