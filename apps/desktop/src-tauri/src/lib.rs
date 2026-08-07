mod adapters;
mod application;
mod domain;
mod infrastructure;

use adapters::inbound::tauri_commands;

pub fn run() {
    let state = infrastructure::app_state::build_app_state();

    tauri::Builder::default()
        .manage(state)
        .manage(infrastructure::document_window_lifecycle::CloseAuthorizations::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            tauri_commands::validate_diagram_source,
            tauri_commands::open_editor_window,
            tauri_commands::open_editor_tab,
            tauri_commands::merge_all_windows,
            tauri_commands::read_diagram_file,
            tauri_commands::open_diagram_file,
            tauri_commands::save_diagram_file,
            tauri_commands::prompt_unsaved_changes,
            tauri_commands::prompt_external_conflict,
            tauri_commands::authorize_window_close,
            tauri_commands::show_document_error
        ])
        .on_window_event(|window, event| {
            infrastructure::document_window_lifecycle::handle_window_event(window, event);
            infrastructure::clipboard_import::handle_window_event(window, event);
        })
        .setup(infrastructure::window_lifecycle::setup_window_management)
        .run(tauri::generate_context!())
        .expect("failed to run Mermaid Live");
}
