use std::sync::Arc;
use serde_json::json;

use reddit_agent::accounts::AccountManager;
use reddit_agent::actions::ActionManager;
use reddit_agent::agents::opportunity::{OpportunityWorker, ProductContext, RedditThread, ReviewStatus};
use reddit_agent::audit::AuditLogger;
use reddit_agent::auth::{AuthManager, MemoryKeyring};
use reddit_agent::drivers::Capability;
use reddit_agent::issues::IssueEngine;
use reddit_agent::mcp::McpServer;
use reddit_agent::ontology::{
    AutoOntologyBuilder, EngagementVerdict, PreflightAnswerabilityGate, RuleSeverity, TermCategory,
};
use reddit_agent::router::CapabilityRouter;
use reddit_agent::storage::Storage;

struct TestDummyDriver;

#[async_trait::async_trait]
impl reddit_agent::drivers::Driver for TestDummyDriver {
    fn name(&self) -> &'static str {
        "test_dummy"
    }
    fn supports(&self, _cap: &Capability) -> bool {
        true
    }
    async fn execute(
        &self,
        cap: &Capability,
        _ctx: &reddit_agent::accounts::AccountContext,
        in_val: serde_json::Value,
    ) -> Result<serde_json::Value, reddit_agent::drivers::DriverError> {
        match cap {
            Capability::GetIdentity => Ok(json!({
                "identity": { "name": "ontology_tester", "link_karma": 100, "comment_karma": 500 }
            })),
            _ => Ok(in_val),
        }
    }
    async fn health_check(
        &self,
        ctx: &reddit_agent::accounts::AccountContext,
    ) -> Result<reddit_agent::drivers::HealthReport, reddit_agent::drivers::DriverError> {
        Ok(reddit_agent::drivers::HealthReport {
            account: ctx.account.alias.clone(),
            reddit_username: ctx.account.reddit_username.clone(),
            status: "healthy".to_string(),
            issues: vec![],
        })
    }
}

#[tokio::test]
async fn test_e2e_auto_ontology_lifecycle_and_gate_enforcement() {
    // 1. Setup in-memory SQLite storage
    let storage = Arc::new(Storage::new_in_memory().expect("in-memory sqlite"));

    // 2. Synthesize CommunityOntology from candidate threads
    let threads = vec![
        RedditThread {
            id: "th_vibe_1".to_string(),
            subreddit: "vibecoding".to_string(),
            title: "Cursor and copilot are broken and slow when working on large projects".to_string(),
            body: "I hate wasting hours debugging AI hallucinations. Is there an alternative to cursor that is fast and reliable?".to_string(),
            author: "builder1".to_string(),
            created_utc: 1700000000,
            num_comments: 15,
            is_locked: false,
            is_archived: false,
            url: "https://reddit.com/r/vibecoding/comments/th_vibe_1/".to_string(),
        },
        RedditThread {
            id: "th_vibe_spam".to_string(),
            subreddit: "vibecoding".to_string(),
            title: "Upvote for upvote discord server for developers!".to_string(),
            body: "Join our network to buy followers and exchange upvotes!".to_string(),
            author: "spammer_bot".to_string(),
            created_utc: 1700000100,
            num_comments: 0,
            is_locked: false,
            is_archived: false,
            url: "https://reddit.com/r/vibecoding/comments/th_vibe_spam/".to_string(),
        },
        RedditThread {
            id: "th_vibe_offtopic".to_string(),
            subreddit: "vibecoding".to_string(),
            title: "What is your favorite cooking recipe for pasta?".to_string(),
            body: "I love making carbonara with pecorino cheese.".to_string(),
            author: "chef_user".to_string(),
            created_utc: 1700000200,
            num_comments: 3,
            is_locked: false,
            is_archived: false,
            url: "https://reddit.com/r/vibecoding/comments/th_vibe_offtopic/".to_string(),
        },
    ];

    let mut ontology = AutoOntologyBuilder::build_from_threads(
        "vibecoding",
        "Practitioners exploring AI-assisted and spec-driven software engineering",
        &threads,
    );

    assert_eq!(ontology.subreddit, "vibecoding");
    assert!(!ontology.terms.is_empty());
    assert!(!ontology.rules.is_empty());
    assert!(!ontology.relationships.is_empty());

    // Verify pain point detected
    let has_wasting_hours = ontology.terms.iter().any(|t| t.term == "wasting hours" && t.category == TermCategory::PainPoint);
    assert!(has_wasting_hours);

    // 3. Human-in-the-Loop Certification: Add and activate a certified angle
    let angle = AutoOntologyBuilder::add_certified_angle(
        &mut ontology,
        "Local-First Deterministic Engineering",
        "building local-first CLI tools in Rust with local SQLite to eliminate cloud failures",
        "wasting hours",
        "Full disclosure: I am the author of reddit-agent",
        "principal_architect",
    );
    assert_eq!(angle.certified_by, "principal_architect");
    assert!(angle.active);

    // 4. Persist ontology & certified angle in Storage
    storage.save_community_ontology(&ontology).expect("save ontology");
    storage.save_certified_angle("vibecoding", &angle).expect("save angle");

    let loaded_ont = storage.get_community_ontology("vibecoding").expect("query").expect("found");
    assert_eq!(loaded_ont.subreddit, "vibecoding");
    assert_eq!(loaded_ont.certified_angles.len(), 1);

    // 5. Test Preflight Answerability Gate across scenarios
    let product = ProductContext {
        product_name: "reddit-agent".to_string(),
        value_prop: "Local-first deterministic Reddit operator".to_string(),
        icp_target: "Engineers and founders".to_string(),
        pain_points: vec!["wasting hours".to_string(), "broken".to_string()],
        competitors: vec!["generic-bot".to_string()],
        target_subreddits: vec!["vibecoding".to_string()],
        high_intent_phrases: vec!["alternative to cursor".to_string()],
        affiliation_disclosure: "Full disclosure: I maintain reddit-agent".to_string(),
    };

    // Scenario A: Spam thread -> HighRiskViolation (Fail-closed, 0 API waste)
    let verdict_spam = PreflightAnswerabilityGate::check_engagable(&threads[1], &loaded_ont, &product);
    match verdict_spam {
        EngagementVerdict::HighRiskViolation { rule_name, severity, detected_phrase } => {
            assert_eq!(rule_name, "AntiSpamAndVoteManipulation");
            assert_eq!(severity, RuleSeverity::StrictBan);
            assert!(detected_phrase.contains("upvote for upvote"));
        }
        other => panic!("Expected HighRiskViolation for spam thread, got {:?}", other),
    }

    // Scenario B: Off-topic thread -> UnanswerableMissingConcepts
    let verdict_offtopic = PreflightAnswerabilityGate::check_engagable(&threads[2], &loaded_ont, &product);
    match verdict_offtopic {
        EngagementVerdict::UnanswerableMissingConcepts { missing_concepts, .. } => {
            assert!(!missing_concepts.is_empty());
        }
        other => panic!("Expected UnanswerableMissingConcepts for cooking thread, got {:?}", other),
    }

    // Scenario C: High-signal thread -> CertifiedSafe
    let verdict_safe = PreflightAnswerabilityGate::check_engagable(&threads[0], &loaded_ont, &product);
    match verdict_safe {
        EngagementVerdict::CertifiedSafe { angle: matched_angle, matched_terms, alignment_score } => {
            assert_eq!(matched_angle.title, "Local-First Deterministic Engineering");
            assert!(matched_terms.contains(&"wasting hours".to_string()));
            assert!(alignment_score >= 0.5);
        }
        other => panic!("Expected CertifiedSafe for high-signal thread, got {:?}", other),
    }

    // 6. Test OpportunityWorker Integration with Preflight Ontology Gate
    let now_utc = 1700000500;
    let ranked = OpportunityWorker::discover_opportunities_with_ontology(&threads, &product, &loaded_ont, now_utc);

    // Spam thread and off-topic thread are automatically eliminated!
    assert_eq!(ranked.len(), 1);
    assert_eq!(ranked[0].thread.id, "th_vibe_1");
    assert!(ranked[0].composite_score > 0.75);

    // 7. Verify Drafted Reply utilizes the Certified Angle
    let draft = OpportunityWorker::draft_reply(&ranked[0], &product).expect("draft reply");
    assert_eq!(draft.review_status, ReviewStatus::Approved);
    assert!(draft.draft_text.contains("building local-first CLI tools in Rust"));
    assert!(draft.draft_text.contains("Full disclosure: I am the author of reddit-agent"));
    assert!(draft.experience_grammar_valid);
    assert!(draft.word_count >= 20 && draft.word_count <= 65);

    // 8. Test MCP Tools Server Integration
    let keyring = Arc::new(MemoryKeyring::new());
    let client = reqwest::Client::new();
    let auth = Arc::new(AuthManager::new(keyring, client));
    let account_mgr = Arc::new(AccountManager::new(storage.clone(), auth));
    let driver = Arc::new(TestDummyDriver);
    let router = Arc::new(CapabilityRouter::new(driver.clone(), driver));
    let audit = Arc::new(AuditLogger::new(storage.clone()));
    let action_mgr = Arc::new(ActionManager::new(storage.clone(), router.clone(), account_mgr.clone(), audit));
    let issue_engine = Arc::new(IssueEngine::new(router.clone(), account_mgr.clone(), action_mgr.clone(), storage.clone(), "test_cid".to_string()));

    let mcp = McpServer::new(account_mgr, router, action_mgr, issue_engine).with_storage(storage.clone());

    // MCP: List ontologies
    let mcp_list = mcp.handle_tool_call("reddit_ontology_list", json!({})).await.unwrap();
    assert_eq!(mcp_list, json!(["vibecoding"]));

    // MCP: Get ontology
    let mcp_ont = mcp.handle_tool_call("reddit_ontology_get", json!({ "subreddit": "vibecoding" })).await.unwrap();
    assert_eq!(mcp_ont["subreddit"], "vibecoding");
    assert_eq!(mcp_ont["certified_angles"].as_array().unwrap().len(), 1);

    // MCP: Check engagable
    let mcp_check = mcp.handle_tool_call(
        "reddit_ontology_check_engagable",
        json!({
            "subreddit": "vibecoding",
            "title": "Cursor broken, looking for alternative to cursor that is wasting hours",
            "pain_point": "wasting hours"
        }),
    ).await.unwrap();
    assert!(mcp_check.get("CertifiedSafe").is_some());
}
