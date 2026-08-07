use crate::domain::chart_document::{ChartFileError, DiagramFileSnapshot};
use crate::domain::diagram::{DiagramSource, DiagramValidation};

pub trait DiagramAnalyzer: Send + Sync {
    fn analyze(&self, source: &DiagramSource) -> DiagramValidation;
}

pub trait ChartFileRepository: Clone + Send + Sync + 'static {
    fn read(&self, path: &str) -> Result<DiagramFileSnapshot, ChartFileError>;
    fn write_atomic(&self, path: &str, source: &str)
        -> Result<DiagramFileSnapshot, ChartFileError>;
}
