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
            tauri_commands::validate_diagram_source
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Mermaid Live");
}
