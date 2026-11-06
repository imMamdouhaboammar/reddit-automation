use crate::agents::opportunity::{ProductContext, RedditThread};
use crate::ontology::models::{
    CertifiedAngle, CommunityOntology, EngagementVerdict, RuleSeverity, TermCategory,
};

/// Preflight Answerability / Feasibility Gate.
///
/// Directly inspired by NVIDIA Auto-Ontology's `check_answerable` engine:
/// Before an agent spends tokens drafting or attempting to engage on Reddit,
/// this gate checks whether the thread is answerable, certified, and compliant
/// with community taboos.
pub struct PreflightAnswerabilityGate;

impl PreflightAnswerabilityGate {
    /// Evaluates a Reddit thread against community ontology and product context.
    pub fn check_engagable(
        thread: &RedditThread,
        ontology: &CommunityOntology,
        context: &ProductContext,
    ) -> EngagementVerdict {
        let text_lower = format!("{} {}", thread.title, thread.body).to_lowercase();

        // 1. Rule & Taboo Preflight Check (Fail-Closed Safety)
        for rule in &ontology.rules {
            for phrase in &rule.forbidden_phrases {
                let phrase_lower = phrase.to_lowercase();
                if text_lower.contains(&phrase_lower) {
                    return EngagementVerdict::HighRiskViolation {
                        rule_name: rule.rule_name.clone(),
                        detected_phrase: phrase.clone(),
                        severity: rule.severity.clone(),
                    };
                }
            }
        }

        for term in &ontology.terms {
            if term.is_taboo {
                let term_lower = term.term.to_lowercase();
                if text_lower.contains(&term_lower) {
                    return EngagementVerdict::HighRiskViolation {
                        rule_name: "TabooCommunityTerm".to_string(),
                        detected_phrase: term.term.clone(),
                        severity: RuleSeverity::StrictBan,
                    };
                }
            }
        }

        // 2. Semantic Term Grounding Check
        let mut matched_terms = Vec::new();
        let mut matched_pain_points = Vec::new();

        for term in &ontology.terms {
            let term_lower = term.term.to_lowercase();
            let mut matched = text_lower.contains(&term_lower);
            if !matched {
                for syn in &term.synonyms {
                    if text_lower.contains(&syn.to_lowercase()) {
                        matched = true;
                        break;
                    }
                }
            }

            if matched {
                matched_terms.push(term.term.clone());
                if term.category == TermCategory::PainPoint {
                    matched_pain_points.push(term.term.clone());
                }
            }
        }

        // Also check against product context pain points if ontology terms were sparse
        for pain in &context.pain_points {
            if text_lower.contains(&pain.to_lowercase()) && !matched_pain_points.contains(pain) {
                matched_pain_points.push(pain.clone());
            }
        }

        // Also check against certified angles' target pain points
        for angle in ontology.active_certified_angles() {
            let angle_pain = angle.target_pain_point.to_lowercase();
            if text_lower.contains(&angle_pain)
                && !matched_pain_points.contains(&angle.target_pain_point)
            {
                matched_pain_points.push(angle.target_pain_point.clone());
            }
        }

        // If the ontology has terms defined, but none matched the thread at all
        if !ontology.terms.is_empty() && matched_terms.is_empty() && matched_pain_points.is_empty()
        {
            return EngagementVerdict::UnanswerableMissingConcepts {
                missing_concepts: vec![thread.title.clone()],
                rationale: format!(
                    "Thread in r/{} does not align with any known concepts in the community ontology",
                    ontology.subreddit
                ),
            };
        }

        // 3. Answerability Verification: Thread must express an actionable pain point or workflow need
        let has_high_intent = context
            .high_intent_phrases
            .iter()
            .any(|phrase| text_lower.contains(&phrase.to_lowercase()));

        if matched_pain_points.is_empty() && !has_high_intent {
            return EngagementVerdict::UnanswerableMissingConcepts {
                missing_concepts: vec![thread.title.clone()],
                rationale: format!(
                    "Thread in r/{} does not express any actionable pain point or workflow need answerable by the ontology",
                    ontology.subreddit
                ),
            };
        }

        // 4. Human-in-the-Loop Certification Check
        let detected_pain = matched_pain_points
            .first()
            .cloned()
            .unwrap_or_else(|| "general_workflow_need".to_string());

        // Find an active certified angle matching the detected pain or context
        let matching_angle: Option<&CertifiedAngle> = ontology
            .active_certified_angles()
            .into_iter()
            .find(|angle| {
                let angle_pain = angle.target_pain_point.to_lowercase();
                matched_pain_points.iter().any(|p| {
                    p.to_lowercase().contains(&angle_pain) || angle_pain.contains(&p.to_lowercase())
                }) || text_lower.contains(&angle.target_pain_point.to_lowercase())
            });

        if let Some(angle) = matching_angle {
            let term_factor = (matched_terms.len() as f32 * 0.15).min(0.45);
            let pain_factor = (matched_pain_points.len() as f32 * 0.25).min(0.50);
            let alignment_score = (0.35 + term_factor + pain_factor).min(1.0);

            EngagementVerdict::CertifiedSafe {
                angle: angle.clone(),
                matched_terms,
                alignment_score,
            }
        } else {
            // A pain point was detected, but no human has certified an angle for this specific angle yet
            EngagementVerdict::UncertifiedAngleRequired {
                detected_pain_point: detected_pain,
                suggested_pitch: format!(
                    "Authentic value-first perspective addressing pain point for {}",
                    context.product_name
                ),
            }
        }
    }
}
