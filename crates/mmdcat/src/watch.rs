use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use mermaid_core::protocol::{RenderRequest, RenderResponse, RequestKind};

use crate::fit::{compute_display_area, compute_placement, compute_target_box};
use crate::options::Options;
use crate::png::read_png_size;
use crate::render_client::{launch_app, open_subscription};
use crate::terminal::capability::{detect_capability, GraphicsSupport};
use crate::terminal::kitty_graphics::build_image_sequences;
use crate::terminal::metrics::read_metrics;
use crate::terminal::tty::Tty;

/// How often the read loop wakes up to notice a resize or a lost connection.
const POLL_INTERVAL: Duration = Duration::from_millis(500);
/// Pause before re-subscribing, so a restarting app is not hammered.
const RECONNECT_DELAY: Duration = Duration::from_secs(1);
const CLEAR_SCREEN: &str = "\x1b[2J\x1b[H";

/// Set from the signal handler; the read loop acts on it.
static RESIZED: AtomicBool = AtomicBool::new(false);

extern "C" fn note_resize(_signal: libc::c_int) {
    RESIZED.store(true, Ordering::Relaxed);
}

fn watch_for_resize() {
    // Cast through the function pointer type; a direct cast of the function
    // item to an integer is not well defined.
    let handler = note_resize as extern "C" fn(libc::c_int) as libc::sighandler_t;

    // SAFETY: the handler only stores into an atomic, which is async-signal-safe.
    unsafe { libc::signal(libc::SIGWINCH, handler) };
}

/// Local wall clock, for labelling each chart as it arrives.
fn local_time() -> String {
    // SAFETY: `time` with a null pointer returns the value; `localtime_r`
    // writes into our own `tm`.
    let now = unsafe { libc::time(std::ptr::null_mut()) };
    let mut parts: libc::tm = unsafe { std::mem::zeroed() };
    unsafe { libc::localtime_r(&now, &mut parts) };

    format!(
        "{:02}:{:02}:{:02}",
        parts.tm_hour, parts.tm_min, parts.tm_sec
    )
}

/// Where watch output goes: the terminal itself when there is one.
enum Sink {
    Tty(Tty),
    Stdout,
}

impl Sink {
    fn write(&self, data: &str) {
        match self {
            Self::Tty(tty) => tty.write(data),
            Self::Stdout => {
                let mut stdout = std::io::stdout();
                let _ = stdout.write_all(data.as_bytes());
                let _ = stdout.flush();
            }
        }
    }
}

fn report(message: &str) {
    eprintln!("mmdcat: {message}");
}

fn show_chart(
    sink: &Sink,
    options: &Options,
    metrics: &crate::terminal::metrics::Metrics,
    area: &crate::fit::DisplayArea,
    png_base64: &str,
    label: Option<&str>,
) {
    let Ok(png) = BASE64.decode(png_base64.as_bytes()) else {
        report("a pushed image was unreadable");
        return;
    };

    let Some(size) = read_png_size(&png) else {
        report("a pushed image was not a readable PNG");
        return;
    };

    let placement = compute_placement(size, metrics, options.scale, area);
    let sequences = build_image_sequences(&png, &placement);
    if sequences.is_empty() {
        return;
    }

    let header = format!("{}  {}", local_time(), label.unwrap_or("Mermaid chart"));

    if options.clear {
        sink.write(CLEAR_SCREEN);
        sink.write(&format!("{header}\n"));
    } else {
        sink.write(&format!("\n{header}\n"));
    }

    sink.write(&sequences.concat());
    sink.write("\n");
}

/// Reads pushes until the connection ends or the terminal is resized.
///
/// Returns `true` when the caller should resubscribe immediately because the
/// size changed.
fn consume_pushes(
    stream: &mut std::os::unix::net::UnixStream,
    sink: &Sink,
    options: &Options,
    metrics: &crate::terminal::metrics::Metrics,
    area: &crate::fit::DisplayArea,
) -> bool {
    let mut pending = Vec::new();
    let mut chunk = [0u8; 8192];

    loop {
        if RESIZED.swap(false, Ordering::Relaxed) {
            return true;
        }

        match stream.read(&mut chunk) {
            Ok(0) => return false,
            Ok(count) => pending.extend_from_slice(&chunk[..count]),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                continue
            }
            Err(_) => return false,
        }

        while let Some(newline) = pending.iter().position(|byte| *byte == b'\n') {
            let line: Vec<u8> = pending.drain(..=newline).collect();
            let Ok(text) = std::str::from_utf8(&line) else {
                continue;
            };

            match serde_json::from_str::<RenderResponse>(text.trim()) {
                Ok(RenderResponse::Watching {
                    clipboard_watch_enabled,
                    ..
                }) => {
                    if clipboard_watch_enabled {
                        report("watching the clipboard; copy a Mermaid chart to see it here");
                    } else {
                        report("subscribed, but clipboard watching is turned off in the app's tray menu");
                    }
                }
                Ok(RenderResponse::Rendered {
                    png_base64, label, ..
                }) => show_chart(sink, options, metrics, area, &png_base64, label.as_deref()),
                Ok(RenderResponse::Failed { message, .. }) => report(&message),
                Ok(RenderResponse::NotAChart { .. }) | Err(_) => {}
            }
        }
    }
}

/// Shows every chart the app finds on the clipboard, until interrupted.
pub fn run_watch(options: &Options) -> i32 {
    let mut tty = Tty::open();
    let capability = detect_capability(tty.as_mut());

    if capability.support != GraphicsSupport::Supported && !options.force {
        report(&format!(
            "{} cannot display inline images ({}); use --force to try anyway",
            capability.terminal_name, capability.reason
        ));
        return 1;
    }

    watch_for_resize();

    if !launch_app() {
        report("could not start the desktop app; start it and try again");
    }

    let sink = match tty {
        Some(tty) => Sink::Tty(tty),
        None => Sink::Stdout,
    };

    loop {
        // Re-measured on every subscription so a resize takes effect.
        let mut probe = Tty::open();
        let metrics = read_metrics(probe.as_mut());
        drop(probe);

        let area = compute_display_area(&metrics);
        let target = compute_target_box(&area, &metrics, options.scale);

        let request = RenderRequest {
            kind: RequestKind::Watch,
            request_id: crate::new_request_id(),
            text: String::new(),
            width_px: target.width_px,
            height_px: target.height_px,
            scale: options.scale,
        };

        match open_subscription(&request, POLL_INTERVAL) {
            Ok(mut stream) => {
                if consume_pushes(&mut stream, &sink, options, &metrics, &area) {
                    // Resized: resubscribe at the new size without pausing.
                    continue;
                }
                report("the render service closed the subscription; reconnecting");
            }
            Err(error) => report(&format!("could not subscribe: {error}")),
        }

        std::thread::sleep(RECONNECT_DELAY);
    }
}
