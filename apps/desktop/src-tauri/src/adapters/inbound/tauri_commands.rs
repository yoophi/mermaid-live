use serde::Serialize;

use crate::adapters::outbound::{chart_file_dialog, native_window_manager};
use crate::domain::chart_document::{
    ChartFileError, OpenDiagramOutcome, SaveDiagramOutcome, SaveDiagramRequest,
};
use crate::domain::diagram::DiagramValidation;
use crate::infrastructure::{app_state::AppState, document_window_lifecycle::CloseAuthorizations};

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
pub async fn open_diagram_file(
    window: tauri::Window,
    state: tauri::State<'_, AppState>,
) -> Result<OpenDiagramOutcome, ChartFileError> {
    let Some(path) = chart_file_dialog::pick_diagram_file(&window).map_err(|message| {
        ChartFileError::new(
            crate::domain::chart_document::ChartFileErrorCategory::ReadFailed,
            message,
        )
    })?
    else {
        return Ok(OpenDiagramOutcome::Cancelled);
    };

    state
        .open_chart_file
        .execute(path.to_string_lossy().into_owned())
        .map(|snapshot| OpenDiagramOutcome::Opened { snapshot })
}

#[tauri::command]
pub async fn save_diagram_file(
    window: tauri::Window,
    state: tauri::State<'_, AppState>,
    request: SaveDiagramRequest,
) -> Result<SaveDiagramOutcome, ChartFileError> {
    let path = match request.target_path {
        Some(path) => path,
        None => {
            let Some(path) =
                chart_file_dialog::pick_save_path(&window, &request.suggested_file_name).map_err(
                    |message| {
                        ChartFileError::new(
                            crate::domain::chart_document::ChartFileErrorCategory::WriteFailed,
                            message,
                        )
                    },
                )?
            else {
                return Ok(SaveDiagramOutcome::Cancelled);
            };
            path.to_string_lossy().into_owned()
        }
    };

    state.save_chart_file.execute(
        path,
        request.source,
        request.expected_revision,
        request.force,
    )
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UnsavedChangesDecision {
    Save,
    Discard,
    Cancel,
}

#[tauri::command]
pub async fn prompt_unsaved_changes(
    window: tauri::Window,
    file_name: String,
) -> UnsavedChangesDecision {
    match chart_file_dialog::prompt_unsaved_changes(&window, &file_name) {
        chart_file_dialog::UnsavedChangesDecision::Save => UnsavedChangesDecision::Save,
        chart_file_dialog::UnsavedChangesDecision::Discard => UnsavedChangesDecision::Discard,
        chart_file_dialog::UnsavedChangesDecision::Cancel => UnsavedChangesDecision::Cancel,
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExternalConflictDecision {
    Reload,
    Overwrite,
    Cancel,
}

#[tauri::command]
pub async fn prompt_external_conflict(
    window: tauri::Window,
    file_name: String,
) -> ExternalConflictDecision {
    match chart_file_dialog::prompt_external_conflict(&window, &file_name) {
        chart_file_dialog::ExternalConflictDecision::Reload => ExternalConflictDecision::Reload,
        chart_file_dialog::ExternalConflictDecision::Overwrite => {
            ExternalConflictDecision::Overwrite
        }
        chart_file_dialog::ExternalConflictDecision::Cancel => ExternalConflictDecision::Cancel,
    }
}

#[tauri::command]
pub fn authorize_window_close(
    window: tauri::Window,
    authorizations: tauri::State<'_, CloseAuthorizations>,
) -> Result<(), String> {
    authorizations.authorize(window.label());
    window.close().map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn show_document_error(window: tauri::Window, title: String, message: String) {
    chart_file_dialog::show_error(&window, &title, &message);
}
