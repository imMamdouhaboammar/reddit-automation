use crate::actions::ActionPlan;
use crate::drivers::DriverError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Network Egress Strategy to bypass datacenter IP blocks without expensive residential proxies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EgressMode {
    /// Mode A: Encrypted Relay through the operator's authentic residential node (Mac/PC/Home Router)
    SplitPlaneRelay,
    /// Mode B: Direct cloud egress using registered Reddit Developer OAuth allowlisting
    DirectOfficialApi { client_id: String },
}

#[derive(Debug, Clone)]
pub struct EgressRouter {
    pub mode: EgressMode,
}

impl EgressRouter {
    pub fn new(mode: EgressMode) -> Self {
        Self { mode }
    }

    pub fn mode_name(&self) -> &'static str {
        match &self.mode {
            EgressMode::SplitPlaneRelay => "split_plane_relay",
            EgressMode::DirectOfficialApi { .. } => "direct_official_api",
        }
    }

    /// Validates that the requested action is safe to dispatch via the current egress path
    pub fn validate_egress_safety(&self, plan: &ActionPlan) -> Result<(), DriverError> {
        match &self.mode {
            EgressMode::SplitPlaneRelay => {
                // Ensure target and action are well-formed before dispatching to relay
                if plan.target.is_empty() {
                    return Err(DriverError::Internal(
                        "Split-plane dispatch rejected: empty target".to_string(),
                    ));
                }
                Ok(())
            }
            EgressMode::DirectOfficialApi { client_id } => {
                if client_id.trim().is_empty() {
                    return Err(DriverError::Internal(
                        "Direct Official API mode requires a valid non-empty client_id".to_string(),
                    ));
                }
                Ok(())
            }
        }
    }

    /// Encodes an ActionPlan into a cryptographic envelope for local relay execution
    pub fn encode_relay_envelope(&self, plan: &ActionPlan) -> Result<Value, DriverError> {
        self.validate_egress_safety(plan)?;

        Ok(json!({
            "action_id": plan.id,
            "egress_mode": self.mode_name(),
            "account": plan.account,
            "reddit_username": plan.reddit_username,
            "action": plan.action,
            "target": plan.target,
            "payload": plan.payload_preview,
            "payload_hash": plan.payload_hash,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}
