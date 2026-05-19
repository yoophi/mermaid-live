use crate::application::ports::DiagramAnalyzer;
use crate::domain::diagram::{DiagramSource, DiagramValidation};

pub struct SimpleDiagramAnalyzer;

impl DiagramAnalyzer for SimpleDiagramAnalyzer {
    fn analyze(&self, source: &DiagramSource) -> DiagramValidation {
        DiagramValidation {
            line_count: source.as_str().lines().count(),
            character_count: source.as_str().chars().count(),
        }
    }
}
