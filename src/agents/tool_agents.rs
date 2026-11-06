use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::agents::opportunity::{
    DiggerResult, MinerResult, OpportunityDraft, OpportunityWorker, ProductContext,
};
use crate::drivers::reddit_keyless::RedditKeylessDriver;

/// Shared blackboard state passed across Tool-Agents during orchestration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrchestrationContext {
    pub user_prompt: String,
    pub product: Option<ProductContext>,
    pub digger_result: Option<DiggerResult>,
    pub miner_result: Option<MinerResult>,
    pub drafts: Vec<OpportunityDraft>,
    pub generated_skill_name: Option<String>,
    pub pending_action_id: Option<String>,
    pub audit_summary: Option<String>,
    pub diagnostic_summary: Option<String>,
    pub execution_log: Vec<String>,
}

/// Abstract contract for a specialized Tool-Agent.
#[async_trait]
pub trait ToolAgent: Send + Sync {
    /// Agent persona name.
    fn name(&self) -> &'static str;
    /// Operational role description.
    fn role(&self) -> &'static str;
    /// Primary tool governed by this agent.
    fn primary_tool(&self) -> &'static str;
    /// Executes the agent's bounded responsibility on the orchestration context.
    async fn execute(&self, ctx: &mut OrchestrationContext) -> Result<String, String>;
}

/// Tool-Agent: DiggerAgent (owns candidate discovery & source mapping)
pub struct DiggerAgent {
    driver: Arc<RedditKeylessDriver>,
}

impl DiggerAgent {
    pub fn new() -> Self {
        Self {
            driver: Arc::new(RedditKeylessDriver::new()),
        }
    }
}

impl Default for DiggerAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolAgent for DiggerAgent {
    fn name(&self) -> &'static str {
        "DiggerAgent"
    }
    fn role(&self) -> &'static str {
        "Deep Candidate Source Mapping & Retrieval Specialist"
    }
    fn primary_tool(&self) -> &'static str {
        "reddit_digger"
    }

    async fn execute(&self, ctx: &mut OrchestrationContext) -> Result<String, String> {
        let product = ctx.product.clone().unwrap_or_default();

        ctx.execution_log.push(format!(
            "[{}] Mapping target subreddits: r/{}",
            self.name(),
            product.target_subreddits.join(", r/")
        ));

        let threads = self
            .driver
            .discover_threads(&product.target_subreddits, &product.high_intent_phrases, 15)
            .await;

        let digger_res = OpportunityWorker::dig_sources(
            &threads,
            &product.target_subreddits,
            &product.high_intent_phrases,
        );

        let count = digger_res.total_sources;
        ctx.digger_result = Some(digger_res);
        ctx.execution_log.push(format!(
            "[{}] Aggregated {} deep candidate source threads with canonical URLs.",
            self.name(),
            count
        ));

        Ok(format!("Discovered {} sources", count))
    }
}

/// Tool-Agent: MinerAgent (owns content inspection, prompt sanitization & intent scoring)
pub struct MinerAgent;

#[async_trait]
impl ToolAgent for MinerAgent {
    fn name(&self) -> &'static str {
        "MinerAgent"
    }
    fn role(&self) -> &'static str {
        "Deep Content Mining & Intent Signal Classifier"
    }
    fn primary_tool(&self) -> &'static str {
        "reddit_miner"
    }

    async fn execute(&self, ctx: &mut OrchestrationContext) -> Result<String, String> {
        let sources = ctx
            .digger_result
            .as_ref()
            .map(|d| d.sources.clone())
            .unwrap_or_default();

        if sources.is_empty() {
            ctx.miner_result = Some(MinerResult {
                analyzed_count: 0,
                opportunities: vec![],
            });
            ctx.execution_log.push(format!(
                "[{}] No candidate sources found by DiggerAgent to mine (rate-limited or zero matches).",
                self.name()
            ));
            return Ok("0 sources mined".to_string());
        }

        let product = ctx.product.clone().unwrap_or_default();

        ctx.execution_log.push(format!(
            "[{}] Mining {} candidate threads for intent signals and prompt sanitization...",
            self.name(),
            sources.len()
        ));

        let now = chrono::Utc::now().timestamp();
        let miner_res = OpportunityWorker::mine_content(&sources, &product, now);

        let opp_count = miner_res.opportunities.len();
        ctx.miner_result = Some(miner_res);
        ctx.execution_log.push(format!(
            "[{}] Identified {} high-intent opportunities (RecommendationAsk, ExpressedPain, etc.).",
            self.name(),
            opp_count
        ));

        Ok(format!("Mined {} qualified opportunities", opp_count))
    }
}

/// Tool-Agent: DrafterAgent (owns 25-55 word Experience Grammar drafting)
pub struct DrafterAgent;

#[async_trait]
impl ToolAgent for DrafterAgent {
    fn name(&self) -> &'static str {
        "DrafterAgent"
    }
    fn role(&self) -> &'static str {
        "Anti-Slop Experience Grammar Formulator"
    }
    fn primary_tool(&self) -> &'static str {
        "reddit_draft_response"
    }

    async fn execute(&self, ctx: &mut OrchestrationContext) -> Result<String, String> {
        let opportunities = ctx
            .miner_result
            .as_ref()
            .map(|m| m.opportunities.clone())
            .unwrap_or_default();

        if opportunities.is_empty() {
            ctx.execution_log.push(format!(
                "[{}] No qualified opportunities in context; skipping draft generation.",
                self.name()
            ));
            return Ok("0 drafts generated".to_string());
        }

        let product = ctx.product.clone().unwrap_or_default();

        let mut drafted = 0;
        // Draft for top opportunities
        for opp in opportunities.iter().take(3) {
            match OpportunityWorker::draft_reply(opp, &product) {
                Ok(draft) => {
                    ctx.drafts.push(draft);
                    drafted += 1;
                }
                Err(e) => {
                    ctx.execution_log.push(format!(
                        "[{}] Draft rejected by safety gate: {}",
                        self.name(),
                        e
                    ));
                }
            }
        }

        ctx.execution_log.push(format!(
            "[{}] Generated {} verified Experience Grammar replies (25-55 words with disclosure).",
            self.name(),
            drafted
        ));

        Ok(format!("Drafted {} verified responses", drafted))
    }
}
