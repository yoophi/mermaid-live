use crate::domain::diagram::{DiagramSource, DiagramValidation};

pub trait DiagramAnalyzer: Send + Sync {
    fn analyze(&self, source: &DiagramSource) -> DiagramValidation;
}
