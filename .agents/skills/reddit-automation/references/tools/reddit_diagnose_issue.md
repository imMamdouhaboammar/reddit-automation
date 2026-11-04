# Tool: reddit_diagnose_issue

Run diagnostic engine on an account or submission failure to identify root cause and evidence.

## Purpose & Scope
Diagnoses errors encountered during Reddit operations using deterministic error-tree pattern matching. Analyzes:
- Reddit API error codes (e.g. `RATELIMIT`, `SUBREDDIT_NOTALLOWED`, `NO_USER`, `INVALID_GRANT`).
- HTTP status codes (401, 403, 429, 503).
- Missing OAuth scopes or expired refresh tokens.
- Flairs and posting requirements violations.

Returns a structured `DiagnosisReport`.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `account` | string | No | Account alias (optional). |
| `subreddit` | string | No | Subreddit where the failure occurred. |
| `error_message` | string | No | Error string returned by the failing call. |
| `payload` | object | No | Rejected payload object. |

## Return Schema
```json
{
  "diagnosis_id": "diag_8a7b6c5d",
  "category": "RateLimitExceeded",
  "root_cause": "Reddit client rate limit reached. Backoff required for 180 seconds.",
  "confidence": 0.98,
  "evidence": {
    "raw_error": "RATELIMIT: 'You are doing that too much. Try again in 3 minutes.'",
    "retry_after_seconds": 180
  },
  "recommended_action": "Execute exponential backoff wait before retrying."
}
```

## Agent Usage Pattern
When any MCP call returns an error, catch the error and pass it to `reddit_diagnose_issue`. Then feed the resulting report into `reddit_plan_repair`.

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "15",
  "method": "tools/call",
  "params": {
    "name": "reddit_diagnose_issue",
    "arguments": {
      "account": "personal_dev",
      "subreddit": "ExperiencedDevs",
      "error_message": "SUBREDDIT_NOFLAIR: 'A flair is required for posts in this subreddit.'"
    }
  }
}
```
