# Tool: reddit_agent_orchestrate

Master OmniSkill Orchestrator: autonomously executes a natural language goal wave-by-wave by dispatching specialized Tool-Agents with fail-closed safety checkpoints.

## Purpose & Scope
Governs multi-agent execution across the Reddit automation ecosystem. Takes any user goal, routes it via `DynamicRouter`, and coordinates autonomous `ToolAgent` personas (`DiggerAgent`, `MinerAgent`, `DrafterAgent`, `PublisherAgent`, `AuditorAgent`, `DiagnosticianAgent`) to completion. Pauses safely when mutating steps require cryptographic approval.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `prompt` | string | Yes | Natural language goal or instruction to orchestrate. |
| `product` | object | No | Optional `ProductContext` specifying product name, value prop, ICP, target subreddits, etc. |

## Return Schema (`OrchestrationReport`)
```json
{
  "workflow_id": "wf_5b88ab8e",
  "intent": "OpportunityHunting",
  "status": "completed",
  "steps_completed": 3,
  "total_steps": 3,
  "context_summary": "Successfully orchestrated 3 tool-agents across 3 waves.",
  "execution_log": [
    "[MasterOrchestrator] Dynamic routing classified intent: OpportunityHunting (Confidence: 0.92, Lang: en)",
    "[MasterOrchestrator] Executing Wave 0 with steps: [\"step_dig_sources\"]",
    "[DiggerAgent] Mapping target subreddits: r/SaaS",
    "[DiggerAgent] Aggregated 6 deep candidate source threads with canonical URLs.",
    "[MasterOrchestrator] Completed step 'step_dig_sources': Discovered 6 sources",
    "[MasterOrchestrator] Executing Wave 1 with steps: [\"step_mine_content\"]",
    "[MinerAgent] Identified 3 high-intent opportunities.",
    "[MasterOrchestrator] Executing Wave 2 with steps: [\"step_draft_response\"]",
    "[DrafterAgent] Generated 3 verified Experience Grammar replies."
  ],
  "result": {
    "sources_discovered": 6,
    "opportunities_mined": 3,
    "drafts_generated": 3,
    "drafts": [],
    "top_opportunities": []
  }
}
```

## Native CLI Command
```bash
cargo run --bin reddit-agent -- orchestrate "Find SaaS opportunities" --product "FlowKit" --subreddits "SaaS"
```
