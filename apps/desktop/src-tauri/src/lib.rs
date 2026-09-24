mod adapters;
mod application;
mod domain;
mod infrastructure;

use adapters::inbound::tauri_commands;

pub fn run() {
    let state = infrastructure::app_state::build_app_state();

    let app = tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            tauri_commands::validate_diagram_source,
            tauri_commands::open_editor_window,
            tauri_commands::open_editor_tab,
            tauri_commands::merge_all_windows,
            tauri_commands::read_diagram_file,
            tauri_commands::save_diagram_file,
            tauri_commands::deliver_chart_png
        ])
        .on_window_event(infrastructure::clipboard_import::handle_window_event)
        .setup(|app| {
            infrastructure::window_lifecycle::setup_window_management(app)?;
            infrastructure::clipboard_watcher::start(app.handle());
            adapters::inbound::render_socket::start(app.handle());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build Mermaid Live");

    app.run(|app, event| match event {
        // Closing the last window must not stop the clipboard watcher; the tray
        // icon stays behind to bring a detected chart back. Cmd+Q still quits,
        // because macOS routes it straight to the app rather than through here.
        tauri::RunEvent::ExitRequested { code, api, .. } if code.is_none() => {
            api.prevent_exit();
        }
        #[cfg(target_os = "macos")]
        tauri::RunEvent::Reopen {
            has_visible_windows,
            ..
        } if !has_visible_windows => infrastructure::clipboard_watcher::reopen(app),
        _ => {}
    });
}
