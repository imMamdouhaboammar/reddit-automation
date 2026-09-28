use super::opportunity::{ProductContext, RedditThread};
use serde::{Deserialize, Serialize};

/// Subreddit community profile capturing guidelines, rules, and engagement norms.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommunityProfile {
    pub subreddit: String,
    pub subscriber_count: u64,
    pub self_promo_allowed: bool,
    pub promo_guidelines: String,
    pub culture_notes: String,
    pub primary_pain_points: Vec<String>,
}

/// Engagement log entry to audit the 90/10 value-to-promo rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngagementRecord {
    pub id: String,
    pub subreddit: String,
    pub is_promotional: bool,
    pub summary: String,
    pub timestamp_utc: i64,
}

/// Audit result for the 90/10 value rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RatioAudit {
    pub total_engagements: u32,
    pub value_add_count: u32,
    pub promotional_count: u32,
    pub value_percentage: f64,
    pub is_compliant: bool,
    pub advisory: String,
}

/// Sentiment classification for community discussions and brand mentions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SentimentClassification {
    Positive,
    Neutral,
    Negative,
    CriticalCrisis,
}

/// Brand mention event tracked across monitored subreddits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BrandMention {
    pub id: String,
    pub subreddit: String,
    pub author: String,
    pub snippet: String,
    pub sentiment: SentimentClassification,
    pub urgency: u8, // 1 (lowest) to 5 (highest crisis)
}

/// A value-first educational content strategy proposal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentStrategy {
    EducationalGuide {
        title: String,
        target_subreddit: String,
        key_takeaways: Vec<String>,
        outline: Vec<String>,
    },
    ResourceSharing {
        title: String,
        target_subreddit: String,
        resource_description: String,
    },
    AmaPlan {
        target_subreddit: String,
        host_role: String,
        core_theme: String,
        anticipated_questions: Vec<String>,
        community_value_deliverables: Vec<String>,
    },
}

/// Summary report of community sentiment and discovered pain points.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubredditSentimentReport {
    pub subreddit: String,
    pub total_analyzed: usize,
    pub positive_count: usize,
    pub neutral_count: usize,
    pub negative_count: usize,
    pub identified_pain_points: Vec<String>,
}

/// A non-defensive, helpful response drafted for a brand mention.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommunityRemediationDraft {
    pub mention_id: String,
    pub original_snippet: String,
    pub sentiment: SentimentClassification,
    pub drafted_reply: String,
    pub tone: String,
    pub community_first_validated: bool,
}

/// Fully autonomous Community Builder Worker implementing `agency-reddit-community-builder`.
pub struct CommunityBuilderWorker;

impl CommunityBuilderWorker {
    /// Audits recent engagements to strictly enforce the Reddit 90/10 value-to-promo rule.
    pub fn audit_90_10_ratio(history: &[EngagementRecord]) -> RatioAudit {
        if history.is_empty() {
            return RatioAudit {
                total_engagements: 0,
                value_add_count: 0,
                promotional_count: 0,
                value_percentage: 100.0,
                is_compliant: true,
                advisory: "No historical activity yet. Start with pure value-add posts."
                    .to_string(),
            };
        }

        let total = history.len() as u32;
        let promo = history.iter().filter(|r| r.is_promotional).count() as u32;
        let value = total.saturating_sub(promo);
        let value_percentage = (value as f64 / total as f64) * 100.0;

        let is_compliant = value_percentage >= 90.0;
        let advisory = if is_compliant {
            format!(
                "Compliant with 90/10 rule ({:.1}% value). Safe to engage authentically.",
                value_percentage
            )
        } else {
            let needed_value = ((promo as f64 / 0.10) - total as f64).ceil() as u32;
            format!(
                "NON-COMPLIANT: Promotional content is {:.1}%. You must publish at least {} pure value contributions before mentioning products.",
                100.0 - value_percentage, needed_value
            )
        };

        RatioAudit {
            total_engagements: total,
            value_add_count: value,
            promotional_count: promo,
            value_percentage,
            is_compliant,
            advisory,
        }
    }

    /// Analyzes sentiment and extracts common friction points across subreddit threads.
    pub fn analyze_subreddit_sentiment(
        subreddit: &str,
        threads: &[RedditThread],
    ) -> SubredditSentimentReport {
        let mut pos = 0;
        let mut neu = 0;
        let mut neg = 0;
        let mut pain_points = Vec::new();

        let negative_markers = [
            "frustrated",
            "hate",
            "broken",
            "terrible",
            "issue",
            "bug",
            "scam",
            "waste",
        ];
        let positive_markers = [
            "love",
            "awesome",
            "great",
            "helpful",
            "fixed",
            "best",
            "thanks",
            "recommend",
        ];

        for thread in threads {
            let text = format!("{} {}", thread.title, thread.body).to_lowercase();

            let has_neg = negative_markers.iter().any(|m| text.contains(m));
            let has_pos = positive_markers.iter().any(|m| text.contains(m));

            if has_neg && !has_pos {
                neg += 1;
                // Capture first 5 words as pain point hint
                let words: Vec<&str> = thread.title.split_whitespace().take(6).collect();
                pain_points.push(words.join(" "));
            } else if has_pos && !has_neg {
                pos += 1;
            } else {
                neu += 1;
            }
        }

        SubredditSentimentReport {
            subreddit: subreddit.to_string(),
            total_analyzed: threads.len(),
            positive_count: pos,
            neutral_count: neu,
            negative_count: neg,
            identified_pain_points: pain_points,
        }
    }

    /// Generates value-first educational content strategies or AMA blueprints.
    pub fn generate_educational_strategy(
        profile: &CommunityProfile,
        topic: &str,
    ) -> ContentStrategy {
        ContentStrategy::EducationalGuide {
            title: format!("[Guide] Solving {} in r/{}", topic, profile.subreddit),
            target_subreddit: profile.subreddit.clone(),
            key_takeaways: vec![
                "Root cause breakdown based on community discussions".to_string(),
                "Step-by-step resolution without proprietary dependencies".to_string(),
                "Common mistakes to avoid".to_string(),
            ],
            outline: vec![
                "1. Introduction: The friction many community members encounter".to_string(),
                "2. Technical analysis of the problem".to_string(),
                "3. Worked open-source/free solution".to_string(),
                "4. Community Q&A invite (no pitch)".to_string(),
            ],
        }
    }

    /// Plans an authentic AMA (Ask Me Anything) focused on subject matter expertise.
    pub fn plan_ama(subreddit: &str, founder_role: &str, domain_theme: &str) -> ContentStrategy {
        ContentStrategy::AmaPlan {
            target_subreddit: subreddit.to_string(),
            host_role: founder_role.to_string(),
            core_theme: domain_theme.to_string(),
            anticipated_questions: vec![
                "What made you build this and what was the hardest technical challenge?"
                    .to_string(),
                "How do you handle rate limits and platform integrity rules?".to_string(),
                "What advice do you have for solo engineers starting in this space?".to_string(),
            ],
            community_value_deliverables: vec![
                "Transparent sharing of architecture decisions and mistakes made".to_string(),
                "Free checklist/rubric for participants".to_string(),
            ],
        }
    }

    /// Monitors brand mentions and drafts authentic, helpful remediation replies.
    pub fn remediate_brand_mention(
        mention: &BrandMention,
        product: &ProductContext,
    ) -> CommunityRemediationDraft {
        let (drafted_reply, tone) = match mention.sentiment {
            SentimentClassification::Negative | SentimentClassification::CriticalCrisis => {
                let reply = format!(
                    "Hey, {} founder here. Really appreciate you sharing this candid feedback about {}. You're completely right to expect better reliability here. We are actively investigating this specific friction point — no defensive excuses. If you're open to it, could you share more details so we can fix it for everyone?",
                    product.product_name, mention.snippet
                );
                (reply, "humble, transparent, ownership-taking".to_string())
            }
            SentimentClassification::Positive => {
                let reply = format!(
                    "Thanks so much for the kind words about {}! Glad the workflow was helpful for your setup. If you ever hit any roadblocks, let us know!",
                    product.product_name
                );
                (reply, "grateful, non-promotional".to_string())
            }
            SentimentClassification::Neutral => {
                let reply = format!(
                    "Appreciate the mention! Happy to answer any specific technical questions about how we designed {} if anyone in the thread is interested.",
                    product.product_name
                );
                (reply, "helpful, educational".to_string())
            }
        };

        CommunityRemediationDraft {
            mention_id: mention.id.clone(),
            original_snippet: mention.snippet.clone(),
            sentiment: mention.sentiment.clone(),
            drafted_reply,
            tone,
            community_first_validated: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_90_10_ratio_compliant() {
        let history = vec![
            EngagementRecord {
                id: "1".to_string(),
                subreddit: "SaaS".to_string(),
                is_promotional: false,
                summary: "Helpful guide".to_string(),
                timestamp_utc: 1,
            },
            EngagementRecord {
                id: "2".to_string(),
                subreddit: "SaaS".to_string(),
                is_promotional: false,
                summary: "Answer question".to_string(),
                timestamp_utc: 2,
            },
            EngagementRecord {
                id: "3".to_string(),
                subreddit: "SaaS".to_string(),
                is_promotional: false,
                summary: "Free resource".to_string(),
                timestamp_utc: 3,
            },
            EngagementRecord {
                id: "4".to_string(),
                subreddit: "SaaS".to_string(),
                is_promotional: false,
                summary: "Answer question".to_string(),
                timestamp_utc: 4,
            },
            EngagementRecord {
                id: "5".to_string(),
                subreddit: "SaaS".to_string(),
                is_promotional: false,
                summary: "Answer question".to_string(),
                timestamp_utc: 5,
            },
            EngagementRecord {
                id: "6".to_string(),
                subreddit: "SaaS".to_string(),
                is_promotional: false,
                summary: "Answer question".to_string(),
                timestamp_utc: 6,
            },
            EngagementRecord {
                id: "7".to_string(),
                subreddit: "SaaS".to_string(),
                is_promotional: false,
                summary: "Answer question".to_string(),
                timestamp_utc: 7,
            },
            EngagementRecord {
                id: "8".to_string(),
                subreddit: "SaaS".to_string(),
                is_promotional: false,
                summary: "Answer question".to_string(),
                timestamp_utc: 8,
            },
            EngagementRecord {
                id: "9".to_string(),
                subreddit: "SaaS".to_string(),
                is_promotional: false,
                summary: "Answer question".to_string(),
                timestamp_utc: 9,
            },
            EngagementRecord {
                id: "10".to_string(),
                subreddit: "SaaS".to_string(),
                is_promotional: true,
                summary: "Disclosed tool mention".to_string(),
                timestamp_utc: 10,
            },
        ];

        let audit = CommunityBuilderWorker::audit_90_10_ratio(&history);
        assert!(audit.is_compliant);
        assert_eq!(audit.value_percentage, 90.0);
    }

    #[test]
    fn test_audit_90_10_ratio_non_compliant() {
        let history = vec![
            EngagementRecord {
                id: "1".to_string(),
                subreddit: "SaaS".to_string(),
                is_promotional: false,
                summary: "Answer".to_string(),
                timestamp_utc: 1,
            },
            EngagementRecord {
                id: "2".to_string(),
                subreddit: "SaaS".to_string(),
                is_promotional: true,
                summary: "Promo".to_string(),
                timestamp_utc: 2,
            },
        ];

        let audit = CommunityBuilderWorker::audit_90_10_ratio(&history);
        assert!(!audit.is_compliant);
        assert_eq!(audit.value_percentage, 50.0);
        assert!(audit.advisory.contains("NON-COMPLIANT"));
    }

    #[test]
    fn test_subreddit_sentiment_analysis() {
        let threads = vec![
            RedditThread {
                id: "1".to_string(),
                subreddit: "indiehackers".to_string(),
                title: "Frustrated with broken marketing tools".to_string(),
                body: "Everything is terrible and spammy.".to_string(),
                author: "user1".to_string(),
                created_utc: 100,
                num_comments: 5,
                is_locked: false,
                is_archived: false,
            },
            RedditThread {
                id: "2".to_string(),
                subreddit: "indiehackers".to_string(),
                title: "Love the new indie hacker guide".to_string(),
                body: "Awesome resources!".to_string(),
                author: "user2".to_string(),
                created_utc: 200,
                num_comments: 10,
                is_locked: false,
                is_archived: false,
            },
        ];

        let report = CommunityBuilderWorker::analyze_subreddit_sentiment("indiehackers", &threads);
        assert_eq!(report.negative_count, 1);
        assert_eq!(report.positive_count, 1);
        assert_eq!(report.identified_pain_points.len(), 1);
    }

    #[test]
    fn test_remediate_negative_brand_mention() {
        let product = ProductContext::default();
        let mention = BrandMention {
            id: "m1".to_string(),
            subreddit: "SaaS".to_string(),
            author: "skeptic_user".to_string(),
            snippet: "AgenticReddit kept failing to fetch comments yesterday".to_string(),
            sentiment: SentimentClassification::Negative,
            urgency: 4,
        };

        let remediation = CommunityBuilderWorker::remediate_brand_mention(&mention, &product);
        assert!(remediation.community_first_validated);
        assert!(remediation.drafted_reply.contains("founder here"));
        assert!(remediation.drafted_reply.contains("no defensive excuses"));
    }
}
