use std::fs::{self, File};
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::time::UNIX_EPOCH;

use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

use crate::application::ports::ChartFileRepository;
use crate::domain::chart_document::{
    normalized_extension, ChartFileError, ChartFileErrorCategory, DiagramFileSnapshot,
    DocumentFileBinding, FileRevision,
};

#[derive(Clone, Copy, Default)]
pub struct LocalChartFileRepository;

impl ChartFileRepository for LocalChartFileRepository {
    fn read(&self, path: &str) -> Result<DiagramFileSnapshot, ChartFileError> {
        let bytes = fs::read(path).map_err(|error| map_read_error(path, error))?;
        let source = String::from_utf8(bytes.clone()).map_err(|_| {
            ChartFileError::new(
                ChartFileErrorCategory::InvalidUtf8,
                format!("UTF-8 Mermaid 파일이 아닙니다: {}", display_name(path)),
            )
        })?;
        snapshot(path, source, &bytes)
    }

    fn write_atomic(
        &self,
        path: &str,
        source: &str,
    ) -> Result<DiagramFileSnapshot, ChartFileError> {
        let target = Path::new(path);
        let parent = target
            .parent()
            .filter(|value| !value.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut temporary = NamedTempFile::new_in(parent).map_err(|error| {
            ChartFileError::new(
                ChartFileErrorCategory::WriteFailed,
                format!(
                    "임시 저장 파일을 만들 수 없습니다: {}: {error}",
                    display_name(path)
                ),
            )
        })?;

        if let Ok(metadata) = fs::metadata(target) {
            let _ = temporary.as_file().set_permissions(metadata.permissions());
        }

        temporary
            .write_all(source.as_bytes())
            .map_err(|error| write_error(path, error))?;
        temporary
            .flush()
            .map_err(|error| write_error(path, error))?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|error| write_error(path, error))?;
        temporary.persist(target).map_err(|error| {
            ChartFileError::new(
                ChartFileErrorCategory::ReplaceFailed,
                format!(
                    "기존 파일을 안전하게 교체할 수 없습니다: {}: {}",
                    display_name(path),
                    error.error
                ),
            )
        })?;

        #[cfg(unix)]
        if let Ok(directory) = File::open(parent) {
            let _ = directory.sync_all();
        }

        self.read(path)
    }
}

fn snapshot(
    path: &str,
    source: String,
    bytes: &[u8],
) -> Result<DiagramFileSnapshot, ChartFileError> {
    let metadata = fs::metadata(path).map_err(|error| map_read_error(path, error))?;
    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64);
    let content_hash = format!("{:x}", Sha256::digest(bytes));

    Ok(DiagramFileSnapshot {
        source,
        binding: DocumentFileBinding {
            path: path.to_owned(),
            file_name: display_name(path),
            extension: normalized_extension(path)?,
            revision: FileRevision {
                content_hash,
                byte_length: bytes.len() as u64,
                modified_at,
            },
        },
    })
}

fn display_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(path)
        .to_owned()
}

fn map_read_error(path: &str, error: std::io::Error) -> ChartFileError {
    let category = match error.kind() {
        ErrorKind::NotFound => ChartFileErrorCategory::Missing,
        ErrorKind::PermissionDenied => ChartFileErrorCategory::PermissionDenied,
        _ => ChartFileErrorCategory::ReadFailed,
    };
    ChartFileError::new(
        category,
        format!("파일을 읽을 수 없습니다: {}: {error}", display_name(path)),
    )
}

fn write_error(path: &str, error: std::io::Error) -> ChartFileError {
    let category = if error.kind() == ErrorKind::PermissionDenied {
        ChartFileErrorCategory::PermissionDenied
    } else {
        ChartFileErrorCategory::WriteFailed
    };
    ChartFileError::new(
        category,
        format!("파일을 저장할 수 없습니다: {}: {error}", display_name(path)),
    )
}

#[cfg(test)]
mod tests {
    use super::LocalChartFileRepository;
    use crate::application::ports::ChartFileRepository;

    #[test]
    fn round_trips_utf8_and_line_endings() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("chart.mmd");
        let source = "flowchart LR\r\n  A --> 한글\r\n";

        let repository = LocalChartFileRepository;
        let written = repository
            .write_atomic(path.to_str().unwrap(), source)
            .unwrap();
        let read = repository.read(path.to_str().unwrap()).unwrap();

        assert_eq!(written.source, source);
        assert_eq!(read.source, source);
        assert_eq!(written.binding.revision, read.binding.revision);
    }
}
