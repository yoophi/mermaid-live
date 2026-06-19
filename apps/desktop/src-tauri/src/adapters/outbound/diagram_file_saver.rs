use std::path::PathBuf;

use tauri::{Runtime, Window};
use tauri_plugin_dialog::DialogExt;

const DEFAULT_EXTENSION: &str = "mmd";

pub fn save_diagram_source<R: Runtime>(
    window: &Window<R>,
    source: &str,
    default_file_name: &str,
) -> Result<Option<PathBuf>, String> {
    let Some(file_path) = window
        .dialog()
        .file()
        .set_parent(window)
        .set_title("Save Mermaid chart")
        .set_file_name(default_file_name)
        .add_filter("Mermaid chart", &["mmd", "mermaid"])
        .add_filter("Markdown", &["md"])
        .blocking_save_file()
    else {
        return Ok(None);
    };

    let mut path = file_path.into_path().map_err(|error| error.to_string())?;
    if path.extension().is_none() {
        path.set_extension(DEFAULT_EXTENSION);
    }

    std::fs::write(&path, source).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(Some(path))
}
