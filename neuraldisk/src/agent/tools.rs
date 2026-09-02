use std::sync::{OnceLock, mpsc};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use slint::{ComponentHandle, Weak};

use crate::agent::ollama_client::{ToolSpec, ToolSpecFunction};
use crate::model_operations::get_checked_info_from_app;
use crate::{ActiveTab, Callabler, GuiState, MainWindow, PopupRequest, SelectMode, Translations, flk};

/// Drives the app the same way a manual button click would: every tool call goes through this
/// weak handle and the app's existing Slint callbacks, never touching scan internals directly.
pub(crate) struct AgentContext {
    pub(crate) app_weak: Weak<MainWindow>,
}

const TOOL_NAMES: &[(&str, ActiveTab)] = &[
    ("duplicate_files", ActiveTab::DuplicateFiles),
    ("empty_folders", ActiveTab::EmptyFolders),
    ("big_files", ActiveTab::BigFiles),
    ("empty_files", ActiveTab::EmptyFiles),
    ("temporary_files", ActiveTab::TemporaryFiles),
    ("similar_images", ActiveTab::SimilarImages),
    ("similar_videos", ActiveTab::SimilarVideos),
    ("similar_music", ActiveTab::SimilarMusic),
    ("invalid_symlinks", ActiveTab::InvalidSymlinks),
    ("broken_files", ActiveTab::BrokenFiles),
    ("bad_extensions", ActiveTab::BadExtensions),
    ("bad_names", ActiveTab::BadNames),
    ("exif_remover", ActiveTab::ExifRemover),
    ("video_optimizer", ActiveTab::VideoOptimizer),
];

// Only these 4 tools show grouped/header rows; every other tool's rows are flat. Selecting by
// "which one to keep in each group" only makes sense - and only avoids panicking - for these.
const GROUPED_SELECT_MODES: &[&str] = &["keep_newest", "keep_oldest", "keep_biggest", "keep_smallest", "keep_shortest_path", "keep_longest_path"];

const ACTIONS: &[(&str, PopupRequest)] = &[
    ("delete", PopupRequest::Delete),
    ("trash", PopupRequest::Trash),
    ("clean_exif", PopupRequest::CleanExif),
    ("hardlink", PopupRequest::Hardlink),
    ("symlink", PopupRequest::Symlink),
    ("rename_bad_extension", PopupRequest::RenameBadExtension),
    ("rename_bad_name", PopupRequest::RenameBadFileName),
    ("optimize_video", PopupRequest::OptimizeVideo),
];

const MAX_SCAN_WAIT: Duration = Duration::from_secs(30 * 60);
const SCAN_POLL_INTERVAL: Duration = Duration::from_millis(200);

fn active_tab_from_str(name: &str) -> Result<ActiveTab, String> {
    TOOL_NAMES
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, tab)| *tab)
        .ok_or_else(|| format!("Unknown tool \"{name}\" - must be one of: {}", TOOL_NAMES.iter().map(|(k, _)| *k).collect::<Vec<_>>().join(", ")))
}

fn select_mode_from_str(name: &str, active_tab: ActiveTab) -> Result<SelectMode, String> {
    if GROUPED_SELECT_MODES.contains(&name) && !active_tab.get_is_header_mode() {
        return Err(format!(
            "\"{name}\" only applies to tools with duplicate groups (duplicate_files, similar_images, similar_videos, similar_music); use \"all\" instead for this tool."
        ));
    }
    Ok(match name {
        "all" => SelectMode::SelectAll,
        "none" => SelectMode::UnselectAll,
        "invert" => SelectMode::InvertSelection,
        "keep_newest" => SelectMode::SelectAllExceptNewest,
        "keep_oldest" => SelectMode::SelectAllExceptOldest,
        "keep_biggest" => SelectMode::SelectAllExceptBiggestSize,
        "keep_smallest" => SelectMode::SelectAllExceptSmallestSize,
        "keep_shortest_path" => SelectMode::SelectAllExceptShortestPath,
        "keep_longest_path" => SelectMode::SelectAllExceptLongestPath,
        other => return Err(format!("Unknown select_mode \"{other}\"")),
    })
}

fn action_from_str(name: &str) -> Result<PopupRequest, String> {
    ACTIONS
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, request)| *request)
        .ok_or_else(|| format!("Unknown action \"{name}\" - must be one of: {}", ACTIONS.iter().map(|(k, _)| *k).collect::<Vec<_>>().join(", ")))
}

/// Several confirm handlers downstream (`connect_show_confirmation.rs`, `file_actions/*`) assume
/// the active tab matches the action and `panic!`/`.expect()` otherwise (e.g. optimize_video on a
/// tab with no video-optimizer scan, or clean_exif outside the ExifRemover tab). This must be
/// checked here - the LLM is free-form input and nothing upstream stops a mismatched pair.
fn validate_action_for_tab(action: &str, active_tab: ActiveTab, tool_name: &str) -> Result<(), String> {
    let valid = match action {
        "delete" | "trash" => true,
        "hardlink" | "symlink" => active_tab.get_is_header_mode(),
        "clean_exif" => active_tab == ActiveTab::ExifRemover,
        "optimize_video" => active_tab == ActiveTab::VideoOptimizer,
        "rename_bad_extension" => active_tab == ActiveTab::BadExtensions,
        "rename_bad_name" => active_tab == ActiveTab::BadNames,
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(format!(
            "Action \"{action}\" doesn't apply to tool \"{tool_name}\" - clean_exif only works with exif_remover, optimize_video only with video_optimizer, rename_bad_extension only with bad_extensions, rename_bad_name only with bad_names, and hardlink/symlink only with duplicate_files/similar_images/similar_videos/similar_music."
        ))
    }
}

fn string_enum(values: &[&str]) -> Value {
    Value::Array(values.iter().map(|v| Value::String((*v).to_string())).collect())
}

fn tool_name_enum_json() -> Value {
    string_enum(&TOOL_NAMES.iter().map(|(key, _)| *key).collect::<Vec<_>>())
}

const SELECT_MODE_DESCRIPTION: &str = "Which single entry in each group to keep; every other entry in that group is selected for the action. \"all\"/\"none\"/\"invert\" select every row directly instead of applying a per-group rule. Only duplicate_files/similar_images/similar_videos/similar_music have groups - use \"all\" for every other tool.";

static TOOL_SPECS: OnceLock<Vec<ToolSpec>> = OnceLock::new();

/// Tool specs are static for the process lifetime - cached so a multi-tool-call chat turn
/// doesn't reallocate the same JSON schemas on every round trip.
pub(crate) fn tool_specs() -> &'static [ToolSpec] {
    TOOL_SPECS.get_or_init(build_tool_specs).as_slice()
}

fn build_tool_specs() -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            kind: "function",
            function: ToolSpecFunction {
                name: "run_scan",
                description: "Run one of NeuralDisk's scanners. Only scans the directories the user has already added to Included Directories in the app - it never accepts or invents new paths. Blocks until the scan finishes and returns the same human-readable summary the app itself shows.",
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "tool": {
                            "type": "string",
                            "enum": tool_name_enum_json(),
                            "description": "Which scanner to run.",
                        }
                    },
                    "required": ["tool"]
                }),
            },
        },
        ToolSpec {
            kind: "function",
            function: ToolSpecFunction {
                name: "get_last_scan_summary",
                description: "Get the summary text of whatever the app most recently reported (e.g. the last scan or action), without running anything new.",
                parameters: json!({ "type": "object", "properties": {}, "required": [] }),
            },
        },
        ToolSpec {
            kind: "function",
            function: ToolSpecFunction {
                name: "select_and_propose_action",
                description: "Switch to the given tool's results, select entries according to a rule, then open the app's existing confirmation dialog for the requested action (delete, trash, clean_exif, hardlink, symlink, rename_bad_extension, rename_bad_name, optimize_video). This never performs the action by itself - the user must still review and confirm the dialog. The action MUST match the tool: clean_exif only works with exif_remover, optimize_video only with video_optimizer, rename_bad_extension only with bad_extensions, rename_bad_name only with bad_names, and hardlink/symlink only with duplicate_files/similar_images/similar_videos/similar_music - any other pairing is rejected. Only run this after `run_scan` has already populated results for that tool. For moving files to a folder, use `propose_move` instead.",
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "tool": {
                            "type": "string",
                            "enum": tool_name_enum_json(),
                            "description": "Which tool's results to act on (must already have been scanned).",
                        },
                        "select_mode": {
                            "type": "string",
                            "enum": ["all", "none", "invert", "keep_newest", "keep_oldest", "keep_biggest", "keep_smallest", "keep_shortest_path", "keep_longest_path"],
                            "description": SELECT_MODE_DESCRIPTION,
                        },
                        "action": {
                            "type": "string",
                            "enum": string_enum(&ACTIONS.iter().map(|(k, _)| *k).collect::<Vec<_>>()),
                            "description": "Which confirmation dialog to open - must match the tool, see this function's description.",
                        }
                    },
                    "required": ["tool", "select_mode", "action"]
                }),
            },
        },
        ToolSpec {
            kind: "function",
            function: ToolSpecFunction {
                name: "propose_move",
                description: "Switch to the given tool's results, select entries according to a rule, then open the app's existing move-confirmation dialog with the given destination folder. This never moves anything by itself - the user must still review and confirm the dialog. Only run this after `run_scan` has already populated results for that tool.",
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "tool": {
                            "type": "string",
                            "enum": tool_name_enum_json(),
                            "description": "Which tool's results to act on (must already have been scanned).",
                        },
                        "select_mode": {
                            "type": "string",
                            "enum": ["all", "none", "invert", "keep_newest", "keep_oldest", "keep_biggest", "keep_smallest", "keep_shortest_path", "keep_longest_path"],
                            "description": SELECT_MODE_DESCRIPTION,
                        },
                        "destination_folder": {
                            "type": "string",
                            "description": "Absolute path of the folder to move the selected files into.",
                        }
                    },
                    "required": ["tool", "select_mode", "destination_folder"]
                }),
            },
        },
    ]
}

pub(crate) fn execute_tool(ctx: &AgentContext, name: &str, arguments: &Value) -> Result<Value, String> {
    match name {
        "run_scan" => run_scan_tool(ctx, arguments),
        "get_last_scan_summary" => read_text_summary(&ctx.app_weak).map(|summary| json!({ "summary": summary })),
        "select_and_propose_action" => select_and_propose_action_tool(ctx, arguments),
        "propose_move" => propose_move_tool(ctx, arguments),
        other => Err(format!("Unknown tool \"{other}\"")),
    }
}

fn required_str_arg<'a>(arguments: &'a Value, key: &str) -> Result<&'a str, String> {
    arguments.get(key).and_then(Value::as_str).ok_or_else(|| format!("Missing required \"{key}\" argument"))
}

/// Every tool call needs to read/mutate Slint state, which only exists on the UI thread. This
/// runs `f` there via a queued closure and blocks the calling (agent worker) thread for the
/// result, collapsing the mpsc-channel + upgrade_in_event_loop boilerplate to one call site.
fn run_on_ui<T: Send + 'static>(app_weak: &Weak<MainWindow>, f: impl FnOnce(&MainWindow) -> T + Send + 'static) -> Result<T, String> {
    let (tx, rx) = mpsc::channel();
    app_weak
        .upgrade_in_event_loop(move |app| {
            let _ = tx.send(f(&app));
        })
        .map_err(|e| format!("The app window is gone: {e}"))?;
    rx.recv().map_err(|_| "The app closed before responding".to_string())
}

fn run_scan_tool(ctx: &AgentContext, arguments: &Value) -> Result<Value, String> {
    let tool_name = required_str_arg(arguments, "tool")?;
    let active_tab = active_tab_from_str(tool_name)?;

    // `scan_starting`'s handler bails out synchronously (no thread spawned) when e.g. no included
    // directories are configured, flipping `scanning` back to false before this closure returns.
    // A real scan spawns a worker thread and returns to the event loop with `scanning` still true.
    // Reading it back here, in the same closure, reliably tells the two cases apart.
    let scan_actually_started = run_on_ui(&ctx.app_weak, move |app| {
        app.global::<GuiState>().set_active_tab(active_tab);
        app.set_scanning(true);
        app.invoke_scan_starting(active_tab);
        app.get_scanning()
    })?;

    if !scan_actually_started {
        let reason = read_text_summary(&ctx.app_weak)?;
        return Ok(json!({ "status": "scan_not_started", "tool": tool_name, "reason": reason }));
    }

    wait_for_scan_to_finish(&ctx.app_weak)?;

    let summary = read_text_summary(&ctx.app_weak)?;
    Ok(json!({
        "status": "done",
        "tool": tool_name,
        "summary": summary,
        "note": "Read the summary text carefully - it describes what actually happened and may report an error rather than results.",
    }))
}

fn wait_for_scan_to_finish(app_weak: &Weak<MainWindow>) -> Result<(), String> {
    let start = Instant::now();
    loop {
        let still_scanning = run_on_ui(app_weak, |app| app.get_scanning())?;
        if !still_scanning {
            return Ok(());
        }
        if start.elapsed() > MAX_SCAN_WAIT {
            return Err("The scan has been running for over 30 minutes without finishing - it may still be running in the background; use the app's Stop button if you want to cancel it.".to_string());
        }
        std::thread::sleep(SCAN_POLL_INTERVAL);
    }
}

fn read_text_summary(app_weak: &Weak<MainWindow>) -> Result<String, String> {
    run_on_ui(app_weak, |app| app.get_text_summary_text().to_string())
}

fn select_and_propose_action_tool(ctx: &AgentContext, arguments: &Value) -> Result<Value, String> {
    let tool_name = required_str_arg(arguments, "tool")?;
    let active_tab = active_tab_from_str(tool_name)?;
    let select_mode = select_mode_from_str(required_str_arg(arguments, "select_mode")?, active_tab)?;
    let action_name = required_str_arg(arguments, "action")?;
    let popup_request = action_from_str(action_name)?;
    validate_action_for_tab(action_name, active_tab, tool_name)?;

    run_on_ui(&ctx.app_weak, move |app| {
        app.global::<GuiState>().set_active_tab(active_tab);
        app.global::<Callabler>().invoke_select_items(select_mode);
        app.invoke_request_setup_action_popup(popup_request);
    })?;

    Ok(json!({
        "status": "confirmation_popup_shown",
        "tool": tool_name,
        "action": action_name,
        "note": "The confirmation dialog is now open in the app. Nothing has happened yet - the user must review and confirm it themselves.",
    }))
}

fn propose_move_tool(ctx: &AgentContext, arguments: &Value) -> Result<Value, String> {
    let tool_name = required_str_arg(arguments, "tool")?;
    let active_tab = active_tab_from_str(tool_name)?;
    let select_mode = select_mode_from_str(required_str_arg(arguments, "select_mode")?, active_tab)?;
    let destination_folder = required_str_arg(arguments, "destination_folder")?.to_string();

    let destination_for_popup = destination_folder.clone();
    run_on_ui(&ctx.app_weak, move |app| {
        app.global::<GuiState>().set_active_tab(active_tab);
        app.global::<Callabler>().invoke_select_items(select_mode);

        let checked_items_number = get_checked_info_from_app(app).checked_items_number;
        let mut confirmation_text = flk!("rust_move_confirmation");
        confirmation_text.push_str(&format!("\n{}", flk!("rust_move_confirmation_number_simple", items = checked_items_number)));
        app.global::<Translations>().set_move_confirmation_text(confirmation_text.into());

        app.invoke_show_action_popup(PopupRequest::Move, destination_for_popup.into());
    })?;

    Ok(json!({
        "status": "confirmation_popup_shown",
        "tool": tool_name,
        "action": "move",
        "destination_folder": destination_folder,
        "note": "The move-confirmation dialog is now open in the app. Nothing has been moved yet - the user must review and confirm it themselves.",
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_tab_from_str_accepts_every_declared_tool_name() {
        for (name, _) in TOOL_NAMES {
            active_tab_from_str(name).unwrap_or_else(|e| panic!("expected {name} to resolve: {e}"));
        }
    }

    #[test]
    fn active_tab_from_str_rejects_unknown_name() {
        active_tab_from_str("not_a_real_tool").unwrap_err();
    }

    #[test]
    fn select_mode_from_str_rejects_grouped_modes_for_non_grouped_tools() {
        select_mode_from_str("keep_newest", ActiveTab::BigFiles).unwrap_err();
        select_mode_from_str("keep_biggest", ActiveTab::ExifRemover).unwrap_err();
    }

    #[test]
    fn select_mode_from_str_allows_grouped_modes_for_grouped_tools() {
        select_mode_from_str("keep_newest", ActiveTab::DuplicateFiles).unwrap();
        select_mode_from_str("keep_biggest", ActiveTab::SimilarImages).unwrap();
    }

    #[test]
    fn select_mode_from_str_always_allows_all_none_invert() {
        for mode in ["all", "none", "invert"] {
            select_mode_from_str(mode, ActiveTab::BigFiles).unwrap();
            select_mode_from_str(mode, ActiveTab::DuplicateFiles).unwrap();
        }
    }

    #[test]
    fn select_mode_from_str_rejects_unknown_mode() {
        select_mode_from_str("not_a_real_mode", ActiveTab::DuplicateFiles).unwrap_err();
    }

    #[test]
    fn action_from_str_accepts_every_declared_action() {
        for (name, _) in ACTIONS {
            action_from_str(name).unwrap_or_else(|e| panic!("expected {name} to resolve: {e}"));
        }
    }

    #[test]
    fn validate_action_for_tab_allows_delete_and_trash_everywhere() {
        for (name, _) in TOOL_NAMES {
            let tab = active_tab_from_str(name).expect("declared tool name must resolve");
            validate_action_for_tab("delete", tab, name).unwrap();
            validate_action_for_tab("trash", tab, name).unwrap();
        }
    }

    #[test]
    fn validate_action_for_tab_rejects_optimize_video_outside_video_optimizer() {
        validate_action_for_tab("optimize_video", ActiveTab::VideoOptimizer, "video_optimizer").unwrap();
        validate_action_for_tab("optimize_video", ActiveTab::BigFiles, "big_files").unwrap_err();
        validate_action_for_tab("optimize_video", ActiveTab::DuplicateFiles, "duplicate_files").unwrap_err();
    }

    #[test]
    fn validate_action_for_tab_rejects_clean_exif_outside_exif_remover() {
        validate_action_for_tab("clean_exif", ActiveTab::ExifRemover, "exif_remover").unwrap();
        validate_action_for_tab("clean_exif", ActiveTab::BigFiles, "big_files").unwrap_err();
    }

    #[test]
    fn validate_action_for_tab_rejects_rename_actions_on_wrong_tab() {
        validate_action_for_tab("rename_bad_extension", ActiveTab::BadExtensions, "bad_extensions").unwrap();
        validate_action_for_tab("rename_bad_extension", ActiveTab::BadNames, "bad_names").unwrap_err();
        validate_action_for_tab("rename_bad_name", ActiveTab::BadNames, "bad_names").unwrap();
        validate_action_for_tab("rename_bad_name", ActiveTab::BadExtensions, "bad_extensions").unwrap_err();
    }

    #[test]
    fn validate_action_for_tab_hardlink_symlink_only_for_grouped_tools() {
        validate_action_for_tab("hardlink", ActiveTab::DuplicateFiles, "duplicate_files").unwrap();
        validate_action_for_tab("symlink", ActiveTab::SimilarMusic, "similar_music").unwrap();
        validate_action_for_tab("hardlink", ActiveTab::BigFiles, "big_files").unwrap_err();
        validate_action_for_tab("symlink", ActiveTab::ExifRemover, "exif_remover").unwrap_err();
    }

    #[test]
    fn tool_specs_are_stable_across_calls() {
        let first = tool_specs();
        let second = tool_specs();
        assert_eq!(first.len(), second.len());
        assert!(std::ptr::eq(first.as_ptr(), second.as_ptr()), "tool_specs() should return the same cached allocation");
    }
}
