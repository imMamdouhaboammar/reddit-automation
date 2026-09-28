use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditIdentity {
    pub name: String,
    pub id: String,
    pub created_utc: f64,
    pub link_karma: i64,
    pub comment_karma: i64,
    pub total_karma: Option<i64>,
    pub is_mod: bool,
    pub is_gold: bool,
    pub over_18: bool,
    pub has_verified_email: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KarmaBreakdownItem {
    pub sr: String,
    pub comment_karma: i64,
    pub link_karma: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KarmaResponse {
    pub data: Vec<KarmaBreakdownItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RedditPreferences {
    pub default_comment_sort: Option<String>,
    pub email_digests: Option<bool>,
    pub email_messages: Option<bool>,
    pub email_unsubscribe_all: Option<bool>,
    pub feed_recommendations_enabled: Option<bool>,
    pub hide_from_robots: Option<bool>,
    pub label_nsfw: Option<bool>,
    pub nightmode: Option<bool>,
    pub over_18: Option<bool>,
    pub search_include_over_18: Option<bool>,
    pub show_presence: Option<bool>,
    pub threaded_messages: Option<bool>,
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubredditRule {
    pub short_name: String,
    pub description: String,
    pub violation_reason: Option<String>,
    pub created_utc: Option<f64>,
    pub priority: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubredditRulesResponse {
    pub rules: Vec<SubredditRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostRequirements {
    pub title_regexes: Option<Vec<String>>,
    pub body_blacklisted_strings: Option<Vec<String>>,
    pub body_required_strings: Option<Vec<String>>,
    pub body_restriction: Option<String>,
    pub is_flair_required: Option<bool>,
    pub title_text_min_length: Option<usize>,
    pub title_text_max_length: Option<usize>,
    pub body_text_min_length: Option<usize>,
    pub body_text_max_length: Option<usize>,
    pub guidelines_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubredditAbout {
    pub display_name: String,
    pub title: String,
    pub public_description: String,
    pub subscribers: u64,
    pub active_user_count: Option<u64>,
    pub over18: bool,
    pub subreddit_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditSubmission {
    pub id: String,
    pub name: String, // e.g. "t3_xxx"
    pub title: String,
    pub author: String,
    pub subreddit: String,
    pub selftext: String,
    pub url: String,
    pub score: i64,
    pub num_comments: i64,
    pub permalink: String,
    pub created_utc: f64,
    pub over_18: bool,
    pub spoiler: bool,
    pub is_self: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditComment {
    pub id: String,
    pub name: String, // e.g. "t1_xxx"
    pub parent_id: String,
    pub link_id: String,
    pub author: String,
    pub body: String,
    pub score: i64,
    pub permalink: String,
    pub created_utc: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditMessage {
    pub id: String,
    pub name: String,
    pub author: Option<String>,
    pub subject: String,
    pub body: String,
    pub was_comment: bool,
    pub new: bool,
    pub created_utc: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResponseDetail {
    pub url: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResponseJson {
    pub errors: Vec<Vec<serde_json::Value>>,
    pub data: Option<SubmitResponseDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResponse {
    pub json: SubmitResponseJson,
}
