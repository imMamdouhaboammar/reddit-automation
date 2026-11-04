use crate::ontology::models::{CertifiedAngle, CommunityOntology, CommunityTerm};

/// Manages human-in-the-loop certification of terms and engagement angles.
pub struct CertificationManager;

impl CertificationManager {
    /// Formally certifies a community term by human reviewer.
    pub fn certify_term(
        ontology: &mut CommunityOntology,
        term_id: &str,
    ) -> Result<CommunityTerm, String> {
        if let Some(term) = ontology.terms.iter_mut().find(|t| t.id == term_id) {
            term.certified = true;
            ontology.updated_at = chrono::Utc::now().to_rfc3339();
            Ok(term.clone())
        } else {
            Err(format!("Term {} not found in ontology", term_id))
        }
    }

    /// Marks a community term as taboo (strictly prohibited).
    pub fn mark_term_taboo(
        ontology: &mut CommunityOntology,
        term_id: &str,
    ) -> Result<CommunityTerm, String> {
        if let Some(term) = ontology.terms.iter_mut().find(|t| t.id == term_id) {
            term.is_taboo = true;
            ontology.updated_at = chrono::Utc::now().to_rfc3339();
            Ok(term.clone())
        } else {
            Err(format!("Term {} not found in ontology", term_id))
        }
    }

    /// Certifies and activates a value proposition angle.
    pub fn certify_angle(
        ontology: &mut CommunityOntology,
        angle_id: &str,
        certified_by: &str,
    ) -> Result<CertifiedAngle, String> {
        if let Some(angle) = ontology.certified_angles.iter_mut().find(|a| a.id == angle_id) {
            angle.active = true;
            angle.certified_by = certified_by.to_string();
            angle.certified_at = chrono::Utc::now().to_rfc3339();
            ontology.updated_at = chrono::Utc::now().to_rfc3339();
            Ok(angle.clone())
        } else {
            Err(format!("Angle {} not found in ontology", angle_id))
        }
    }

    /// Revokes or deactivates a previously certified angle.
    pub fn revoke_angle(
        ontology: &mut CommunityOntology,
        angle_id: &str,
    ) -> Result<CertifiedAngle, String> {
        if let Some(angle) = ontology.certified_angles.iter_mut().find(|a| a.id == angle_id) {
            angle.active = false;
            ontology.updated_at = chrono::Utc::now().to_rfc3339();
            Ok(angle.clone())
        } else {
            Err(format!("Angle {} not found in ontology", angle_id))
        }
    }

    /// Lists all uncertified terms awaiting human review.
    pub fn uncertified_terms(ontology: &CommunityOntology) -> Vec<&CommunityTerm> {
        ontology.terms.iter().filter(|t| !t.certified).collect()
    }
}
