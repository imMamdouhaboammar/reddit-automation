# Tool: reddit_accounts

List all configured Reddit account aliases with active default indicator and verified username.

## Purpose & Scope
Inspect the active local multi-account registry managed by `reddit-automation`. Identifies which accounts are configured, whether their credentials are valid, which one is currently selected as default, and their account type (`personal`, `brand`, `tester`).

Read-only operation. Does not perform network calls or mutate state.

## Parameters
None (`properties: {}`).

## Return Schema
```json
{
  "accounts": [
    {
      "alias": "personal_dev",
      "username": "CodeExplorer",
      "is_default": true,
      "account_type": "personal",
      "has_oauth": true,
      "has_browser_session": false
    },
    {
      "alias": "agency_brand",
      "username": "FlowKitOfficial",
      "is_default": false,
      "account_type": "brand",
      "has_oauth": true,
      "has_browser_session": true
    }
  ],
  "default_alias": "personal_dev"
}
```

## Agent Usage Pattern
Always run `reddit_accounts` before executing publishing or account-specific actions if the target `account` parameter is not explicitly provided in agent context.

```bash
# JSON-RPC MCP call
{
  "jsonrpc": "2.0",
  "id": "1",
  "method": "tools/call",
  "params": {
    "name": "reddit_accounts",
    "arguments": {}
  }
}
```

## Safety Rules
- Safe for autonomous invocation.
- Never reveals raw OAuth refresh tokens or passwords.
