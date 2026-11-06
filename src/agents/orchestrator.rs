use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::agents::opportunity::ProductContext;
use crate::agents::router::{DynamicRouter, IntentCategory};
use crate::agents::tool_agents::{
    DiggerAgent, DrafterAgent, MinerAgent, OrchestrationContext, ToolAgent,
};

/// Structured report returned by the Master Orchestrator upon completing a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationReport {
    pub workflow_id: String,
    pub intent: IntentCategory,
    pub status: String,
    pub steps_completed: usize,
    pub total_steps: usize,
    pub context_summary: String,
    pub execution_log: Vec<String>,
    pub result: serde_json::Value,
}

/// Master Orchestrator governing multi-agent workflow execution.
pub struct MasterOrchestrator {
    digger_agent: Arc<DiggerAgent>,
    miner_agent: Arc<MinerAgent>,
    drafter_agent: Arc<DrafterAgent>,
}

impl MasterOrchestrator {
    pub fn new() -> Self {
        Self {
            digger_agent: Arc::new(DiggerAgent::new()),
            miner_agent: Arc::new(MinerAgent),
            drafter_agent: Arc::new(DrafterAgent),
        }
    }

    /// Orchestrates an end-to-end execution directly from natural language prompt.
    pub async fn orchestrate_from_prompt(
        &self,
        prompt: &str,
        product_opt: Option<ProductContext>,
    ) -> Result<OrchestrationReport, String> {
        let workflow_id = format!("wf_{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let mut context = OrchestrationContext {
            user_prompt: prompt.to_string(),
            product: product_opt,
            ..Default::default()
        };

        // 1. Dynamic Routing
        let target_ctx = context.product.as_ref().map(|p| {
            serde_json::json!({
                "product_name": p.product_name,
                "subreddits": p.target_subreddits
            })
        });

        let dag = DynamicRouter::build_execution_dag(prompt, target_ctx);
        context.execution_log.push(format!(
            "[MasterOrchestrator] Dynamic routing classified intent: {:?} (Confidence: {:.2}, Lang: {})",
            dag.intent_category, dag.confidence_score, dag.detected_language
        ));

        // 2. Wave-by-wave execution
        let mut steps_completed = 0;
        let total_steps = dag.steps.len();

        for wave in &dag.execution_waves {
            context.execution_log.push(format!(
                "[MasterOrchestrator] Executing Wave {} with steps: {:?}",
                wave.wave_index, wave.step_ids
            ));

            for step_id in &wave.step_ids {
                let step = dag
                    .steps
                    .iter()
                    .find(|s| &s.step_id == step_id)
                    .ok_or_else(|| format!("Step ID {} not found in DAG", step_id))?;

                // Check human approval gate
                if step.requires_human_approval {
                    context.execution_log.push(format!(
                        "[MasterOrchestrator] Checkpoint gate hit at step '{}' (Action requires human approval).",
                        step.step_id
                    ));
                    return Ok(OrchestrationReport {
                        workflow_id,
                        intent: dag.intent_category,
                        status: "pending_human_approval".to_string(),
                        steps_completed,
                        total_steps,
                        context_summary: "Workflow paused at cryptographic action approval gate."
                            .to_string(),
                        execution_log: context.execution_log,
                        result: serde_json::json!({
                            "paused_at_step": step.step_id,
                            "reason": "Mutation requires human verification."
                        }),
                    });
                }

                // Dispatch to specialized Tool-Agent
                let res = match step.agent_role.as_str() {
                    "DiggerAgent" => self.digger_agent.execute(&mut context).await?,
                    "MinerAgent" => self.miner_agent.execute(&mut context).await?,
                    "DrafterAgent" => self.drafter_agent.execute(&mut context).await?,
                    _ => {
                        format!("Delegated to {}", step.agent_role)
                    }
                };

                steps_completed += 1;
                context.execution_log.push(format!(
                    "[MasterOrchestrator] Completed step '{}': {}",
                    step.step_id, res
                ));
            }
        }

        // 3. Assemble Final Deliverable
        let deliverable = match dag.intent_category {
            IntentCategory::OpportunityHunting => serde_json::json!({
                "sources_discovered": context.digger_result.as_ref().map(|d| d.total_sources).unwrap_or(0),
                "opportunities_mined": context.miner_result.as_ref().map(|m| m.opportunities.len()).unwrap_or(0),
                "drafts_generated": context.drafts.len(),
                "drafts": context.drafts,
                "top_opportunities": context.miner_result.as_ref().map(|m| &m.opportunities).unwrap_or(&vec![]),
            }),
            _ => serde_json::json!({
                "intent": dag.intent_category,
                "summary": "Workflow completed successfully."
            }),
        };

        Ok(OrchestrationReport {
            workflow_id,
            intent: dag.intent_category,
            status: "completed".to_string(),
            steps_completed,
            total_steps,
            context_summary: format!(
                "Successfully orchestrated {} tool-agents across {} waves.",
                dag.steps.len(),
                dag.execution_waves.len()
            ),
            execution_log: context.execution_log,
            result: deliverable,
        })
    }
}

impl Default for MasterOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_initialization_and_route() {
        let orchestrator = MasterOrchestrator::new();
        let report = orchestrator
            .orchestrate_from_prompt("Find customer leads on Reddit", None)
            .await
            .expect("Orchestration failed");

        assert_eq!(report.intent, IntentCategory::OpportunityHunting);
        assert_eq!(report.status, "completed");
        assert_eq!(report.steps_completed, 3);
        assert_eq!(report.total_steps, 3);
        assert!(!report.execution_log.is_empty());
    }

    #[tokio::test]
    async fn test_orchestrator_publishing_checkpoint() {
        let orchestrator = MasterOrchestrator::new();
        let report = orchestrator
            .orchestrate_from_prompt("Publish a post to Reddit", None)
            .await
            .expect("Orchestration failed");

        assert_eq!(report.intent, IntentCategory::ContentPublishing);
        assert_eq!(report.status, "pending_human_approval");
        assert_eq!(report.steps_completed, 1);
    }
}
