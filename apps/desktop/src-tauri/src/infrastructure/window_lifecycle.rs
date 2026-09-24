use crate::adapters::inbound::{tray_menu, window_menu};

pub fn setup_window_management(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();
    window_menu::setup_window_menu(&handle)?;
    tray_menu::setup_tray(&handle)?;

    // Tray and window menu events share one stream, so the tray claims its own
    // ids first and the window menu handles the rest.
    handle.on_menu_event(|app, event| {
        let id = event.id().as_ref();
        if tray_menu::handle_menu_event(app, id) {
            return;
        }

        window_menu::handle_window_menu_event(app, id);
    });

    Ok(())
}
