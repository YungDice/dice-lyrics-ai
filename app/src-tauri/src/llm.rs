use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub system: String,
    pub prompt: String,
    pub temperature: f32,
}

/// The one seam between the app and any local model backend. Both the
/// Analyze and Generate flows go through this trait exclusively, so tests
/// can substitute `FakeLlmClient` instead of talking to a real Ollama.
#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn generate(&self, req: LlmRequest) -> Result<String, String>;

    /// Streaming variant: `on_token` is called with each new text fragment
    /// as the model produces it; the full response is returned at the end.
    async fn generate_stream(
        &self,
        req: LlmRequest,
        on_token: &mut (dyn FnMut(String) + Send),
    ) -> Result<String, String>;
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

    fn body<'a>(&'a self, req: &'a LlmRequest, stream: bool) -> GenerateRequestBody<'a> {
        GenerateRequestBody {
            model: &self.model,
            system: &req.system,
            prompt: &req.prompt,
            stream,
            options: GenerateOptions {
                num_ctx: 32768,
                temperature: req.temperature,
                top_p: 0.95,
                repeat_penalty: 1.1,
                num_predict: 1600,
            },
        }
    }

    async fn send(&self, req: &LlmRequest, stream: bool) -> Result<reqwest::Response, String> {
        let url = format!("{}/api/generate", self.base_url);
        let resp = self
            .http
            .post(&url)
            .json(&self.body(req, stream))
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
        Ok(resp)
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
    // far too small to hold a rich style profile plus reference excerpts.
    num_ctx: u32,
    temperature: f32,
    top_p: f32,
    repeat_penalty: f32,
    num_predict: i32,
}

#[derive(Deserialize)]
struct GenerateResponseBody {
    response: String,
}

#[derive(Deserialize)]
struct StreamChunk {
    #[serde(default)]
    response: String,
    #[serde(default)]
    error: Option<String>,
}

#[async_trait]
impl LlmClient for OllamaClient {
    async fn generate(&self, req: LlmRequest) -> Result<String, String> {
        let resp = self.send(&req, false).await?;
        let parsed: GenerateResponseBody = resp
            .json()
            .await
            .map_err(|e| format!("Could not parse Ollama response: {}", e))?;
        Ok(parsed.response)
    }

    async fn generate_stream(
        &self,
        req: LlmRequest,
        on_token: &mut (dyn FnMut(String) + Send),
    ) -> Result<String, String> {
        let mut resp = self.send(&req, true).await?;
        let mut full = String::new();
        let mut buf: Vec<u8> = Vec::new();
        // Ollama streams newline-delimited JSON objects.
        while let Some(chunk) = resp
            .chunk()
            .await
            .map_err(|e| format!("Stream error from Ollama: {}", e))?
        {
            buf.extend_from_slice(&chunk);
            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buf.drain(..=pos).collect();
                let line = String::from_utf8_lossy(&line);
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Ok(parsed) = serde_json::from_str::<StreamChunk>(line) {
                    if let Some(err) = parsed.error {
                        return Err(format!("Ollama error: {}", err));
                    }
                    if !parsed.response.is_empty() {
                        full.push_str(&parsed.response);
                        on_token(parsed.response);
                    }
                }
            }
        }
        Ok(full)
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

    async fn generate_stream(
        &self,
        _req: LlmRequest,
        on_token: &mut (dyn FnMut(String) + Send),
    ) -> Result<String, String> {
        on_token(self.response.clone());
        Ok(self.response.clone())
    }
}
