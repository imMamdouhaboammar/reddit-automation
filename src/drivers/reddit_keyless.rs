use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, USER_AGENT};
use serde_json::Value;
use std::collections::HashSet;
use std::time::Duration;
use tracing::{debug, warn};

use crate::agents::opportunity::RedditThread;
use crate::drivers::DriverError;

/// Noise words to ignore during relevance scoring (matching last30days-skill catalog).
pub const NOISE_WORDS: &[&str] = &[
    "best",
    "top",
    "good",
    "great",
    "awesome",
    "killer",
    "latest",
    "new",
    "news",
    "update",
    "updates",
    "trending",
    "hottest",
    "popular",
    "practices",
    "features",
    "tips",
    "recommendations",
    "advice",
    "prompt",
    "prompts",
    "prompting",
    "methods",
    "strategies",
    "approaches",
    "how",
    "to",
    "the",
    "a",
    "an",
    "for",
    "with",
    "of",
    "in",
    "on",
    "is",
    "are",
    "what",
    "which",
    "guide",
    "tutorial",
    "using",
    "and",
    "or",
    "i",
    "my",
    "we",
    "our",
    "you",
    "your",
    "it",
    "this",
    "that",
];

/// Keyless Reddit Driver inspired by the last30days-skill multi-tier discovery pipeline.
/// Provides resilient Reddit thread extraction via public JSON listings and Arctic Shift archive fallback.
#[derive(Clone)]
pub struct RedditKeylessDriver {
    client: reqwest::Client,
    custom_user_agent: String,
}

impl Default for RedditKeylessDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl RedditKeylessDriver {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            custom_user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36".to_string(),
        }
    }

    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.custom_user_agent = ua.into();
        self
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&self.custom_user_agent)
                .unwrap_or_else(|_| HeaderValue::from_static("reddit-keyless-agent/1.0")),
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/json, text/plain, */*"),
        );
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
        headers
    }

    /// Fetches public subreddit posts (hot, new, top) via reddit.com/r/{sub}/{sort}.json
    pub async fn fetch_subreddit_listing(
        &self,
        subreddit: &str,
        sort: &str,
        limit: usize,
    ) -> Result<Vec<RedditThread>, DriverError> {
        let clean_sub = subreddit.trim_start_matches("r/").trim_start_matches('/');
        let valid_sort = match sort {
            "new" | "top" | "rising" => sort,
            _ => "hot",
        };

        let url = format!(
            "https://www.reddit.com/r/{}/{}.json?limit={}",
            clean_sub,
            valid_sort,
            limit.min(100)
        );

        debug!(url = %url, "Fetching keyless reddit listing");

        let resp = self
            .client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await
            .map_err(|e| DriverError::Api {
                status: 0,
                message: format!("Keyless request failed: {}", e),
            })?;

        let status = resp.status();
        if !status.is_success() {
            return Err(DriverError::Api {
                status: status.as_u16(),
                message: format!("Subreddit listing returned HTTP {}", status),
            });
        }

        let json_val: Value = resp.json().await.map_err(|e| DriverError::Api {
            status: 200,
            message: format!("Failed to parse JSON listing: {}", e),
        })?;

        let threads = Self::parse_reddit_listing_json(&json_val, clean_sub);
        Ok(threads)
    }

    /// Queries Arctic Shift / Pushshift archive API for targeted threads.
    /// Acts as resilient fallback when Reddit public JSON endpoints 403 or rate-limit.
    pub async fn fetch_arctic_archive(
        &self,
        query: &str,
        subreddit: Option<&str>,
        limit: usize,
    ) -> Result<Vec<RedditThread>, DriverError> {
        let mut url = format!(
            "https://arctic-shift.photon-reddit.com/api/posts/search?title={}&limit={}",
            urlencoding::encode(query),
            limit.min(100)
        );

        if let Some(sub) = subreddit {
            let clean_sub = sub.trim_start_matches("r/").trim_start_matches('/');
            url.push_str(&format!("&subreddit={}", clean_sub));
        }

        debug!(url = %url, "Fetching arctic shift archive");

        let resp = self
            .client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await
            .map_err(|e| DriverError::Api {
                status: 0,
                message: format!("Arctic shift request failed: {}", e),
            })?;

        if !resp.status().is_success() {
            return Err(DriverError::Api {
                status: resp.status().as_u16(),
                message: format!("Arctic shift returned HTTP {}", resp.status()),
            });
        }

        let json_val: Value = resp.json().await.map_err(|e| DriverError::Api {
            status: 200,
            message: format!("Failed to parse Arctic JSON: {}", e),
        })?;

        let threads = Self::parse_arctic_json(&json_val);
        Ok(threads)
    }

    /// Aggregates threads across multiple subreddits and search queries using multi-tier fallback.
    /// Mirrors last30days: attempts public listing first, supplements with Arctic Shift on failure.
    pub async fn discover_threads(
        &self,
        subreddits: &[String],
        search_phrases: &[String],
        limit_per_sub: usize,
    ) -> Vec<RedditThread> {
        let mut collected: Vec<RedditThread> = Vec::new();
        let mut seen_ids: HashSet<String> = HashSet::new();

        // Tier 1: Public listings across target subreddits
        for sub in subreddits {
            match self
                .fetch_subreddit_listing(sub, "hot", limit_per_sub)
                .await
            {
                Ok(posts) => {
                    for post in posts {
                        if seen_ids.insert(post.id.clone()) {
                            collected.push(post);
                        }
                    }
                }
                Err(err) => {
                    warn!(subreddit = %sub, error = %err, "Public listing failed, attempting Arctic Shift fallback");
                    // Fallback to Arctic Shift for this subreddit
                    for phrase in search_phrases.iter().take(2) {
                        if let Ok(posts) = self
                            .fetch_arctic_archive(phrase, Some(sub), limit_per_sub)
                            .await
                        {
                            for post in posts {
                                if seen_ids.insert(post.id.clone()) {
                                    collected.push(post);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Tier 2: Search phrases via Arctic archive if needed
        if collected.is_empty() && !search_phrases.is_empty() {
            for phrase in search_phrases {
                if let Ok(posts) = self.fetch_arctic_archive(phrase, None, limit_per_sub).await {
                    for post in posts {
                        if seen_ids.insert(post.id.clone()) {
                            collected.push(post);
                        }
                    }
                }
            }
        }

        collected
    }

    /// Calculates token-overlap relevance score between thread text and target terms (last30days relevance model).
    pub fn calculate_relevance(title: &str, body: &str, target_terms: &[String]) -> f64 {
        if target_terms.is_empty() {
            return 1.0;
        }

        let combined = format!("{} {}", title, body).to_lowercase();
        let noise: HashSet<&str> = NOISE_WORDS.iter().copied().collect();

        let mut matched = 0usize;
        let mut total_meaningful_terms = 0usize;

        for term in target_terms {
            let term_lower = term.to_lowercase();
            let tokens: Vec<&str> = term_lower
                .split_whitespace()
                .filter(|t| !noise.contains(*t) && t.len() > 2)
                .collect();

            if tokens.is_empty() {
                continue;
            }

            total_meaningful_terms += 1;
            if tokens.iter().all(|t| combined.contains(*t)) {
                matched += 1;
            }
        }

        if total_meaningful_terms == 0 {
            return 0.5;
        }

        (matched as f64 / total_meaningful_terms as f64).clamp(0.0, 1.0)
    }

    /// Applies last30days bounded engagement ranking:
    /// Score = Relevance + min(0.25, log10(upvotes + comments + 1) / 20.0)
    pub fn rank_composite_score(relevance: f64, num_comments: u32, upvotes: i64) -> f64 {
        let total_engagement = (num_comments as f64) + (upvotes.max(0) as f64);
        let bonus = (f64::log10(total_engagement + 1.0) / 20.0).min(0.25);
        relevance + bonus
    }

    fn parse_reddit_listing_json(val: &Value, fallback_sub: &str) -> Vec<RedditThread> {
        let mut threads = Vec::new();
        if let Some(children) = val["data"]["children"].as_array() {
            for child in children {
                let data = &child["data"];
                let id = data["id"].as_str().unwrap_or("").to_string();
                if id.is_empty() {
                    continue;
                }

                let title = data["title"].as_str().unwrap_or("").to_string();
                let body = data["selftext"].as_str().unwrap_or("").to_string();
                let author = data["author"].as_str().unwrap_or("[deleted]").to_string();
                let subreddit = data["subreddit"]
                    .as_str()
                    .unwrap_or(fallback_sub)
                    .to_string();
                let created_utc = data["created_utc"].as_f64().unwrap_or(0.0) as i64;
                let num_comments = data["num_comments"].as_u64().unwrap_or(0) as u32;
                let is_locked = data["locked"].as_bool().unwrap_or(false);
                let is_archived = data["archived"].as_bool().unwrap_or(false);
                let permalink = data["permalink"].as_str().unwrap_or("");
                let url = if permalink.is_empty() {
                    format!("https://reddit.com/r/{}/comments/{}/", subreddit, id)
                } else if permalink.starts_with("http") {
                    permalink.to_string()
                } else {
                    format!("https://reddit.com{}", permalink)
                };

                threads.push(RedditThread {
                    id,
                    subreddit,
                    title,
                    body,
                    author,
                    created_utc,
                    num_comments,
                    is_locked,
                    is_archived,
                    url,
                });
            }
        }
        threads
    }

    fn parse_arctic_json(val: &Value) -> Vec<RedditThread> {
        let mut threads = Vec::new();
        let items = if let Some(arr) = val["data"].as_array() {
            arr
        } else if let Some(arr) = val.as_array() {
            arr
        } else {
            return threads;
        };

        for item in items {
            let id = item["id"].as_str().unwrap_or("").to_string();
            if id.is_empty() {
                continue;
            }

            let title = item["title"].as_str().unwrap_or("").to_string();
            let body = item["selftext"].as_str().unwrap_or("").to_string();
            let author = item["author"].as_str().unwrap_or("[deleted]").to_string();
            let subreddit = item["subreddit"].as_str().unwrap_or("unknown").to_string();
            let created_utc = item["created_utc"].as_i64().unwrap_or(0);
            let num_comments = item["num_comments"].as_u64().unwrap_or(0) as u32;

            let permalink = item["permalink"].as_str().unwrap_or("");
            let url = if permalink.is_empty() {
                format!("https://reddit.com/r/{}/comments/{}/", subreddit, id)
            } else if permalink.starts_with("http") {
                permalink.to_string()
            } else {
                format!("https://reddit.com{}", permalink)
            };

            threads.push(RedditThread {
                id,
                subreddit,
                title,
                body,
                author,
                created_utc,
                num_comments,
                is_locked: false,
                is_archived: false,
                url,
            });
        }
        threads
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_overlap_relevance_and_noise_filtering() {
        let title = "Looking for best alternative to spammy reddit bots";
        let body = "I am tired of wasting hours on reddit with generic tools that get banned.";
        let targets = vec![
            "reddit bots".to_string(),
            "wasting hours".to_string(),
            "unrelated search".to_string(),
        ];

        let relevance = RedditKeylessDriver::calculate_relevance(title, body, &targets);
        assert!(
            relevance > 0.6,
            "Relevance should match 2 out of 3 targets, got {}",
            relevance
        );
    }

    #[test]
    fn test_rank_composite_score_bounds_viral_engagement() {
        let on_topic_relevance = 0.90;
        let viral_off_topic_relevance = 0.10;

        let on_topic_score = RedditKeylessDriver::rank_composite_score(on_topic_relevance, 5, 10);
        let viral_score =
            RedditKeylessDriver::rank_composite_score(viral_off_topic_relevance, 5000, 20000);

        // Even with 25,000 engagement, viral post cannot overcome 0.25 bonus cap
        assert!(
            on_topic_score > viral_score,
            "On-topic ({}) must outrank viral off-topic ({})",
            on_topic_score,
            viral_score
        );
    }

    #[test]
    fn test_parse_listing_json() {
        let json_str = r#"{
            "kind": "Listing",
            "data": {
                "children": [
                    {
                        "kind": "t3",
                        "data": {
                            "id": "1abcde",
                            "title": "What tools do you use for customer research?",
                            "selftext": "Looking for honest recommendations.",
                            "author": "founder_bob",
                            "subreddit": "SaaS",
                            "created_utc": 1700000000.0,
                            "num_comments": 14,
                            "locked": false,
                            "archived": false
                        }
                    }
                ]
            }
        }"#;

        let val: Value = serde_json::from_str(json_str).unwrap();
        let threads = RedditKeylessDriver::parse_reddit_listing_json(&val, "SaaS");
        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0].id, "1abcde");
        assert_eq!(threads[0].subreddit, "SaaS");
        assert_eq!(threads[0].num_comments, 14);
    }
}
