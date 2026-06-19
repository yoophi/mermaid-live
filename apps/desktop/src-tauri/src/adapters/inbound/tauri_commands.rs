use crate::adapters::outbound::native_window_manager;
use crate::domain::diagram::DiagramValidation;
use crate::infrastructure::app_state::AppState;

#[tauri::command]
pub fn validate_diagram_source(
    state: tauri::State<'_, AppState>,
    source: String,
) -> Result<DiagramValidation, String> {
    state
        .validate_diagram_source
        .execute(source)
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn open_editor_window(app: tauri::AppHandle) -> Result<(), String> {
    native_window_manager::open_editor_window(&app).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn open_editor_tab(app: tauri::AppHandle) {
    native_window_manager::open_editor_tab(&app);
}

#[tauri::command]
pub fn merge_all_windows(app: tauri::AppHandle) {
    native_window_manager::merge_all_windows(&app);
}

#[tauri::command]
pub fn toggle_tab_bar(app: tauri::AppHandle) {
    native_window_manager::toggle_tab_bar(&app);
}

#[tauri::command]
pub fn read_diagram_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|err| format!("{path}: {err}"))
}
