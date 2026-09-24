mod fit;
mod input;
mod options;
mod png;
mod probe;
mod render_client;
mod terminal;
mod watch;

use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use mermaid_core::protocol::{RenderRequest, RequestKind};

use crate::fit::{compute_display_area, compute_placement, compute_target_box};
use crate::input::{exceeds_limit, is_blank, read_input, resolve_input_source};
use crate::options::{parse_arguments, DEFAULT_SCALE};
use crate::png::read_png_size;
use crate::probe::run_probe;
use crate::render_client::{request_render, RenderOutcome};
use crate::terminal::capability::{detect_capability, GraphicsSupport};
use crate::terminal::kitty_graphics::build_image_sequences;
use crate::terminal::metrics::read_metrics;
use crate::terminal::tty::Tty;
use crate::watch::run_watch;

/// Keeps the shell readable: renderer errors can be multi-line parse dumps.
const MAX_PROBLEM_LENGTH: usize = 200;

fn help() -> String {
    format!(
        "mmdcat - render a Mermaid chart inline in the terminal

Usage:
  mmdcat                read the chart from the clipboard
  mmdcat -              read the chart from stdin
  mmdcat <file>         read the chart from a file

Options:
  --probe               report terminal and render service readiness
  --watch               stay open and show every chart copied to the clipboard
  --clear               with --watch, replace the screen instead of appending
  --force               display images even when support could not be confirmed
  --scale <n>           rendering scale for high-resolution displays (default {})
  -h, --help            show this help

Text that is not a Mermaid chart produces no output.
",
        DEFAULT_SCALE as u32
    )
}

fn report_problem(message: &str) {
    let one_line = message.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = if one_line.chars().count() > MAX_PROBLEM_LENGTH {
        let head: String = one_line.chars().take(MAX_PROBLEM_LENGTH - 1).collect();
        format!("{head}\u{2026}")
    } else {
        one_line
    };

    eprintln!("mmdcat: {trimmed}");
}

/// Unique per invocation and safe as a window label, without a uuid crate.
pub fn new_request_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();

    format!("{:x}-{nanos:x}", std::process::id())
}

fn run() -> i32 {
    let parsed = parse_arguments(std::env::args().skip(1));
    let options = parsed.options;

    if options.help {
        print!("{}", help());
        return 0;
    }

    if options.probe {
        return run_probe();
    }

    if options.watch {
        return run_watch(&options);
    }

    let Some(text) = read_input(&resolve_input_source(&parsed.positionals)) else {
        return 0;
    };

    // Nothing to show and nothing worth saying: the user may simply have
    // pressed the key with an empty or unreadable selection.
    if is_blank(&text) || exceeds_limit(&text) {
        return 0;
    }

    // Rejecting here means a selection that is not a chart never opens a socket
    // and does not need the app to be running at all. The app still runs its own
    // detection on what it receives.
    if mermaid_core::chart::extract_mermaid_chart_source(&text).is_none() {
        return 0;
    }

    let mut tty = Tty::open();
    let capability = detect_capability(tty.as_mut());

    if capability.support != GraphicsSupport::Supported && !options.force {
        report_problem(&format!(
            "{} cannot display inline images ({}); use --force to try anyway",
            capability.terminal_name, capability.reason
        ));
        return 1;
    }

    let metrics = read_metrics(tty.as_mut());
    let area = compute_display_area(&metrics);
    let target = compute_target_box(&area, &metrics, options.scale);

    let outcome = request_render(&RenderRequest {
        kind: RequestKind::Render,
        request_id: new_request_id(),
        text,
        width_px: target.width_px,
        height_px: target.height_px,
        scale: options.scale,
    });

    let png = match outcome {
        RenderOutcome::NotAChart => return 0,
        RenderOutcome::Failed(message) | RenderOutcome::Unavailable(message) => {
            report_problem(&message);
            return 1;
        }
        RenderOutcome::Rendered { png, .. } => png,
    };

    let Some(size) = read_png_size(&png) else {
        report_problem("the rendered image was not a readable PNG");
        return 1;
    };

    let placement = compute_placement(size, &metrics, options.scale, &area);
    let sequences = build_image_sequences(&png, &placement);

    if sequences.is_empty() {
        report_problem("the rendered image was empty");
        return 1;
    }

    let output = sequences.concat();
    // The cursor stops at the end of the placement, so start a fresh line for
    // whatever the shell prints next.
    let output = format!("{output}\n");

    match tty.as_ref() {
        Some(tty) => tty.write(&output),
        None => {
            let mut stdout = std::io::stdout();
            let _ = stdout.write_all(output.as_bytes());
            let _ = stdout.flush();
        }
    }

    0
}

fn main() {
    std::process::exit(run());
}
