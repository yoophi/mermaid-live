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
