use std::sync::Arc;
use std::thread;

use neuraldisk_core::common::consts::DEFAULT_THREAD_SIZE;
use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::agent::ollama_client::OllamaClient;
use crate::agent::tools::AgentContext;
use crate::agent::{AgentSession, handle_user_message};
use crate::{AgentChatMessageModel, Callabler, GuiState, MainWindow, Settings};

pub(crate) fn connect_agent_chat(app: &MainWindow) {
    let session = Arc::new(AgentSession::new());

    let a = app.as_weak();
    app.global::<Callabler>().on_agent_send_message(move |text| {
        let app = a.upgrade().expect("agent_send_message: MainWindow dropped while callback is still live");
        let text = text.to_string();
        if text.trim().is_empty() {
            return;
        }

        if app.get_scanning() || app.get_processing() {
            push_chat_message(&app, "assistant", "A scan or file operation is already running - please wait for it to finish before asking me to do something.");
            return;
        }

        // Read fresh on every message (not just once at startup) so a Settings change to the
        // Ollama server URL or model takes effect on the next message, no app restart needed.
        let settings = app.global::<Settings>();
        let client = OllamaClient::new(settings.get_agent_ollama_base_url().to_string(), settings.get_agent_ollama_model().to_string());

        push_chat_message(&app, "user", &text);
        app.global::<GuiState>().set_agent_chat_busy(true);
        // Mirrors the manual scan/file-action buttons: blocks the user from starting a second
        // scan or file action while the agent might be mid-tool-call driving the same state.
        app.set_processing(true);

        let ctx = AgentContext { app_weak: app.as_weak() };
        let session = Arc::clone(&session);
        let weak = app.as_weak();

        thread::Builder::new()
            .stack_size(DEFAULT_THREAD_SIZE)
            .spawn(move || {
                let result = handle_user_message(&session, &client, &ctx, text);
                let _ = weak.upgrade_in_event_loop(move |app| {
                    app.global::<GuiState>().set_agent_chat_busy(false);
                    app.set_processing(false);
                    match result {
                        Ok(messages) => {
                            for message in messages {
                                let (role, text) = if message.role == "tool" {
                                    ("tool", format!("[{}] {}", message.tool_name.as_deref().unwrap_or("tool"), message.content))
                                } else {
                                    (message.role.as_str(), message.content.clone())
                                };
                                if !text.trim().is_empty() {
                                    push_chat_message(&app, role, &text);
                                }
                            }
                        }
                        Err(e) => push_chat_message(&app, "assistant", &format!("Error talking to Ollama: {e}")),
                    }
                });
            })
            .expect("connect_agent_chat: failed to spawn background thread for an agent turn");
    });
}

fn push_chat_message(app: &MainWindow, role: &str, text: &str) {
    let gui_state = app.global::<GuiState>();
    let mut messages: Vec<AgentChatMessageModel> = gui_state.get_agent_chat_messages().iter().collect();
    messages.push(AgentChatMessageModel { role: role.into(), text: text.into() });
    gui_state.set_agent_chat_messages(ModelRc::new(VecModel::from(messages)));
}
