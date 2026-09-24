use std::sync::atomic::AtomicBool;

use crate::adapters::outbound::chart_inbox::ChartInbox;
use crate::adapters::outbound::clipboard_reader::SystemClipboard;
use crate::adapters::outbound::simple_diagram_analyzer::SimpleDiagramAnalyzer;
use crate::application::raster_cache::RasterCache;
use crate::application::use_cases::{DetectClipboardChart, ValidateDiagramSource};
use crate::infrastructure::render_service::{PendingRenders, Subscribers};

pub type ValidateDiagramSourceUseCase = ValidateDiagramSource<SimpleDiagramAnalyzer>;
pub type DetectClipboardChartUseCase = DetectClipboardChart<SystemClipboard>;

pub struct AppState {
    pub validate_diagram_source: ValidateDiagramSourceUseCase,
    pub detect_clipboard_chart: DetectClipboardChartUseCase,
    pub chart_inbox: ChartInbox,
    /// Recently rendered chart images, so re-showing a chart skips the webview.
    pub raster_cache: RasterCache,
    /// Renders waiting on their hidden window to report back.
    pub pending_renders: PendingRenders,
    /// Terminals subscribed to charts as the clipboard watcher finds them.
    pub subscribers: Subscribers,
    /// Whether the background watcher polls. Kept in memory only, so the app
    /// starts watching again after a restart.
    pub clipboard_watch_enabled: AtomicBool,
}

pub fn build_app_state() -> AppState {
    AppState {
        validate_diagram_source: ValidateDiagramSource::new(SimpleDiagramAnalyzer),
        detect_clipboard_chart: DetectClipboardChart::new(SystemClipboard),
        chart_inbox: ChartInbox::default(),
        raster_cache: RasterCache::default(),
        pending_renders: PendingRenders::default(),
        subscribers: Subscribers::default(),
        clipboard_watch_enabled: AtomicBool::new(true),
    }
}
