use serde::{Deserialize, Serialize};

/// High-level domain categorization for a community term.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TermCategory {
    PainPoint,
    SlangOrJargon,
    CompetitorOrTool,
    TechnicalConcept,
    SolutionPattern,
    GeneralDiscussion,
}

/// A certified or discovered term within a subreddit's semantic graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommunityTerm {
    pub id: String,
    pub term: String,
    pub synonyms: Vec<String>,
    pub category: TermCategory,
    /// Sentiment bias from -1.0 (deeply frustrated/negative) to +1.0 (enthusiastic/positive).
    pub sentiment_bias: f32,
    /// Number of occurrences detected during mining.
    pub frequency: u32,
    /// Flag indicating whether usage of this term requires extreme caution or is outright banned.
    pub is_taboo: bool,
    /// Human-in-the-loop certification flag.
    pub certified: bool,
}

/// Severity of an ontology constraint or rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleSeverity {
    StrictBan,
    SelfPromoRestriction,
    TimeRestricted,
    AdvisoryWarning,
}

/// A semantic boundary or behavioral constraint for a specific community.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OntologyRule {
    pub id: String,
    pub rule_name: String,
    pub description: String,
    pub severity: RuleSeverity,
    pub forbidden_phrases: Vec<String>,
}

/// A human-certified value proposition / pitch angle for ethical, high-value engagement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CertifiedAngle {
    pub id: String,
    pub title: String,
    pub angle_pitch: String,
    pub target_pain_point: String,
    pub mandatory_disclosure: String,
    pub certified_by: String,
    pub certified_at: String,
    pub active: bool,
}

/// The nature of relationship between two ontology entities.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationType {
    Solves,
    CompetesWith,
    Causes,
    CorrelatesWith,
    SubconceptOf,
}

/// A directional edge in the community knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OntologyRelationship {
    pub from_term: String,
    pub to_term: String,
    pub relation: RelationType,
    pub confidence: f32,
}

/// Complete Semantic Ontology for a Subreddit community.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommunityOntology {
    pub subreddit: String,
    pub description: String,
    pub terms: Vec<CommunityTerm>,
    pub rules: Vec<OntologyRule>,
    pub certified_angles: Vec<CertifiedAngle>,
    pub relationships: Vec<OntologyRelationship>,
    pub version: u32,
    pub created_at: String,
    pub updated_at: String,
}

impl CommunityOntology {
    pub fn new(subreddit: &str, description: &str) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            subreddit: subreddit.to_string(),
            description: description.to_string(),
            terms: Vec::new(),
            rules: Vec::new(),
            certified_angles: Vec::new(),
            relationships: Vec::new(),
            version: 1,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn find_term(&self, query: &str) -> Option<&CommunityTerm> {
        let lower = query.to_lowercase();
        self.terms.iter().find(|t| {
            t.term.to_lowercase() == lower
                || t.synonyms.iter().any(|s| s.to_lowercase() == lower)
        })
    }

    pub fn active_certified_angles(&self) -> Vec<&CertifiedAngle> {
        self.certified_angles.iter().filter(|a| a.active).collect()
    }
}

/// Outcome of the Preflight Answerability / Feasibility Gate (`check_engagable`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EngagementVerdict {
    /// Completely safe to answer using an explicitly certified value proposition.
    CertifiedSafe {
        angle: CertifiedAngle,
        matched_terms: Vec<String>,
        alignment_score: f32,
    },
    /// Violates an explicit community taboo or rule (fail-closed).
    HighRiskViolation {
        rule_name: String,
        detected_phrase: String,
        severity: RuleSeverity,
    },
    /// The thread topics are outside the verified ontology domain (stops hallucinations).
    UnanswerableMissingConcepts {
        missing_concepts: Vec<String>,
        rationale: String,
    },
    /// The thread matches a pain point, but no human-certified angle exists yet.
    UncertifiedAngleRequired {
        detected_pain_point: String,
        suggested_pitch: String,
    },
}
