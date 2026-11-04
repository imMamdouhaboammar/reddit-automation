# Tool: reddit_plan_repair

Generate a deterministic remediation plan for a diagnosed Reddit issue.

## Purpose & Scope
Takes a `DiagnosisReport` produced by `reddit_diagnose_issue` and generates a step-by-step `RepairPlan`. 
Distinguishes between:
- **Safe automated repairs**: Exponential backoff, token refresh, flair lookup & attachment, formatting adjustment.
- **Human checkpoints**: Re-authenticating credentials, resolving CAPTCHAs, appealing moderator shadowbans.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `diagnosis` | object | Yes | `DiagnosisReport` returned by `reddit_diagnose_issue`. |

## Return Schema
```json
{
  "plan_id": "repair_plan_102938",
  "diagnosis_id": "diag_8a7b6c5d",
  "is_fully_automated": true,
  "steps": [
    {
      "step_number": 1,
      "kind": "BackoffWait",
      "duration_seconds": 180,
      "description": "Wait out Reddit rate limit cooldown window."
    },
    {
      "step_number": 2,
      "kind": "RecheckAccountHealth",
      "description": "Verify rate-limit headers reset to > 0 requests."
    }
  ],
  "human_checkpoint_required": false
}
```

## Agent Usage Pattern
Pass the output of `reddit_diagnose_issue` directly into `reddit_plan_repair`. If `human_checkpoint_required` is `false`, proceed to `reddit_apply_safe_repair`.

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "16",
  "method": "tools/call",
  "params": {
    "name": "reddit_plan_repair",
    "arguments": {
      "diagnosis": {
        "diagnosis_id": "diag_8a7b6c5d",
        "category": "RateLimitExceeded",
        "evidence": { "retry_after_seconds": 180 }
      }
    }
  }
}
```
