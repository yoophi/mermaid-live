use crate::adapters::outbound::local_chart_file_repository::LocalChartFileRepository;
use crate::adapters::outbound::simple_diagram_analyzer::SimpleDiagramAnalyzer;
use crate::application::use_cases::{OpenChartFile, SaveChartFile, ValidateDiagramSource};

pub type ValidateDiagramSourceUseCase = ValidateDiagramSource<SimpleDiagramAnalyzer>;

pub struct AppState {
    pub validate_diagram_source: ValidateDiagramSourceUseCase,
    pub open_chart_file: OpenChartFile<LocalChartFileRepository>,
    pub save_chart_file: SaveChartFile<LocalChartFileRepository>,
}

pub fn build_app_state() -> AppState {
    let repository = LocalChartFileRepository;
    AppState {
        validate_diagram_source: ValidateDiagramSource::new(SimpleDiagramAnalyzer),
        open_chart_file: OpenChartFile::new(repository),
        save_chart_file: SaveChartFile::new(repository),
    }
}
