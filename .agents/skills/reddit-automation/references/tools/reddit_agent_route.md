# Tool: reddit_agent_route

Dynamic OmniSkill Agent Router: analyzes natural language prompts and compiles an optimal multi-step Directed Acyclic Graph (DAG).

## Purpose & Scope
Implements OmniSkill v6.2.0 Dynamic Intent Routing. Analyzes any prompt (English, Arabic, or mixed), extracts intent class, confidence level, and compiles an ordered DAG broken into sequential dependency waves with explicit human-approval gates for mutating actions.

## Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `prompt` | string | Yes | Natural language request, instruction, or goal in any language. |
| `context` | object | No | Optional context map (e.g. `product_name`, `subreddits`, `thing_id`). |

## Return Schema (`ExecutionDag`)
```json
{
  "intent_category": "OpportunityHunting",
  "user_prompt": "ابحث لي عن فرص تسويقية لخدمتي في ريديت",
  "detected_language": "ar",
  "confidence_score": 0.92,
  "steps": [
    {
      "step_id": "step_dig_sources",
      "agent_role": "DiggerAgent",
      "tool_name": "reddit_digger",
      "description": "Determine candidate subreddits and deeply aggregate source threads",
      "dependencies": [],
      "requires_human_approval": false,
      "parameters": {
        "product_name": "TargetProduct",
        "target_subreddits": ["SaaS", "startups"],
        "limit_per_sub": 10
      }
    },
    {
      "step_id": "step_mine_content",
      "agent_role": "MinerAgent",
      "tool_name": "reddit_miner",
      "description": "Sanitize prompt injection, classify intent signals, and rank opportunities",
      "dependencies": ["step_dig_sources"],
      "requires_human_approval": false,
      "parameters": {}
    },
    {
      "step_id": "step_draft_response",
      "agent_role": "DrafterAgent",
      "tool_name": "reddit_draft_response",
      "description": "Formulate 25-55 word Experience Grammar reply with mandatory disclosure",
      "dependencies": ["step_mine_content"],
      "requires_human_approval": false,
      "parameters": {}
    }
  ],
  "execution_waves": [
    {
      "wave_index": 0,
      "step_ids": ["step_dig_sources"]
    },
    {
      "wave_index": 1,
      "step_ids": ["step_mine_content"]
    },
    {
      "wave_index": 2,
      "step_ids": ["step_draft_response"]
    }
  ],
  "safety_notes": [
    "Multi-tier keyless scraping preserves anonymity.",
    "Replies strictly enforced within 25-55 words with anti-slop rules."
  ]
}
```

## Native CLI Command
```bash
cargo run --bin reddit-agent -- route "Find SaaS opportunities" --format text
```
