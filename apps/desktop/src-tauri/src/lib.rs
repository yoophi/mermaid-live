mod adapters;
mod application;
mod domain;
mod infrastructure;

use adapters::inbound::tauri_commands;

pub fn run() {
    let state = infrastructure::app_state::build_app_state();

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            tauri_commands::validate_diagram_source,
            tauri_commands::open_editor_window,
            tauri_commands::open_editor_tab,
            tauri_commands::merge_all_windows,
            tauri_commands::toggle_tab_bar,
            tauri_commands::read_diagram_file
        ])
        .on_window_event(infrastructure::clipboard_import::handle_window_event)
        .setup(infrastructure::window_lifecycle::setup_window_management)
        .run(tauri::generate_context!())
        .expect("failed to run Mermaid Live");
}
