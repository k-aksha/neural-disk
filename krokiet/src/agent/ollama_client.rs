use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

const REQUEST_TIMEOUT_SECS: u64 = 120;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChatMessage {
    pub(crate) role: String,
    #[serde(default)]
    pub(crate) content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) tool_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) tool_calls: Option<Vec<ToolCall>>,
}

impl ChatMessage {
    pub(crate) fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
            tool_name: None,
            tool_calls: None,
        }
    }

    pub(crate) fn tool(tool_name: String, content: String) -> Self {
        Self {
            role: "tool".to_string(),
            content,
            tool_name: Some(tool_name),
            tool_calls: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ToolCall {
    pub(crate) function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ToolCallFunction {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) arguments: Value,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolSpec {
    #[serde(rename = "type")]
    pub(crate) kind: &'static str,
    pub(crate) function: ToolSpecFunction,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolSpecFunction {
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
    pub(crate) parameters: Value,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    stream: bool,
    tools: &'a [ToolSpec],
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ChatMessage,
}

#[derive(Debug)]
pub(crate) enum OllamaError {
    Request(String),
    Decode(String),
}

impl std::fmt::Display for OllamaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(e) => write!(f, "Could not reach Ollama at the configured server URL: {e}"),
            Self::Decode(e) => write!(f, "Unexpected response from Ollama: {e}"),
        }
    }
}

pub(crate) struct OllamaClient {
    base_url: String,
    model: String,
    http: reqwest::blocking::Client,
}

impl OllamaClient {
    pub(crate) fn new(base_url: String, model: String) -> Self {
        Self {
            base_url,
            model,
            http: reqwest::blocking::Client::new(),
        }
    }

    pub(crate) fn chat(&self, history: &[ChatMessage], tools: &[ToolSpec]) -> Result<ChatMessage, OllamaError> {
        let request = ChatRequest {
            model: &self.model,
            messages: history,
            stream: false,
            tools,
        };

        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));
        let response = self
            .http
            .post(url)
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .json(&request)
            .send()
            .map_err(|e| OllamaError::Request(e.to_string()))?;

        let status = response.status();
        let body = response.text().map_err(|e| OllamaError::Decode(e.to_string()))?;

        if !status.is_success() {
            return Err(OllamaError::Request(format!("{status}: {}", error_message_from_body(body))));
        }

        let parsed: ChatResponse = serde_json::from_str(&body).map_err(|e| OllamaError::Decode(e.to_string()))?;
        Ok(parsed.message)
    }
}

/// Ollama returns a JSON body like `{"error": "model ... not found"}` on failure - read it
/// instead of letting `error_for_status()` collapse everything to a bare HTTP code.
fn error_message_from_body(body: String) -> String {
    serde_json::from_str::<Value>(&body)
        .ok()
        .and_then(|v| v.get("error").and_then(Value::as_str).map(str::to_string))
        .unwrap_or(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_message_user_has_no_tool_name_or_tool_calls() {
        let message = ChatMessage::user("hello".to_string());
        assert_eq!(message.role, "user");
        assert_eq!(message.content, "hello");
        assert!(message.tool_name.is_none());
        assert!(message.tool_calls.is_none());
    }

    #[test]
    fn chat_message_tool_carries_its_name() {
        let message = ChatMessage::tool("run_scan".to_string(), "{}".to_string());
        assert_eq!(message.role, "tool");
        assert_eq!(message.tool_name.as_deref(), Some("run_scan"));
    }

    #[test]
    fn chat_request_serializes_in_ollamas_expected_shape() {
        let history = vec![ChatMessage::user("find duplicates".to_string())];
        let tools = vec![ToolSpec {
            kind: "function",
            function: ToolSpecFunction {
                name: "run_scan",
                description: "Run a scan",
                parameters: serde_json::json!({ "type": "object", "properties": {} }),
            },
        }];
        let request = ChatRequest {
            model: "llama3.1",
            messages: &history,
            stream: false,
            tools: &tools,
        };

        let value = serde_json::to_value(&request).expect("ChatRequest must serialize");
        assert_eq!(value["model"], "llama3.1");
        assert_eq!(value["stream"], false);
        assert_eq!(value["messages"][0]["role"], "user");
        assert_eq!(value["messages"][0]["content"], "find duplicates");
        assert_eq!(value["tools"][0]["type"], "function");
        assert_eq!(value["tools"][0]["function"]["name"], "run_scan");
    }

    #[test]
    fn chat_message_omits_tool_name_and_tool_calls_when_absent() {
        let message = ChatMessage::user("hi".to_string());
        let value = serde_json::to_value(&message).expect("must serialize");
        assert!(value.get("tool_name").is_none());
        assert!(value.get("tool_calls").is_none());
    }

    #[test]
    fn chat_response_deserializes_plain_text_reply() {
        let body = r#"{"model":"llama3.1","message":{"role":"assistant","content":"Hello there"},"done":true}"#;
        let parsed: ChatResponse = serde_json::from_str(body).expect("must parse a plain-text Ollama response");
        assert_eq!(parsed.message.role, "assistant");
        assert_eq!(parsed.message.content, "Hello there");
        assert!(parsed.message.tool_calls.is_none());
    }

    #[test]
    fn chat_response_deserializes_tool_call_reply() {
        // Matches Ollama's documented /api/chat tool-calling response shape.
        let body = r#"{
            "model": "llama3.1",
            "message": {
                "role": "assistant",
                "content": "",
                "tool_calls": [
                    { "function": { "name": "run_scan", "arguments": { "tool": "duplicate_files" } } }
                ]
            },
            "done": true
        }"#;
        let parsed: ChatResponse = serde_json::from_str(body).expect("must parse a tool-calling Ollama response");
        let tool_calls = parsed.message.tool_calls.expect("tool_calls must be present");
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].function.name, "run_scan");
        assert_eq!(tool_calls[0].function.arguments["tool"], "duplicate_files");
    }

    #[test]
    fn chat_response_ignores_unknown_top_level_fields() {
        let body = r#"{"model":"llama3.1","created_at":"2026-01-01T00:00:00Z","message":{"role":"assistant","content":"ok"},"done":true,"total_duration":123}"#;
        let parsed: ChatResponse = serde_json::from_str(body).expect("unknown fields must not break parsing");
        assert_eq!(parsed.message.content, "ok");
    }

    #[test]
    fn error_message_from_body_extracts_ollamas_error_field() {
        let body = r#"{"error":"model 'llama3.1' not found, try pulling it first"}"#.to_string();
        assert_eq!(error_message_from_body(body), "model 'llama3.1' not found, try pulling it first");
    }

    #[test]
    fn error_message_from_body_falls_back_to_raw_body_when_not_json() {
        let body = "internal server error".to_string();
        assert_eq!(error_message_from_body(body.clone()), body);
    }

    #[test]
    fn error_message_from_body_falls_back_to_raw_body_when_no_error_field() {
        let body = r#"{"unexpected":"shape"}"#.to_string();
        assert_eq!(error_message_from_body(body.clone()), body);
    }
}
