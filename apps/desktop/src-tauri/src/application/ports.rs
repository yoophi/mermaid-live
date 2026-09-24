use crate::domain::chart_raster::{RasterError, RasterSpec};
use crate::domain::clipboard_chart::{ClipboardRevision, DetectedChart};
use crate::domain::diagram::{DiagramSource, DiagramValidation};

pub trait DiagramAnalyzer: Send + Sync {
    fn analyze(&self, source: &DiagramSource) -> DiagramValidation;
}

/// Reads the clipboard of the host system.
pub trait ClipboardSource: Send + Sync {
    /// Cheap change marker that must not read the clipboard contents.
    ///
    /// Content reads are the expensive and privacy sensitive part — on macOS
    /// they can raise a paste permission alert — so the watcher polls this
    /// instead and only reads when the revision moves. Implementations that
    /// cannot report a revision without reading return `None`.
    fn revision(&self) -> Option<ClipboardRevision>;

    /// Returns the clipboard text, or `None` when the clipboard holds no plain
    /// text or the owning application marked its contents as concealed.
    fn read_text(&self) -> Option<String>;
}

/// Announces a detected chart outside of the application windows, for when the
/// user is working somewhere else.
pub trait ChartNotifier: Send + Sync {
    fn notify_detected(&self, chart: &DetectedChart, pending_count: usize);
}

/// Turns a chart into a raster image that fits the requested box.
pub trait ChartRasterizer: Send + Sync {
    fn rasterize(&self, chart: &DetectedChart, spec: RasterSpec) -> Result<Vec<u8>, RasterError>;
}
