use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::adapters::outbound::{native_window_manager, temp_diagram_file};
use crate::application::ports::ChartRasterizer;
use crate::domain::chart_raster::{RasterError, RasterSpec};
use crate::domain::clipboard_chart::DetectedChart;
use crate::infrastructure::app_state::AppState;
use crate::infrastructure::render_service::RenderReply;

/// Generous enough for a cold webview to load its bundle and render, short
/// enough that a wedged render does not hold a terminal forever.
const RENDER_TIMEOUT: Duration = Duration::from_secs(25);

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Rasterises charts in a hidden window of the running app.
///
/// Reusing the app's webview keeps one Mermaid version and one theme for both
/// the on-screen preview and the terminal image.
pub struct WebviewRasterizer {
    app: AppHandle,
}

impl WebviewRasterizer {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl ChartRasterizer for WebviewRasterizer {
    fn rasterize(&self, chart: &DetectedChart, spec: RasterSpec) -> Result<Vec<u8>, RasterError> {
        // The webview loads the chart from a file, the same way the clipboard
        // route already hands charts to a window.
        let source_file =
            temp_diagram_file::write_temp_diagram_file(chart.source(), chart.fingerprint())
                .map_err(RasterError::Renderer)?;

        // The fingerprint is hex, so the id stays safe in a window label and a URL.
        let request_id = format!(
            "{}-{}",
            chart.fingerprint(),
            REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed)
        );

        let receiver = self
            .app
            .state::<AppState>()
            .pending_renders
            .register(request_id.clone());

        // Windows may only be built on the main thread.
        let app = self.app.clone();
        let window_request_id = request_id.clone();
        self.app
            .run_on_main_thread(move || {
                let opened = native_window_manager::open_rasterize_window(
                    &app,
                    &source_file,
                    &window_request_id,
                    spec.width_px(),
                    spec.height_px(),
                    spec.scale(),
                );

                if let Err(error) = opened {
                    app.state::<AppState>().pending_renders.complete(
                        &window_request_id,
                        RenderReply::Failed(format!("could not open a render window: {error}")),
                    );
                }
            })
            .map_err(|error| RasterError::Renderer(error.to_string()))?;

        let reply = receiver.recv_timeout(RENDER_TIMEOUT);

        // The window has done its job either way, including on timeout.
        let closing_app = self.app.clone();
        let closing_request_id = request_id.clone();
        let _ = self.app.run_on_main_thread(move || {
            native_window_manager::close_rasterize_window(&closing_app, &closing_request_id);
        });

        match reply {
            Ok(RenderReply::Png(png)) => Ok(png),
            Ok(RenderReply::Failed(message)) => Err(RasterError::Renderer(message)),
            Err(_) => {
                self.app
                    .state::<AppState>()
                    .pending_renders
                    .cancel(&request_id);
                Err(RasterError::Timeout)
            }
        }
    }
}
