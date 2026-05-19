use crate::adapters::outbound::simple_diagram_analyzer::SimpleDiagramAnalyzer;
use crate::application::use_cases::ValidateDiagramSource;

pub type ValidateDiagramSourceUseCase = ValidateDiagramSource<SimpleDiagramAnalyzer>;

pub struct AppState {
    pub validate_diagram_source: ValidateDiagramSourceUseCase,
}

pub fn build_app_state() -> AppState {
    AppState {
        validate_diagram_source: ValidateDiagramSource::new(SimpleDiagramAnalyzer),
    }
}
