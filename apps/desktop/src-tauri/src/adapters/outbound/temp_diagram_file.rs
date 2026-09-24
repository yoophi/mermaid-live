use std::path::PathBuf;

/// Writes the chart where a webview can load it, named after its fingerprint so
/// the same chart always maps to the same file.
pub fn write_temp_diagram_file(source: &str, fingerprint: &str) -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("mermaid-live");
    std::fs::create_dir_all(&dir).map_err(|error| format!("temp dir create failed: {error}"))?;

    let path = dir.join(format!("clipboard-{fingerprint}.mmd"));
    std::fs::write(&path, source).map_err(|error| format!("temp diagram write failed: {error}"))?;
    Ok(path)
}
