use crate::agents::opportunity::RedditThread;
use crate::ai::prompts::build_anti_slop_prompt;
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AiError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Vertex AI API error (Status {status}): {message}")]
    Api { status: u16, message: String },
    #[error("Empty candidates returned from Vertex AI model")]
    EmptyResponse,
    #[error("Response parsing error: {0}")]
    Parse(String),
    #[error("Authentication error: {0}")]
    Auth(String),
}

#[derive(Debug, Clone)]
pub struct VertexAiClient {
    pub project_id: String,
    pub location: String,
    pub model: String,
    client: reqwest::Client,
}

impl VertexAiClient {
    pub fn new(project_id: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            location: location.into(),
            model: "gemini-1.5-pro".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn endpoint_url(&self) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
            self.location, self.project_id, self.location, self.model
        )
    }

    pub fn parse_vertex_response(val: &Value) -> Result<String, AiError> {
        let text = val["candidates"]
            .as_array()
            .and_then(|c| c.first())
            .and_then(|c| c["content"]["parts"].as_array())
            .and_then(|p| p.first())
            .and_then(|p| p["text"].as_str())
            .ok_or(AiError::EmptyResponse)?;

        Ok(text.trim().to_string())
    }

    /// Generates an anti-slop reply using Vertex AI Gemini
    pub async fn generate_anti_slop_reply(
        &self,
        thread: &RedditThread,
        angle: &str,
        token_override: Option<&str>,
    ) -> Result<String, AiError> {
        let token = if let Some(t) = token_override {
            t.to_string()
        } else {
            // Attempt to resolve access token via gcloud CLI when running in local / dev
            let output = std::process::Command::new("gcloud")
                .args(["auth", "print-access-token"])
                .output()
                .map_err(|e| AiError::Auth(format!("Failed to execute gcloud: {}", e)))?;
            if !output.status.success() {
                return Err(AiError::Auth(format!(
                    "gcloud auth print-access-token failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        };

        let prompt = build_anti_slop_prompt(thread, angle);
        let payload = json!({
            "contents": [{
                "role": "user",
                "parts": [{ "text": prompt }]
            }],
            "generationConfig": {
                "temperature": 0.2,
                "maxOutputTokens": 1024,
                "topP": 0.95
            }
        });

        let resp = self
            .client
            .post(self.endpoint_url())
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await?;

        let status = resp.status();
        let body: Value = resp.json().await?;

        if !status.is_success() {
            let msg = body["error"]["message"]
                .as_str()
                .unwrap_or("Unknown Vertex AI error");
            return Err(AiError::Api {
                status: status.as_u16(),
                message: msg.to_string(),
            });
        }

        Self::parse_vertex_response(&body)
    }
}
