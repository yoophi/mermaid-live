use crate::adapters::outbound::{diagram_file_saver, native_window_manager};
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
pub fn read_diagram_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|err| format!("{path}: {err}"))
}

#[tauri::command]
pub async fn save_diagram_file(
    window: tauri::Window,
    source: String,
    default_file_name: String,
) -> Result<Option<String>, String> {
    diagram_file_saver::save_diagram_source(&window, &source, &default_file_name)
        .map(|path| path.map(|path| path.display().to_string()))
}
