// Chart detection and the raster size contract live in `mermaid-core` so the
// terminal CLI shares one definition with the app. Re-exported under the
// original module names to keep every `crate::domain::...` path working.
pub use mermaid_core::chart as mermaid_chart;
pub use mermaid_core::raster as chart_raster;

pub mod clipboard_chart;
pub mod diagram;
