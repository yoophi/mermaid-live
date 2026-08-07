use crate::application::ports::ChartFileRepository;
use crate::application::ports::DiagramAnalyzer;
use crate::domain::chart_document::{
    normalize_open_path, normalize_save_path, ChartFileError, DiagramFileSnapshot, FileRevision,
    SaveDiagramOutcome,
};
use crate::domain::diagram::{DiagramError, DiagramSource, DiagramValidation};

pub struct ValidateDiagramSource<A>
where
    A: DiagramAnalyzer,
{
    analyzer: A,
}

pub struct OpenChartFile<R: ChartFileRepository> {
    repository: R,
}

impl<R: ChartFileRepository> OpenChartFile<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn execute(&self, path: String) -> Result<DiagramFileSnapshot, ChartFileError> {
        self.repository.read(&normalize_open_path(&path)?)
    }
}

pub struct SaveChartFile<R: ChartFileRepository> {
    repository: R,
}

impl<R: ChartFileRepository> SaveChartFile<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn execute(
        &self,
        path: String,
        source: String,
        expected_revision: Option<FileRevision>,
        force: bool,
    ) -> Result<SaveDiagramOutcome, ChartFileError> {
        let path = normalize_save_path(&path)?;

        if let Some(expected) = expected_revision {
            let disk_snapshot = self.repository.read(&path)?;
            if !force && disk_snapshot.binding.revision != expected {
                return Ok(SaveDiagramOutcome::Conflict { disk_snapshot });
            }
        }

        let snapshot = self.repository.write_atomic(&path, &source)?;
        Ok(SaveDiagramOutcome::Saved { snapshot })
    }
}

#[cfg(test)]
mod chart_file_tests {
    use std::sync::{Arc, Mutex};

    use crate::application::ports::ChartFileRepository;
    use crate::domain::chart_document::{
        ChartFileError, DiagramFileSnapshot, DocumentFileBinding, FileRevision, SaveDiagramOutcome,
    };

    use super::{OpenChartFile, SaveChartFile};

    #[derive(Clone)]
    struct FakeRepository {
        snapshot: Arc<Mutex<DiagramFileSnapshot>>,
        writes: Arc<Mutex<usize>>,
    }

    impl FakeRepository {
        fn new(source: &str, hash: &str) -> Self {
            Self {
                snapshot: Arc::new(Mutex::new(snapshot(source, hash))),
                writes: Arc::new(Mutex::new(0)),
            }
        }
    }

    impl ChartFileRepository for FakeRepository {
        fn read(&self, _path: &str) -> Result<DiagramFileSnapshot, ChartFileError> {
            Ok(self.snapshot.lock().unwrap().clone())
        }

        fn write_atomic(
            &self,
            _path: &str,
            source: &str,
        ) -> Result<DiagramFileSnapshot, ChartFileError> {
            *self.writes.lock().unwrap() += 1;
            let next = snapshot(source, &format!("hash-{source}"));
            *self.snapshot.lock().unwrap() = next.clone();
            Ok(next)
        }
    }

    fn snapshot(source: &str, hash: &str) -> DiagramFileSnapshot {
        DiagramFileSnapshot {
            source: source.into(),
            binding: DocumentFileBinding {
                path: "/tmp/chart.mmd".into(),
                file_name: "chart.mmd".into(),
                extension: "mmd".into(),
                revision: FileRevision {
                    content_hash: hash.into(),
                    byte_length: source.len() as u64,
                    modified_at: None,
                },
            },
        }
    }

    #[test]
    fn opens_supported_file_through_repository() {
        let repository = FakeRepository::new("flowchart LR", "original");
        let opened = OpenChartFile::new(repository)
            .execute("/tmp/chart.mmd".into())
            .unwrap();
        assert_eq!(opened.source, "flowchart LR");
    }

    #[test]
    fn detects_conflict_without_writing() {
        let repository = FakeRepository::new("external", "external-hash");
        let writes = repository.writes.clone();
        let outcome = SaveChartFile::new(repository)
            .execute(
                "/tmp/chart.mmd".into(),
                "mine".into(),
                Some(FileRevision {
                    content_hash: "loaded-hash".into(),
                    byte_length: 6,
                    modified_at: None,
                }),
                false,
            )
            .unwrap();

        assert!(matches!(outcome, SaveDiagramOutcome::Conflict { .. }));
        assert_eq!(*writes.lock().unwrap(), 0);
    }

    #[test]
    fn force_overwrite_writes_after_conflict_decision() {
        let repository = FakeRepository::new("external", "external-hash");
        let writes = repository.writes.clone();
        let outcome = SaveChartFile::new(repository)
            .execute(
                "/tmp/chart.mmd".into(),
                "mine".into(),
                Some(FileRevision {
                    content_hash: "loaded-hash".into(),
                    byte_length: 6,
                    modified_at: None,
                }),
                true,
            )
            .unwrap();

        assert!(matches!(outcome, SaveDiagramOutcome::Saved { .. }));
        assert_eq!(*writes.lock().unwrap(), 1);
    }
}

impl<A> ValidateDiagramSource<A>
where
    A: DiagramAnalyzer,
{
    pub fn new(analyzer: A) -> Self {
        Self { analyzer }
    }

    pub fn execute(&self, raw_source: String) -> Result<DiagramValidation, DiagramError> {
        let source = DiagramSource::parse(raw_source)?;

        Ok(self.analyzer.analyze(&source))
    }
}
