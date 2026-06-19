use std::path::PathBuf;

pub fn diagram_source_md5(source: &str) -> String {
    format!("{:x}", md5::compute(source.trim()))
}

pub fn write_temp_diagram_file(source: &str, md5: &str) -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("mermaid-live");
    std::fs::create_dir_all(&dir).map_err(|error| format!("temp dir create failed: {error}"))?;

    let path = dir.join(format!("clipboard-{md5}.mmd"));
    std::fs::write(&path, source).map_err(|error| format!("temp diagram write failed: {error}"))?;
    Ok(path)
}
