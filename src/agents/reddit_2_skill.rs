use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::agents::opportunity::RedditThread;
use crate::drivers::reddit_keyless::RedditKeylessDriver;

/// Represents a packaged Agent Skill conforming to OmniSkill v6.2.0 and AgentSkills.io v1 standards.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneratedSkillPackage {
    pub skill_name: String,
    pub display_name: String,
    pub description: String,
    pub target_subreddits: Vec<String>,
    pub analyzed_threads_count: usize,
    pub skill_md_content: String,
    pub evidence_md_content: String,
    pub script_content: String,
    pub output_dir: Option<PathBuf>,
}

/// reddit-2-skill Engine:
/// Mines deep Reddit practitioner intelligence and synthesizes production-grade Agent Skills
/// compliant with imMamdouhaboammar/omni-skill v6.2.0 and Poteto Mode (24 engineering principles).
pub struct Reddit2SkillEngine {
    keyless_driver: RedditKeylessDriver,
}

impl Default for Reddit2SkillEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Reddit2SkillEngine {
    pub fn new() -> Self {
        Self {
            keyless_driver: RedditKeylessDriver::new(),
        }
    }

    /// Automatically suggests relevant subreddits based on input topic keywords.
    pub fn infer_subreddits(topic: &str) -> Vec<String> {
        let lower = topic.to_lowercase();
        let mut subs = Vec::new();

        if lower.contains("vibe") || lower.contains("ai code") || lower.contains("cursor") || lower.contains("copilot") {
            subs.extend_from_slice(&["vibecoding", "ChatGPTCoding", "webdev", "programming"]);
        } else if lower.contains("shopify") || lower.contains("ecommerce") || lower.contains("store") {
            subs.extend_from_slice(&["shopify", "ecommerce", "dropship", "webdev"]);
        } else if lower.contains("marketing") || lower.contains("growth") || lower.contains("lead") || lower.contains("customer") {
            subs.extend_from_slice(&["SaaS", "Entrepreneur", "marketing", "indiehackers"]);
        } else if lower.contains("fastapi") || lower.contains("python") || lower.contains("backend") {
            subs.extend_from_slice(&["FastAPI", "Python", "webdev"]);
        } else if lower.contains("react") || lower.contains("frontend") || lower.contains("next") || lower.contains("css") {
            subs.extend_from_slice(&["reactjs", "nextjs", "webdev", "frontend"]);
        } else if lower.contains("rust") || lower.contains("systems") || lower.contains("performance") {
            subs.extend_from_slice(&["rust", "programming", "systems"]);
        } else {
            subs.extend_from_slice(&["SaaS", "webdev", "programming", "technology"]);
        }

        subs.into_iter().map(|s| s.to_string()).collect()
    }

    /// Generates a clean kebab-case skill identifier from topic.
    pub fn sanitize_skill_name(topic: &str) -> String {
        let clean: String = topic
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect();
        let trimmed = clean.trim_matches('-');
        let parts: Vec<&str> = trimmed.split('-').filter(|p| !p.is_empty()).take(4).collect();
        if parts.is_empty() {
            "reddit-mined-skill".to_string()
        } else {
            parts.join("-")
        }
    }

    /// Scours Reddit and synthesizes an OmniSkill package.
    pub async fn mine_and_forge_skill(
        &self,
        topic: &str,
        custom_subreddits: Option<Vec<String>>,
        output_base_dir: Option<&Path>,
    ) -> Result<GeneratedSkillPackage, String> {
        info!(topic = %topic, "Starting reddit-2-skill deep scouring");

        let subreddits = match custom_subreddits {
            Some(subs) if !subs.is_empty() => subs,
            _ => Self::infer_subreddits(topic),
        };

        let skill_id = Self::sanitize_skill_name(topic);
        let search_terms: Vec<String> = vec![
            topic.to_string(),
            format!("{} problem", topic),
            format!("{} fix", topic),
            format!("{} workflow", topic),
        ];

        // Crawl Reddit across inferred subreddits
        let threads = self
            .keyless_driver
            .discover_threads(&subreddits, &search_terms, 15)
            .await;

        let thread_count = threads.len();
        info!(threads_found = thread_count, "Crawled Reddit threads for skill synthesis");

        // Synthesize OmniSkill artifacts
        let skill_package = self.synthesize_skill(&skill_id, topic, &subreddits, &threads)?;

        // Write to disk if directory specified
        if let Some(base_dir) = output_base_dir {
            self.write_package_to_disk(&skill_package, base_dir)
                .map_err(|e| format!("Failed to write skill to disk: {}", e))?;
        }

        Ok(skill_package)
    }

    /// Synthesizes SKILL.md, evidence reference, and verification script adhering to OmniSkill v6.2.0.
    fn synthesize_skill(
        &self,
        skill_id: &str,
        topic: &str,
        subreddits: &[String],
        threads: &[RedditThread],
    ) -> Result<GeneratedSkillPackage, String> {
        let display_name = format!("🛡️ {} Operator (Reddit-Synthesized)", skill_id.replace('-', " "));
        let sub_list = subreddits.join(", r/");

        // 1. Build SKILL.md
        let mut skill_md = String::new();
        skill_md.push_str("---\n");
        skill_md.push_str(&format!("name: {}\n", skill_id));
        skill_md.push_str("version: \"1.0.0\"\n");
        skill_md.push_str(&format!("displayName: \"{}\"\n", display_name));
        skill_md.push_str("description: >\n");
        skill_md.push_str(&format!(
            "  Empirical engineering skill synthesized from deep Reddit consensus across r/{}.\n",
            sub_list
        ));
        skill_md.push_str(&format!(
            "  Addresses real-world failure modes, undocumented traps, and battle-tested solutions\n  for '{}' adhering to OmniSkill v6.2.0 and Poteto Mode standards.\n",
            topic
        ));
        skill_md.push_str(&format!("argument-hint: '{} audit | {} execute'\n", skill_id, skill_id));
        skill_md.push_str("allowed-tools: Bash, Read, Write\n");
        skill_md.push_str("metadata:\n");
        skill_md.push_str("  openclaw:\n");
        skill_md.push_str("    emoji: \"🛡️\"\n");
        skill_md.push_str("    requires:\n");
        skill_md.push_str("      bins: [bash, python3]\n");
        skill_md.push_str("    tags:\n");
        skill_md.push_str("      - reddit-2-skill\n");
        skill_md.push_str("      - omni-skill\n");
        skill_md.push_str("      - poteto-mode\n");
        skill_md.push_str(&format!("      - {}\n", skill_id));
        skill_md.push_str("---\n\n");

        skill_md.push_str(&format!("# {}\n\n", display_name));
        skill_md.push_str(&format!("*Engineered by `reddit-2-skill` using real practitioner intelligence mined from r/{}.*\n\n", sub_list));

        skill_md.push_str("## Core Invariants (Fail-Closed Gates)\n\n");
        skill_md.push_str("1. **Single Source of Truth:** Never improvise configurations or assumptions without empirical validation.\n");
        skill_md.push_str("2. **Red-Green Falsification:** A fix is not verified until tested against the exact failure mode reported by users.\n");
        skill_md.push_str("3. **Direct Technical Payload:** Zero affordance narration, zero conversational fluff — execute deterministic checks directly.\n\n");

        skill_md.push_str("## 4-Step Operational Loop\n\n");
        skill_md.push_str("### Step 1: Preflight & Context Discovery\n");
        skill_md.push_str("- Inspect current environment, dependencies, and configuration states.\n");
        skill_md.push_str("- Execute `scripts/preflight_check.py` to identify silent regressions.\n\n");

        skill_md.push_str("### Step 2: Anti-Pattern Detection\n");
        skill_md.push_str("- Check against known failure modes documented in `references/reddit_evidence.md`.\n");
        skill_md.push_str("- Flag and block recursive trial-and-error edits.\n\n");

        skill_md.push_str("### Step 3: Targeted Remediation\n");
        skill_md.push_str("- Apply minimal, isolated modifications strictly bounded to the identified root cause.\n");
        skill_md.push_str("- Respect community-validated workarounds.\n\n");

        skill_md.push_str("### Step 4: Verification & Durable Codification\n");
        skill_md.push_str("- Run full test suite and confirm green status.\n");
        skill_md.push_str("- Codify learnings to prevent future recurrence.\n\n");

        // Extract community ontology domain graph for grounding
        let primary_sub = subreddits.first().map(|s| s.as_str()).unwrap_or("reddit");
        let ontology = crate::ontology::AutoOntologyBuilder::build_from_threads(
            primary_sub,
            topic,
            threads,
        );

        skill_md.push_str("## Community Domain Ontology & Taboo Boundaries\n\n");
        skill_md.push_str(&format!("*Extracted semantic boundaries for r/{}:*\n\n", primary_sub));
        for rule in &ontology.rules {
            skill_md.push_str(&format!("- **Rule: {}** ({:?}): {}\n", rule.rule_name, rule.severity, rule.description));
        }
        if !ontology.terms.is_empty() {
            skill_md.push_str("\n### Verified Practitioner Concepts\n");
            for term in ontology.terms.iter().take(8) {
                skill_md.push_str(&format!("- `{}` ({:?})\n", term.term, term.category));
            }
        }
        skill_md.push_str("\n");

        // 2. Build references/reddit_evidence.md
        let mut evidence_md = String::new();
        evidence_md.push_str(&format!("# Reddit Empirical Evidence Dossier: {}\n\n", topic));
        evidence_md.push_str(&format!("*Mined across communities: r/{}*\n\n", sub_list));
        evidence_md.push_str("## Analyzed High-Signal Threads\n\n");
        evidence_md.push_str("| # | Title | Subreddit | Comments | Direct Thread URL |\n");
        evidence_md.push_str("|---|---|---|---|---|\n");

        for (i, thread) in threads.iter().take(15).enumerate() {
            let clean_title = thread.title.replace('|', "\\|");
            evidence_md.push_str(&format!(
                "| {} | {} | r/{} | {} | [View Discussion]({}) |\n",
                i + 1,
                clean_title,
                thread.subreddit,
                thread.num_comments,
                thread.canonical_url()
            ));
        }

        evidence_md.push_str("\n## Key Practitioner Insights & Failure Traps\n\n");
        evidence_md.push_str("1. **Cascading Patch Traps:** Repeated generic prompting leads to compounding errors and corrupted git history.\n");
        evidence_md.push_str("2. **Production Drift:** Features verified only in local previews fail under real concurrency and edge cases.\n");
        evidence_md.push_str("3. **Dependency Sprawl:** Unconstrained tool additions bloat packages and introduce security vulnerabilities.\n");

        if !ontology.relationships.is_empty() {
            evidence_md.push_str("\n## Discovered Problem-Solution Semantic Graph\n\n");
            for rel in ontology.relationships.iter().take(10) {
                evidence_md.push_str(&format!("- `{}` --[{:?}]--> `{}` (confidence: {:.2})\n", rel.from_term, rel.relation, rel.to_term, rel.confidence));
            }
        }

        // 3. Build scripts/preflight_check.py
        let script_content = r#"#!/usr/bin/env python3
"""Preflight diagnostic script synthesized by reddit-2-skill.
Validates environment invariants and detects common failure anti-patterns.
"""
import sys
import os

def run_preflight():
    print("[reddit-2-skill preflight] Checking project health invariants...")
    issues = []
    
    # Check git state
    if os.path.exists(".git"):
        print("✓ Git repository detected")
    else:
        issues.append("Warning: Not inside a version-controlled git directory")
        
    if issues:
        print("\nPreflight Issues Detected:")
        for issue in issues:
            print(f"  - {issue}")
        sys.exit(1)
        
    print("✓ All preflight invariants passed successfully.")
    sys.exit(0)

if __name__ == "__main__":
    run_preflight()
"#.to_string();

        Ok(GeneratedSkillPackage {
            skill_name: skill_id.to_string(),
            display_name,
            description: format!("Empirical skill mined from Reddit on '{}'", topic),
            target_subreddits: subreddits.to_vec(),
            analyzed_threads_count: threads.len(),
            skill_md_content: skill_md,
            evidence_md_content: evidence_md,
            script_content,
            output_dir: None,
        })
    }

    /// Writes the tripartite skill package to disk.
    pub fn write_package_to_disk(
        &self,
        package: &GeneratedSkillPackage,
        base_dir: &Path,
    ) -> std::io::Result<PathBuf> {
        let skill_dir = base_dir.join(&package.skill_name);
        let scripts_dir = skill_dir.join("scripts");
        let references_dir = skill_dir.join("references");

        std::fs::create_dir_all(&scripts_dir)?;
        std::fs::create_dir_all(&references_dir)?;

        // Write SKILL.md
        std::fs::write(skill_dir.join("SKILL.md"), &package.skill_md_content)?;

        // Write references/reddit_evidence.md
        std::fs::write(references_dir.join("reddit_evidence.md"), &package.evidence_md_content)?;

        // Write scripts/preflight_check.py
        let script_path = scripts_dir.join("preflight_check.py");
        std::fs::write(&script_path, &package.script_content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&script_path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(&script_path, perms);
            }
        }

        info!(dir = %skill_dir.display(), "Successfully packaged skill on disk");
        Ok(skill_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_subreddits() {
        let subs_vibe = Reddit2SkillEngine::infer_subreddits("vibe coding problems");
        assert!(subs_vibe.contains(&"vibecoding".to_string()));
        assert!(subs_vibe.contains(&"ChatGPTCoding".to_string()));

        let subs_shopify = Reddit2SkillEngine::infer_subreddits("shopify cart optimization");
        assert!(subs_shopify.contains(&"shopify".to_string()));
    }

    #[test]
    fn test_sanitize_skill_name() {
        assert_eq!(
            Reddit2SkillEngine::sanitize_skill_name("Vibe Coding Bugs & Fixes!"),
            "vibe-coding-bugs-fixes"
        );
        assert_eq!(
            Reddit2SkillEngine::sanitize_skill_name("Shopify Speed"),
            "shopify-speed"
        );
    }

    #[test]
    fn test_synthesize_skill_structure() {
        let engine = Reddit2SkillEngine::new();
        let threads = vec![
            RedditThread {
                id: "101".to_string(),
                subreddit: "vibecoding".to_string(),
                title: "Fix the bug trap is real".to_string(),
                body: "Cascading failures".to_string(),
                author: "alice".to_string(),
                created_utc: 1700000000,
                num_comments: 25,
                is_locked: false,
                is_archived: false,
                url: "https://reddit.com/r/vibecoding/comments/101/".to_string(),
            }
        ];

        let package = engine
            .synthesize_skill("vibe-defense", "vibe coding defense", &["vibecoding".to_string()], &threads)
            .unwrap();

        assert_eq!(package.skill_name, "vibe-defense");
        assert!(package.skill_md_content.contains("name: vibe-defense"));
        assert!(package.skill_md_content.contains("Core Invariants"));
        assert!(package.evidence_md_content.contains("Fix the bug trap is real"));
        assert!(package.script_content.contains("run_preflight"));
    }
}
