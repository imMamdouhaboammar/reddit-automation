use reddit_agent::agents::opportunity::RedditThread;
use reddit_agent::ai::prompts::build_anti_slop_prompt;
use reddit_agent::ai::vertex::VertexAiClient;

#[test]
fn test_anti_slop_prompt_construction() {
    let thread = RedditThread {
        id: "1xnhs2u".to_string(),
        subreddit: "vibecoding".to_string(),
        title: "Millions upload raw passports".to_string(),
        body: "How to avoid WebKit downsampling in canvas?".to_string(),
        author: "RootDev".to_string(),
        created_utc: 1700000000,
        num_comments: 0,
        is_locked: false,
        is_archived: false,
        url: "https://reddit.com/r/vibecoding/comments/1xnhs2u".to_string(),
    };

    let prompt = build_anti_slop_prompt(&thread, "problem_solving");
    assert!(prompt.contains("vibecoding"));
    assert!(prompt.contains("Millions upload raw passports"));
    assert!(prompt.contains("ANTI-SLOP INVARIANTS"));
    assert!(prompt.contains("Direct payload delivery"));
}

#[tokio::test]
async fn test_vertex_ai_response_parsing_and_mock() {
    let mock_json = serde_json::json!({
        "candidates": [{
            "content": {
                "parts": [{
                    "text": "The main bottleneck in Safari is canvas downsampling beyond 16.7MP."
                }]
            },
            "finishReason": "STOP"
        }]
    });

    let parsed = VertexAiClient::parse_vertex_response(&mock_json).unwrap();
    assert_eq!(
        parsed,
        "The main bottleneck in Safari is canvas downsampling beyond 16.7MP."
    );
}
