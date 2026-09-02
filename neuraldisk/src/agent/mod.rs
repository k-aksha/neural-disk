pub(crate) mod ollama_client;
pub(crate) mod tools;

use std::sync::Mutex;

use ollama_client::{ChatMessage, OllamaClient};
use tools::AgentContext;

/// A model can chain several tool calls (e.g. scan, then select-and-propose-delete) before
/// answering in plain text. This bounds how many round trips one user message can trigger,
/// so a confused model can't loop forever driving the app.
const MAX_TOOL_ITERATIONS: usize = 6;

/// Holds only the conversation history - the Ollama connection is built fresh per message (see
/// `connect_agent_chat.rs`) so a Settings change (server URL/model) takes effect on the very
/// next message instead of requiring an app restart.
pub(crate) struct AgentSession {
    history: Mutex<Vec<ChatMessage>>,
}

impl AgentSession {
    pub(crate) fn new() -> Self {
        Self { history: Mutex::new(Vec::new()) }
    }
}

/// Runs the send -> maybe-call-tools -> send-again loop for one user message and returns every
/// assistant/tool message produced along the way, in order, for the caller to render in the chat.
pub(crate) fn handle_user_message(session: &AgentSession, client: &OllamaClient, ctx: &AgentContext, user_message: String) -> Result<Vec<ChatMessage>, String> {
    // A panic elsewhere while this lock is held (e.g. deep in a tool call) would otherwise
    // permanently poison it and break the chat feature for the rest of the process's life -
    // recovering keeps one bad turn from taking down every turn after it.
    let mut history = session.history.lock().unwrap_or_else(|poisoned| {
        session.history.clear_poison();
        poisoned.into_inner()
    });
    history.push(ChatMessage::user(user_message));

    let mut produced = Vec::new();

    for _ in 0..MAX_TOOL_ITERATIONS {
        let response = client.chat(&history, tools::tool_specs()).map_err(|e| e.to_string())?;
        history.push(response.clone());
        produced.push(response.clone());

        let tool_calls = response.tool_calls.unwrap_or_default();
        if tool_calls.is_empty() {
            return Ok(produced);
        }

        for call in tool_calls {
            let result = tools::execute_tool(ctx, &call.function.name, &call.function.arguments);
            let content = match result {
                Ok(value) => value.to_string(),
                Err(e) => serde_json::json!({ "error": e }).to_string(),
            };
            let tool_message = ChatMessage::tool(call.function.name, content);
            history.push(tool_message.clone());
            produced.push(tool_message);
        }
    }

    Err("Stopped after too many consecutive tool calls - please rephrase your request.".to_string())
}
