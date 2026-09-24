use mermaid_core::protocol::socket_path;

use crate::fit::compute_display_area;
use crate::input::{read_input, InputSource};
use crate::render_client::is_service_reachable;
use crate::terminal::capability::{detect_capability, Capability, GraphicsSupport};
use crate::terminal::metrics::read_metrics;
use crate::terminal::tty::Tty;

/// Chars of clipboard text shown, enough to recognise a selection.
const CLIPBOARD_PREVIEW_LENGTH: usize = 60;
const LABEL_WIDTH: usize = 18;

/// Shows what the CLI would read.
///
/// Selecting text does not reach the system clipboard on every terminal, and
/// without this the symptom is silence: a stale clipboard holding something
/// that is not a chart looks exactly like a key that never fired.
fn describe_clipboard() -> String {
    let Some(text) = read_input(&InputSource::Clipboard) else {
        return "could not be read".to_string();
    };

    if text.trim().is_empty() {
        return "empty".to_string();
    }

    let first_line = text.trim_start().lines().next().unwrap_or_default();
    let preview: String = if first_line.chars().count() > CLIPBOARD_PREVIEW_LENGTH {
        let head: String = first_line.chars().take(CLIPBOARD_PREVIEW_LENGTH).collect();
        format!("{head}\u{2026}")
    } else {
        first_line.to_string()
    };

    format!("{} bytes, starts with: {preview}", text.len())
}

fn describe_support(capability: &Capability) -> &'static str {
    match capability.support {
        GraphicsSupport::Supported => "yes",
        GraphicsSupport::Unsupported => "no",
        GraphicsSupport::Unknown => "unknown",
    }
}

/// Reports whether this terminal and this machine are ready, so a failure can
/// be traced to the terminal, the shell, or the desktop app.
pub fn run_probe() -> i32 {
    let mut tty = Tty::open();
    let has_tty = tty.is_some();

    let capability = detect_capability(tty.as_mut());
    let metrics = read_metrics(tty.as_mut());
    let area = compute_display_area(&metrics);
    let reachable = is_service_reachable();

    let cell_size = match (metrics.cell_width_px, metrics.cell_height_px) {
        (Some(width), Some(height)) => format!("{width}x{height}px"),
        _ => "unavailable".to_string(),
    };

    let rows: Vec<(&str, String)> = vec![
        ("terminal", capability.terminal_name.clone()),
        ("controlling tty", if has_tty { "yes" } else { "no" }.to_string()),
        (
            "multiplexer",
            capability.multiplexer.clone().unwrap_or_else(|| "none".to_string()),
        ),
        (
            "inline images",
            format!("{} ({})", describe_support(&capability), capability.reason),
        ),
        ("grid", format!("{}x{} cells", metrics.columns, metrics.rows)),
        (
            "cell size",
            format!("{cell_size} ({})", metrics.cell_source.label()),
        ),
        (
            "display area",
            format!("{}x{} cells", area.available_columns, area.available_rows),
        ),
        ("clipboard", describe_clipboard()),
        ("render socket", socket_path().display().to_string()),
        (
            "render service",
            if reachable { "reachable" } else { "not reachable" }.to_string(),
        ),
    ];

    for (label, value) in rows {
        println!("{label:<LABEL_WIDTH$}{value}");
    }

    if capability.support == GraphicsSupport::Unsupported {
        return 1;
    }

    if reachable {
        0
    } else {
        1
    }
}
