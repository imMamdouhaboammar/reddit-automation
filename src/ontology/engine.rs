use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::agents::opportunity::RedditThread;
use crate::ontology::models::{
    CertifiedAngle, CommunityOntology, CommunityTerm, OntologyRelationship, OntologyRule,
    RelationType, RuleSeverity, TermCategory,
};

/// Automated Ontology Builder for Subreddits.
///
/// Dissects Reddit discussions, clusters terminology into semantic concepts,
/// detects latent pain points, competitor mentions, community slang,
/// and maps relationship edges into an enterprise-grade community knowledge graph.
pub struct AutoOntologyBuilder;

impl AutoOntologyBuilder {
    /// Builds a full CommunityOntology by mining a collection of RedditThreads.
    pub fn build_from_threads(
        subreddit: &str,
        description: &str,
        threads: &[RedditThread],
    ) -> CommunityOntology {
        let mut ontology = CommunityOntology::new(subreddit, description);

        // 1. Seed baseline subreddit safety & anti-spam rules
        ontology.rules = Self::seed_default_rules(subreddit);

        // 2. Tokenize and extract term frequencies
        let mut word_counts: HashMap<String, u32> = HashMap::new();
        let mut pain_indicators: HashMap<String, u32> = HashMap::new();
        let mut competitor_indicators: HashMap<String, u32> = HashMap::new();

        let pain_triggers = [
            "broken",
            "frustrating",
            "nightmare",
            "sucks",
            "hate",
            "issue",
            "problem",
            "struggling",
            "impossible",
            "failed",
            "bug",
            "crash",
            "slow",
            "expensive",
            "alternative to",
            "wasting hours",
            "getting banned",
        ];

        let competitor_triggers = [
            "tool",
            "bot",
            "platform",
            "software",
            "app",
            "service",
            "competitor",
            "zapier",
            "make.com",
            "cursor",
            "copilot",
            "chatgpt",
            "gemini",
            "claude",
        ];

        let stop_words: HashSet<&'static str> = [
            "the",
            "and",
            "or",
            "to",
            "in",
            "is",
            "of",
            "it",
            "with",
            "for",
            "on",
            "that",
            "this",
            "was",
            "at",
            "by",
            "an",
            "be",
            "from",
            "as",
            "about",
            "into",
            "like",
            "through",
            "after",
            "over",
            "between",
            "out",
            "against",
            "during",
            "without",
            "before",
            "under",
            "around",
            "among",
            "have",
            "has",
            "had",
            "having",
            "do",
            "does",
            "did",
            "doing",
            "would",
            "should",
            "could",
            "ought",
            "i'm",
            "you're",
            "he's",
            "she's",
            "it's",
            "we're",
            "they're",
            "i've",
            "you've",
            "we've",
            "they've",
            "i'd",
            "you'd",
            "he'd",
            "she'd",
            "we'd",
            "they'd",
            "i'll",
            "you'll",
            "he'll",
            "she'll",
            "we'll",
            "they'll",
            "isn't",
            "aren't",
            "wasn't",
            "weren't",
            "hasn't",
            "haven't",
            "hadn't",
            "doesn't",
            "don't",
            "didn't",
            "won't",
            "wouldn't",
            "shan't",
            "shouldn't",
            "can't",
            "cannot",
            "couldn't",
            "mustn't",
            "let's",
            "that's",
            "who's",
            "what's",
            "here's",
            "there's",
            "when's",
            "where's",
            "why's",
            "how's",
            "a",
            "an",
            "the",
            "and",
            "but",
            "if",
            "or",
            "because",
            "as",
            "until",
            "while",
            "of",
            "at",
            "by",
            "for",
            "with",
            "about",
            "against",
            "between",
            "into",
            "through",
            "during",
            "before",
            "after",
            "above",
            "below",
            "to",
            "from",
            "up",
            "down",
            "in",
            "out",
            "on",
            "off",
            "over",
            "under",
            "again",
            "further",
            "then",
            "once",
            "just",
            "my",
            "your",
            "our",
            "their",
            "me",
            "him",
            "her",
            "us",
            "them",
            "what",
            "which",
            "who",
            "whom",
            "this",
            "that",
            "these",
            "those",
            "am",
            "is",
            "are",
            "was",
            "were",
            "be",
            "been",
            "being",
        ]
        .into_iter()
        .collect();

        for t in threads {
            let full_text = format!("{} {}", t.title, t.body).to_lowercase();

            // Check pain phrases
            for pain in pain_triggers {
                if full_text.contains(pain) {
                    *pain_indicators.entry(pain.to_string()).or_insert(0) += 1;
                }
            }

            // Check competitor phrases
            for comp in competitor_triggers {
                if full_text.contains(comp) {
                    *competitor_indicators.entry(comp.to_string()).or_insert(0) += 1;
                }
            }

            // Word frequencies
            for word in full_text.split_whitespace() {
                let clean: String = word
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                    .collect();
                if clean.len() >= 3 && !stop_words.contains(clean.as_str()) {
                    *word_counts.entry(clean).or_insert(0) += 1;
                }
            }
        }

        // 3. Assemble CommunityTerms
        let mut terms = Vec::new();

        // Add detected pain points
        for (pain, count) in pain_indicators {
            terms.push(CommunityTerm {
                id: format!("term_pain_{}", Uuid::new_v4().simple()),
                term: pain.clone(),
                synonyms: vec![format!("trouble with {}", pain), format!("hate {}", pain)],
                category: TermCategory::PainPoint,
                sentiment_bias: -0.75,
                frequency: count,
                is_taboo: false,
                certified: false,
            });
        }

        // Add detected competitor/tool concepts
        for (comp, count) in competitor_indicators {
            terms.push(CommunityTerm {
                id: format!("term_comp_{}", Uuid::new_v4().simple()),
                term: comp.clone(),
                synonyms: vec![format!("{} alternative", comp)],
                category: TermCategory::CompetitorOrTool,
                sentiment_bias: 0.0,
                frequency: count,
                is_taboo: false,
                certified: false,
            });
        }

        // Add top recurring domain keywords as technical concepts / slang
        let mut sorted_words: Vec<(String, u32)> = word_counts.into_iter().collect();
        sorted_words.sort_by_key(|a| std::cmp::Reverse(a.1));

        for (word, count) in sorted_words.into_iter().take(30) {
            let is_slang = word.contains("vibe")
                || word.contains("grind")
                || word.contains("mrr")
                || word.contains("saas")
                || word.contains("dev");

            terms.push(CommunityTerm {
                id: format!("term_kw_{}", Uuid::new_v4().simple()),
                term: word.clone(),
                synonyms: Vec::new(),
                category: if is_slang {
                    TermCategory::SlangOrJargon
                } else {
                    TermCategory::TechnicalConcept
                },
                sentiment_bias: 0.1,
                frequency: count,
                is_taboo: false,
                certified: false,
            });
        }

        ontology.terms = terms;

        // 4. Infer Relationships between discovered entities
        let mut relationships = Vec::new();
        let pain_terms: Vec<String> = ontology
            .terms
            .iter()
            .filter(|t| t.category == TermCategory::PainPoint)
            .map(|t| t.term.clone())
            .collect();

        let comp_terms: Vec<String> = ontology
            .terms
            .iter()
            .filter(|t| t.category == TermCategory::CompetitorOrTool)
            .map(|t| t.term.clone())
            .collect();

        for pain in &pain_terms {
            for comp in &comp_terms {
                relationships.push(OntologyRelationship {
                    from_term: comp.clone(),
                    to_term: pain.clone(),
                    relation: RelationType::Causes,
                    confidence: 0.65,
                });
            }
        }

        ontology.relationships = relationships;
        ontology
    }

    /// Seeds standard subreddit safety boundaries and anti-spam constraints.
    pub fn seed_default_rules(subreddit: &str) -> Vec<OntologyRule> {
        vec![
            OntologyRule {
                id: format!("rule_ban_{}", subreddit),
                rule_name: "AntiSpamAndVoteManipulation".to_string(),
                description:
                    "Outlaw direct affiliate links, upvote trading, and automated spamming"
                        .to_string(),
                severity: RuleSeverity::StrictBan,
                forbidden_phrases: vec![
                    "upvote for upvote".to_string(),
                    "free crypto".to_string(),
                    "affiliate link".to_string(),
                    "buy followers".to_string(),
                    "dm me for link".to_string(),
                ],
            },
            OntologyRule {
                id: format!("rule_promo_{}", subreddit),
                rule_name: "CommunitySelfPromoPolicy".to_string(),
                description: "Disallow aggressive sales pitches lacking engineering depth"
                    .to_string(),
                severity: RuleSeverity::SelfPromoRestriction,
                forbidden_phrases: vec![
                    "buy now".to_string(),
                    "limited time offer".to_string(),
                    "check my product link below".to_string(),
                ],
            },
        ]
    }

    /// Helper to seed an initial human-certified angle for a subreddit.
    pub fn add_certified_angle(
        ontology: &mut CommunityOntology,
        title: &str,
        angle_pitch: &str,
        target_pain_point: &str,
        mandatory_disclosure: &str,
        certified_by: &str,
    ) -> CertifiedAngle {
        let angle = CertifiedAngle {
            id: format!("angle_{}", Uuid::new_v4().simple()),
            title: title.to_string(),
            angle_pitch: angle_pitch.to_string(),
            target_pain_point: target_pain_point.to_string(),
            mandatory_disclosure: mandatory_disclosure.to_string(),
            certified_by: certified_by.to_string(),
            certified_at: chrono::Utc::now().to_rfc3339(),
            active: true,
        };

        ontology.certified_angles.push(angle.clone());
        ontology.updated_at = chrono::Utc::now().to_rfc3339();
        angle
    }
}
