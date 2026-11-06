use reddit_agent::actions::{ActionPlan, ActionStatus, RiskLevel};
use reddit_agent::drivers::split_plane::{EgressMode, EgressRouter};
use serde_json::json;

fn mock_action_plan(action: &str, target: &str) -> ActionPlan {
    ActionPlan {
        id: "act_test_split_plane".to_string(),
        account: "mamdouh".to_string(),
        reddit_username: Some("boammarr".to_string()),
        driver: "router".to_string(),
        action: action.to_string(),
        target: target.to_string(),
        payload_preview: json!({
            "thing_id": target,
            "text": "High-density technical answer"
        }),
        payload_hash: "hash123".to_string(),
        risk_level: RiskLevel::PublicWrite,
        warnings: vec![],
        status: ActionStatus::Approved,
        created_at: chrono::Utc::now().to_rfc3339(),
        expires_at: (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
    }
}

#[tokio::test]
async fn test_split_plane_router_dispatches_to_local_relay() {
    let router = EgressRouter::new(EgressMode::SplitPlaneRelay);
    let plan = mock_action_plan("create_comment", "t3_1xnhs2u");

    let verdict = router.validate_egress_safety(&plan);
    assert!(verdict.is_ok());

    let payload = router
        .encode_relay_envelope(&plan)
        .expect("encode envelope");
    assert_eq!(payload["action_id"], "act_test_split_plane");
    assert_eq!(payload["egress_mode"], "split_plane_relay");
    assert_eq!(payload["target"], "t3_1xnhs2u");
}

#[tokio::test]
async fn test_direct_official_api_mode_validation() {
    let router = EgressRouter::new(EgressMode::DirectOfficialApi {
        client_id: "0R-test-client-id".to_string(),
    });
    let plan = mock_action_plan("create_comment", "t3_1xnhs2u");

    let verdict = router.validate_egress_safety(&plan);
    assert!(verdict.is_ok());
    assert_eq!(router.mode_name(), "direct_official_api");
}
