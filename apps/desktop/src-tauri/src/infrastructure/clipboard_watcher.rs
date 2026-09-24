use std::sync::atomic::Ordering;
use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::adapters::inbound::{render_socket, tray_menu};
use crate::adapters::outbound::chart_inbox::StagedChart;
use crate::adapters::outbound::system_notifier::SystemChartNotifier;
use crate::adapters::outbound::{native_window_manager, temp_diagram_file};
use crate::application::ports::ChartNotifier;
use crate::domain::clipboard_chart::DetectedChart;
use crate::infrastructure::app_state::AppState;
use crate::infrastructure::clipboard_import;

/// Each tick reads only the clipboard change counter, so a short interval keeps
/// detection close to the copy without measurable cost.
const POLL_INTERVAL: Duration = Duration::from_millis(600);

/// Watches the clipboard for the lifetime of the application, including while
/// no window is open.
pub fn start(app: &AppHandle) {
    let app = app.clone();

    std::thread::spawn(move || loop {
        std::thread::sleep(POLL_INTERVAL);

        if !is_enabled(&app) {
            continue;
        }

        // Copying while the app is in the foreground needs no announcement: the
        // user is already looking at the chart, and copying chart text is
        // routine inside an editor. The detection below still runs so the same
        // content is not announced later, once they switch away.
        let app_was_active = clipboard_import::is_app_active();

        let Some(chart) = detect(&app) else {
            continue;
        };

        // A subscribed terminal shows the chart where the user is already
        // looking, so it is neither staged nor announced. `detect` has already
        // marked it seen, so it will not resurface later either.
        if render_socket::deliver_to_subscribers(&app, &chart) {
            continue;
        }

        if app_was_active {
            continue;
        }

        // Windows may only be built on the main thread.
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || {
            if let Some(staged) = stage(&handle, chart) {
                announce(&handle, &staged);
            }
        });
    });
}

/// Imports when the user brings the app forward.
///
/// The detection runs again here so that turning background watching off still
/// leaves the original behaviour of picking up the clipboard on activation.
pub fn import_on_activation(app: &AppHandle) {
    if let Some(chart) = detect(app) {
        stage(app, chart);
    }

    present_latest(app);
}

/// Shows the newest chart the user has not seen yet.
pub fn present_latest(app: &AppHandle) {
    match app.state::<AppState>().chart_inbox.claim_latest() {
        Some(staged) => present(app, &staged),
        None => tray_menu::refresh(app),
    }
}

/// Shows one specific chart, for the tray menu history.
pub fn present_chart(app: &AppHandle, fingerprint: &str) {
    if let Some(staged) = app.state::<AppState>().chart_inbox.claim(fingerprint) {
        present(app, &staged);
    }
}

/// The dock icon was clicked while the app had no visible window.
pub fn reopen(app: &AppHandle) {
    if unopened_count(app) > 0 {
        present_latest(app);
        return;
    }

    if let Err(error) = native_window_manager::open_editor_window(app) {
        eprintln!("[clipboard] failed to reopen an editor window: {error}");
    }
}

pub fn is_enabled(app: &AppHandle) -> bool {
    app.state::<AppState>()
        .clipboard_watch_enabled
        .load(Ordering::Relaxed)
}

pub fn toggle_enabled(app: &AppHandle) {
    let state = app.state::<AppState>();
    let enabled = !state.clipboard_watch_enabled.load(Ordering::Relaxed);
    state
        .clipboard_watch_enabled
        .store(enabled, Ordering::Relaxed);

    tray_menu::refresh(app);
}

pub fn unopened_count(app: &AppHandle) -> usize {
    app.state::<AppState>().chart_inbox.unopened_count()
}

pub fn staged_charts(app: &AppHandle) -> Vec<StagedChart> {
    app.state::<AppState>().chart_inbox.list()
}

fn detect(app: &AppHandle) -> Option<DetectedChart> {
    app.state::<AppState>().detect_clipboard_chart.execute()
}

/// Writes the chart to disk and renders it in a hidden window, so that showing
/// it later costs nothing.
fn stage(app: &AppHandle, chart: DetectedChart) -> Option<StagedChart> {
    let path = match temp_diagram_file::write_temp_diagram_file(chart.source(), chart.fingerprint())
    {
        Ok(path) => path,
        Err(error) => {
            eprintln!("[clipboard] failed to write a temp diagram: {error}");
            return None;
        }
    };

    let state = app.state::<AppState>();

    // Only the newest chart keeps a hidden window, capping the cost at one
    // webview no matter how much is copied.
    if let Some(previous) = state.chart_inbox.release_prewarmed() {
        native_window_manager::close_temp_diagram_window(app, &previous);
    }

    let prewarmed =
        match native_window_manager::prewarm_temp_diagram_window(app, &path, chart.fingerprint()) {
            Ok(()) => true,
            Err(error) => {
                eprintln!("[clipboard] failed to pre-render a diagram window: {error}");
                false
            }
        };

    let staged = StagedChart {
        chart,
        path,
        prewarmed,
        opened: false,
    };
    state.chart_inbox.push(staged.clone());

    Some(staged)
}

fn announce(app: &AppHandle, staged: &StagedChart) {
    SystemChartNotifier::new(app.clone()).notify_detected(&staged.chart, unopened_count(app));

    tray_menu::refresh(app);
}

fn present(app: &AppHandle, staged: &StagedChart) {
    // Showing a window activates the app. Without this the resulting focus
    // event would look like a fresh activation and present a second chart.
    clipboard_import::note_app_activated();

    let fingerprint = staged.chart.fingerprint();
    if !native_window_manager::reveal_temp_diagram_window(app, fingerprint) {
        if let Err(error) =
            native_window_manager::open_temp_diagram_window(app, &staged.path, fingerprint)
        {
            eprintln!("[clipboard] failed to open a diagram window: {error}");
        }
    }

    tray_menu::refresh(app);
}
