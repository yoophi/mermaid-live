use crate::application::ports::DiagramAnalyzer;
use crate::domain::diagram::{DiagramError, DiagramSource, DiagramValidation};

pub struct ValidateDiagramSource<A>
where
    A: DiagramAnalyzer,
{
    analyzer: A,
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
