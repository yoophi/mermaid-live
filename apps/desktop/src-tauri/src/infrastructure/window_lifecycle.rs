use crate::adapters::inbound::window_menu;

pub fn setup_window_management(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();
    window_menu::setup_window_menu(&handle)?;
    handle.on_menu_event(|app, event| {
        window_menu::handle_window_menu_event(app, event.id().as_ref());
    });
    Ok(())
}
