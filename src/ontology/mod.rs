pub mod certification;
pub mod engine;
pub mod gate;
pub mod models;

pub use certification::CertificationManager;
pub use engine::AutoOntologyBuilder;
pub use gate::PreflightAnswerabilityGate;
pub use models::{
    CertifiedAngle, CommunityOntology, CommunityTerm, EngagementVerdict, OntologyRelationship,
    OntologyRule, RelationType, RuleSeverity, TermCategory,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::opportunity::{ProductContext, RedditThread};

    #[test]
    fn test_ontology_builder_and_term_extraction() {
        let threads = vec![
            RedditThread {
                id: "t1".to_string(),
                subreddit: "vibecoding".to_string(),
                title: "Cursor and copilot are broken and slow today".to_string(),
                body: "I am struggling with AI code generation. It is a nightmare wasting hours on errors.".to_string(),
                author: "dev1".to_string(),
                created_utc: 1700000000,
                num_comments: 12,
                is_locked: false,
                is_archived: false,
                url: "".to_string(),
            },
            RedditThread {
                id: "t2".to_string(),
                subreddit: "vibecoding".to_string(),
                title: "Alternative to cursor for vibe coding without getting banned".to_string(),
                body: "Looking for an open source tool that does not crash when handling large repos.".to_string(),
                author: "dev2".to_string(),
                created_utc: 1700000100,
                num_comments: 8,
                is_locked: false,
                is_archived: false,
                url: "".to_string(),
            },
        ];

        let mut ontology = AutoOntologyBuilder::build_from_threads(
            "vibecoding",
            "Vibe coding discussions and practitioner workflows",
            &threads,
        );

        assert_eq!(ontology.subreddit, "vibecoding");
        assert!(!ontology.terms.is_empty());
        assert!(!ontology.rules.is_empty());

        // Verify pain points detected
        let pain_terms: Vec<&CommunityTerm> = ontology
            .terms
            .iter()
            .filter(|t| t.category == TermCategory::PainPoint)
            .collect();
        assert!(!pain_terms.is_empty());

        // Verify relationships
        assert!(!ontology.relationships.is_empty());

        // Add certified angle
        let angle = AutoOntologyBuilder::add_certified_angle(
            &mut ontology,
            "Local-First Deterministic Engineering",
            "We build with Rust and local-first SQLite to eliminate slow cloud AI crashes",
            "broken",
            "Full disclosure: I maintain reddit-agent",
            "senior_reviewer",
        );

        assert_eq!(ontology.certified_angles.len(), 1);
        assert_eq!(angle.certified_by, "senior_reviewer");
    }

    #[test]
    fn test_preflight_gate_blocks_taboo_violations() {
        let mut ontology = CommunityOntology::new("SaaS", "SaaS discussions");
        ontology.rules = AutoOntologyBuilder::seed_default_rules("SaaS");

        let spam_thread = RedditThread {
            id: "sp1".to_string(),
            subreddit: "SaaS".to_string(),
            title: "Join my server for upvote for upvote!".to_string(),
            body: "We trade upvotes and free crypto!".to_string(),
            author: "spammer".to_string(),
            created_utc: 1700000000,
            num_comments: 0,
            is_locked: false,
            is_archived: false,
            url: "".to_string(),
        };

        let context = ProductContext::default();
        let verdict = PreflightAnswerabilityGate::check_engagable(&spam_thread, &ontology, &context);

        match verdict {
            EngagementVerdict::HighRiskViolation { rule_name, detected_phrase, severity } => {
                assert_eq!(rule_name, "AntiSpamAndVoteManipulation");
                assert_eq!(detected_phrase, "upvote for upvote");
                assert_eq!(severity, RuleSeverity::StrictBan);
            }
            _ => panic!("Expected HighRiskViolation for spam thread, got {:?}", verdict),
        }
    }

    #[test]
    fn test_preflight_gate_enforces_human_certified_angles() {
        let mut ontology = CommunityOntology::new("SaaS", "SaaS discussions");
        ontology.rules = AutoOntologyBuilder::seed_default_rules("SaaS");

        ontology.terms.push(CommunityTerm {
            id: "t_pain".to_string(),
            term: "wasting hours".to_string(),
            synonyms: vec!["time consuming".to_string()],
            category: TermCategory::PainPoint,
            sentiment_bias: -0.8,
            frequency: 10,
            is_taboo: false,
            certified: true,
        });

        let thread = RedditThread {
            id: "th1".to_string(),
            subreddit: "SaaS".to_string(),
            title: "Tired of wasting hours on Reddit outreach".to_string(),
            body: "Has anyone found a legitimate tool that doesn't feel like spam?".to_string(),
            author: "founder1".to_string(),
            created_utc: 1700000000,
            num_comments: 5,
            is_locked: false,
            is_archived: false,
            url: "".to_string(),
        };

        let context = ProductContext::default();

        // 1. Without certified angle -> UncertifiedAngleRequired
        let verdict_uncertified = PreflightAnswerabilityGate::check_engagable(&thread, &ontology, &context);
        match verdict_uncertified {
            EngagementVerdict::UncertifiedAngleRequired { detected_pain_point, .. } => {
                assert_eq!(detected_pain_point, "wasting hours");
            }
            _ => panic!("Expected UncertifiedAngleRequired, got {:?}", verdict_uncertified),
        }

        // 2. Add human-certified angle -> CertifiedSafe
        AutoOntologyBuilder::add_certified_angle(
            &mut ontology,
            "Authentic Founder Workflow",
            "Share our experience using local-first open-source CLI tools",
            "wasting hours",
            "Full disclosure: I build reddit-agent",
            "qa_reviewer",
        );

        let verdict_certified = PreflightAnswerabilityGate::check_engagable(&thread, &ontology, &context);
        match verdict_certified {
            EngagementVerdict::CertifiedSafe { angle, matched_terms, alignment_score } => {
                assert_eq!(angle.target_pain_point, "wasting hours");
                assert!(matched_terms.contains(&"wasting hours".to_string()));
                assert!(alignment_score > 0.4);
            }
            _ => panic!("Expected CertifiedSafe, got {:?}", verdict_certified),
        }
    }
}
