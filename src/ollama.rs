use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

// ── public types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    #[serde(default)]
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    #[serde(default)]
    pub id: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Value,
    // Ollama may include an "index" field; accept but ignore it
    #[allow(dead_code)]
    #[serde(default, skip_serializing)]
    pub index: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Value>,
    pub options: Options,
}

#[derive(Debug, Clone, Serialize)]
pub struct Options {
    pub temperature: f64,
    pub num_ctx: u32,
}

// ── internal deserialization ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RawChunk {
    message: RawMessage,
    #[serde(default)]
    done: bool,
}

#[derive(Debug, Deserialize)]
struct RawMessage {
    // Actual response text
    #[serde(default)]
    content: String,
    // Qwen3 / Ollama native thinking field (separate from content)
    #[serde(default)]
    thinking: Option<String>,
    // Tool call requests from the model (come in a done=false chunk)
    #[serde(default)]
    tool_calls: Option<Vec<ToolCall>>,
}

// ── Ollama client ────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Client {
    base: String,
    agent: ureq::Agent,
}

impl Client {
    pub fn new(base_url: &str) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(10))
            .timeout_read(std::time::Duration::from_secs(600))
            .build();
        Self {
            base: base_url.trim_end_matches('/').to_string(),
            agent,
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.agent
            .get(&format!("{}/api/tags", self.base))
            .call()
            .is_ok()
    }

    /// Stream a chat response.
    ///
    /// `show_thinking` – if true, thinking tokens are also forwarded to `on_token`
    /// (displayed dim in the caller).
    ///
    /// Returns `(content, tool_calls)`:
    /// - `content`    – accumulated visible text (may be empty when tool calls fired)
    /// - `tool_calls` – Some(...) when the model wants to invoke tools
    pub fn chat_stream<F>(
        &self,
        request: &ChatRequest,
        show_thinking: bool,
        cancel: Arc<AtomicBool>,
        mut on_token: F,
    ) -> Result<(String, Option<Vec<ToolCall>>), String>
    where
        F: FnMut(&str, bool), // (token, is_thinking)
    {
        let mut req = request.clone();
        req.stream = true;

        let url = format!("{}/api/chat", self.base);
        let resp = match self.agent.post(&url).send_json(&req) {
            Ok(r) => r,
            Err(ureq::Error::Status(code, r)) => {
                let body = r.into_string().unwrap_or_default();
                return Err(format!("Ollama {code}: {body}"));
            }
            Err(e) => return Err(format!("Connection error: {e}")),
        };

        let reader = BufReader::new(resp.into_reader());
        let mut content = String::new();
        let mut tool_calls: Option<Vec<ToolCall>> = None;

        for line in reader.lines() {
            if cancel.load(Ordering::Relaxed) {
                return Err("__cancelled__".into());
            }

            let line = line.map_err(|e| format!("Stream read error: {e}"))?;
            if line.is_empty() {
                continue;
            }

            let chunk: RawChunk = serde_json::from_str(&line)
                .map_err(|e| format!("Stream parse error: {e}"))?;

            // Thinking tokens (Ollama native thinking field, e.g. Qwen3)
            if show_thinking {
                if let Some(ref t) = chunk.message.thinking {
                    if !t.is_empty() {
                        on_token(t, true);
                    }
                }
            }

            // Response content tokens
            if !chunk.message.content.is_empty() {
                on_token(&chunk.message.content.clone(), false);
                content.push_str(&chunk.message.content);
            }

            // Tool calls arrive in a pre-done chunk (done=false)
            if let Some(tc) = chunk.message.tool_calls {
                if !tc.is_empty() {
                    tool_calls = Some(tc);
                }
            }

            if chunk.done {
                break;
            }
        }

        Ok((content, tool_calls))
    }
}
