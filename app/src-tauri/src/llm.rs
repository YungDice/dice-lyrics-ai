use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub system: String,
    pub prompt: String,
}

/// The one seam between the app and any local model backend. Both the
/// Analyze and Generate flows go through this trait exclusively, so tests
/// can substitute `FakeLlmClient` instead of talking to a real Ollama.
#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn generate(&self, req: LlmRequest) -> Result<String, String>;
}

pub struct OllamaClient {
    base_url: String,
    model: String,
    http: reqwest::Client,
}

impl OllamaClient {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            base_url,
            model,
            http: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct GenerateRequestBody<'a> {
    model: &'a str,
    system: &'a str,
    prompt: &'a str,
    stream: bool,
    options: GenerateOptions,
}

#[derive(Serialize)]
struct GenerateOptions {
    // dolphin-mistral:7b supports up to 32K; Ollama's own default (2048) is
    // far too small to hold a style profile plus reference lyric excerpts.
    num_ctx: u32,
}

#[derive(Deserialize)]
struct GenerateResponseBody {
    response: String,
}

#[async_trait]
impl LlmClient for OllamaClient {
    async fn generate(&self, req: LlmRequest) -> Result<String, String> {
        let url = format!("{}/api/generate", self.base_url);
        let body = GenerateRequestBody {
            model: &self.model,
            system: &req.system,
            prompt: &req.prompt,
            stream: false,
            options: GenerateOptions { num_ctx: 32768 },
        };

        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                format!(
                    "Could not reach Ollama at {}: {}. Is Ollama running?",
                    self.base_url, e
                )
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Ollama returned {}: {}", status, text));
        }

        let parsed: GenerateResponseBody = resp
            .json()
            .await
            .map_err(|e| format!("Could not parse Ollama response: {}", e))?;
        Ok(parsed.response)
    }
}

/// Test double for `LlmClient` — returns a canned response without any
/// network call, so backend tests never require a running Ollama instance.
pub struct FakeLlmClient {
    pub response: String,
}

#[async_trait]
impl LlmClient for FakeLlmClient {
    async fn generate(&self, _req: LlmRequest) -> Result<String, String> {
        Ok(self.response.clone())
    }
}
