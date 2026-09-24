use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

use crate::adapters::outbound::{diagram_file_saver, native_window_manager};
use crate::domain::diagram::DiagramValidation;
use crate::infrastructure::app_state::AppState;
use crate::infrastructure::render_service::RenderReply;

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

/// Receives the image the hidden render window produced, or the reason it could
/// not produce one.
#[tauri::command]
pub fn deliver_chart_png(
    state: tauri::State<'_, AppState>,
    request_id: String,
    png_base64: Option<String>,
    error: Option<String>,
) -> Result<(), String> {
    let reply = match (png_base64, error) {
        (Some(encoded), _) => match BASE64.decode(encoded.as_bytes()) {
            Ok(png) => RenderReply::Png(png),
            Err(decode_error) => {
                RenderReply::Failed(format!("the rendered image was unreadable: {decode_error}"))
            }
        },
        (None, Some(message)) => RenderReply::Failed(message),
        (None, None) => RenderReply::Failed("the renderer reported no result".to_string()),
    };

    if state.pending_renders.complete(&request_id, reply) {
        Ok(())
    } else {
        Err(format!("no render is waiting for {request_id}"))
    }
}
