use serde::{Deserialize, Serialize};
use std::fmt;

pub const DEFAULT_EXTENSION: &str = "mmd";
pub const SUPPORTED_EXTENSIONS: [&str; 2] = ["mmd", "mermaid"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRevision {
    pub content_hash: String,
    pub byte_length: u64,
    pub modified_at: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentFileBinding {
    pub path: String,
    pub file_name: String,
    pub extension: String,
    pub revision: FileRevision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagramFileSnapshot {
    pub source: String,
    pub binding: DocumentFileBinding,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDiagramRequest {
    pub source: String,
    pub target_path: Option<String>,
    pub expected_revision: Option<FileRevision>,
    pub force: bool,
    pub suggested_file_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum OpenDiagramOutcome {
    Opened { snapshot: DiagramFileSnapshot },
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum SaveDiagramOutcome {
    Saved {
        snapshot: DiagramFileSnapshot,
    },
    Conflict {
        #[serde(rename = "diskSnapshot")]
        disk_snapshot: DiagramFileSnapshot,
    },
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartFileError {
    pub category: ChartFileErrorCategory,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ChartFileErrorCategory {
    UnsupportedExtension,
    Missing,
    PermissionDenied,
    InvalidUtf8,
    ReadFailed,
    WriteFailed,
    ReplaceFailed,
}

impl ChartFileError {
    pub fn new(category: ChartFileErrorCategory, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
        }
    }
}

impl fmt::Display for ChartFileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ChartFileError {}

pub fn normalize_open_path(path: &str) -> Result<String, ChartFileError> {
    validate_extension(path)?;
    Ok(path.to_owned())
}

pub fn normalize_save_path(path: &str) -> Result<String, ChartFileError> {
    let path = std::path::Path::new(path);
    if path.extension().is_none() {
        return Ok(path
            .with_extension(DEFAULT_EXTENSION)
            .to_string_lossy()
            .into_owned());
    }

    validate_extension(path.to_string_lossy().as_ref())?;
    Ok(path.to_string_lossy().into_owned())
}

pub fn normalized_extension(path: &str) -> Result<String, ChartFileError> {
    let extension = std::path::Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| unsupported_extension(path))?;

    if SUPPORTED_EXTENSIONS.contains(&extension.as_str()) {
        Ok(extension)
    } else {
        Err(unsupported_extension(path))
    }
}

fn validate_extension(path: &str) -> Result<(), ChartFileError> {
    normalized_extension(path).map(|_| ())
}

fn unsupported_extension(path: &str) -> ChartFileError {
    ChartFileError::new(
        ChartFileErrorCategory::UnsupportedExtension,
        format!("지원되는 Mermaid 파일 형식(.mmd, .mermaid)을 사용하세요: {path}"),
    )
}

#[cfg(test)]
mod tests {
    use super::{normalize_open_path, normalize_save_path, normalized_extension};

    #[test]
    fn accepts_supported_extensions_case_insensitively() {
        assert_eq!(normalized_extension("chart.MMD").unwrap(), "mmd");
        assert_eq!(normalized_extension("chart.Mermaid").unwrap(), "mermaid");
    }

    #[test]
    fn adds_default_extension_when_saving_without_one() {
        assert!(normalize_save_path("chart").unwrap().ends_with("chart.mmd"));
    }

    #[test]
    fn rejects_unsupported_or_missing_open_extensions() {
        assert!(normalize_open_path("chart.md").is_err());
        assert!(normalize_open_path("chart").is_err());
    }
}
