use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::atomic::Ordering;
use std::thread;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use mermaid_core::protocol::{
    is_valid_request_id, socket_path, RenderRequest, RenderResponse, RequestKind,
};
use tauri::{AppHandle, Manager};

use crate::adapters::outbound::webview_rasterizer::WebviewRasterizer;
use crate::application::use_cases::{RenderChartOutcome, RenderChartToPng};
use crate::domain::chart_raster::RasterSpec;
use crate::domain::clipboard_chart::DetectedChart;
use crate::infrastructure::app_state::AppState;

/// A request is one JSON line holding a chart; anything larger is not one.
const MAX_REQUEST_BYTES: u64 = 1024 * 1024;

#[derive(Debug)]
enum ParsedRequest {
    Render {
        request_id: String,
        text: String,
        spec: RasterSpec,
    },
    Watch {
        request_id: String,
        spec: RasterSpec,
    },
    Invalid {
        request_id: String,
        message: String,
    },
}

fn parse_request(line: &str) -> ParsedRequest {
    let request: RenderRequest = match serde_json::from_str(line) {
        Ok(request) => request,
        Err(error) => {
            return ParsedRequest::Invalid {
                request_id: String::new(),
                message: format!("malformed request: {error}"),
            }
        }
    };

    if !is_valid_request_id(&request.request_id) {
        return ParsedRequest::Invalid {
            request_id: String::new(),
            message: "request id must be 1-64 characters of letters, digits or dashes".to_string(),
        };
    }

    let spec = match RasterSpec::new(request.width_px, request.height_px, request.scale) {
        Ok(spec) => spec,
        Err(error) => {
            return ParsedRequest::Invalid {
                request_id: request.request_id,
                message: error.to_string(),
            }
        }
    };

    match request.kind {
        RequestKind::Render => ParsedRequest::Render {
            request_id: request.request_id,
            text: request.text,
            spec,
        },
        RequestKind::Watch => ParsedRequest::Watch {
            request_id: request.request_id,
            spec,
        },
    }
}

fn render(app: &AppHandle, request_id: &str, text: &str, spec: RasterSpec) -> RenderResponse {
    let state = app.state::<AppState>();
    let use_case = RenderChartToPng::new(WebviewRasterizer::new(app.clone()), &state.raster_cache);

    match use_case.execute(text, spec) {
        Ok(RenderChartOutcome::NotAChart) => RenderResponse::NotAChart {
            request_id: request_id.to_string(),
        },
        Ok(RenderChartOutcome::Rendered { fingerprint, png }) => {
            // The user has just been shown this chart, so the clipboard watcher
            // must not announce it again.
            state.detect_clipboard_chart.mark_seen(&fingerprint);

            RenderResponse::Rendered {
                request_id: request_id.to_string(),
                fingerprint,
                png_base64: BASE64.encode(png),
                label: None,
            }
        }
        Err(error) => RenderResponse::failed(request_id, error.to_string()),
    }
}

fn handle_connection(app: &AppHandle, stream: UnixStream) {
    let Ok(read_half) = stream.try_clone() else {
        return;
    };

    let mut line = String::new();
    if BufReader::new(read_half.take(MAX_REQUEST_BYTES))
        .read_line(&mut line)
        .is_err()
    {
        return;
    }

    let response = match parse_request(line.trim_end()) {
        ParsedRequest::Invalid {
            request_id,
            message,
        } => RenderResponse::failed(request_id, message),
        ParsedRequest::Render {
            request_id,
            text,
            spec,
        } => render(app, &request_id, &text, spec),
        ParsedRequest::Watch { request_id, spec } => {
            hold_subscription(app, stream, request_id, spec);
            return;
        }
    };

    let Ok(encoded) = serde_json::to_string(&response) else {
        return;
    };

    let mut write_half = stream;
    let _ = write_half.write_all(format!("{encoded}\n").as_bytes());
    let _ = write_half.flush();
}

fn write_line(stream: &mut UnixStream, response: &RenderResponse) -> bool {
    let Ok(encoded) = serde_json::to_string(response) else {
        return false;
    };

    stream
        .write_all(format!("{encoded}\n").as_bytes())
        .and_then(|()| stream.flush())
        .is_ok()
}

/// Keeps a subscriber registered until its terminal goes away.
///
/// The connection carries no further requests; reading it is only how the end
/// of the client is noticed.
fn hold_subscription(app: &AppHandle, mut stream: UnixStream, request_id: String, spec: RasterSpec) {
    let state = app.state::<AppState>();

    let acknowledgement = RenderResponse::Watching {
        request_id: request_id.clone(),
        clipboard_watch_enabled: state.clipboard_watch_enabled.load(Ordering::Relaxed),
    };

    if !write_line(&mut stream, &acknowledgement) {
        return;
    }

    let Ok(push_half) = stream.try_clone() else {
        return;
    };

    let id = state.subscribers.register(request_id, spec, push_half);

    let mut discard = [0u8; 64];
    loop {
        match stream.read(&mut discard) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
    }

    state.subscribers.unregister(id);
}

/// Shows a newly detected chart in every subscribed terminal.
///
/// Returns whether anyone was listening, which is how the clipboard watcher
/// knows the user has already been shown the chart.
pub fn deliver_to_subscribers(app: &AppHandle, chart: &DetectedChart) -> bool {
    if app.state::<AppState>().subscribers.is_empty() {
        return false;
    }

    let app = app.clone();
    let chart = chart.clone();

    // Rendering can take seconds; the clipboard watcher must keep polling.
    thread::spawn(move || {
        let state = app.state::<AppState>();
        let use_case =
            RenderChartToPng::new(WebviewRasterizer::new(app.clone()), &state.raster_cache);

        for (id, request_id, spec) in state.subscribers.targets() {
            let response = match use_case.execute(chart.source(), spec) {
                Ok(RenderChartOutcome::Rendered { fingerprint, png }) => RenderResponse::Rendered {
                    request_id,
                    fingerprint,
                    png_base64: BASE64.encode(png),
                    label: Some(chart.label().to_string()),
                },
                // The watcher only offers charts, so this means the source
                // stopped parsing between detection and rendering.
                Ok(RenderChartOutcome::NotAChart) => continue,
                Err(error) => RenderResponse::failed(request_id, error.to_string()),
            };

            let Ok(encoded) = serde_json::to_string(&response) else {
                continue;
            };

            state.subscribers.send(id, &format!("{encoded}\n"));
        }
    });

    true
}

/// Listens for render requests from the terminal CLI.
///
/// Runs for the lifetime of the app, next to the clipboard watcher, so the
/// terminal can reach the renderer whenever the app is up.
pub fn start(app: &AppHandle) {
    let app = app.clone();

    thread::spawn(move || {
        let path = socket_path();

        if let Some(parent) = path.parent() {
            if let Err(error) = std::fs::create_dir_all(parent) {
                eprintln!("[render] could not create the socket directory: {error}");
                return;
            }
        }

        // A socket file left behind by a previous run would refuse the bind.
        let _ = std::fs::remove_file(&path);

        let listener = match UnixListener::bind(&path) {
            Ok(listener) => listener,
            Err(error) => {
                eprintln!("[render] could not listen on {}: {error}", path.display());
                return;
            }
        };

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let app = app.clone();
                    thread::spawn(move || handle_connection(&app, stream));
                }
                Err(error) => eprintln!("[render] connection failed: {error}"),
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{parse_request, ParsedRequest};

    fn valid_line(extra: &str) -> String {
        format!(
            r#"{{"requestId":"abc-123","text":"flowchart LR","widthPx":800,"heightPx":600{extra}}}"#
        )
    }

    #[test]
    fn accepts_a_well_formed_request() {
        match parse_request(&valid_line(r#","scale":2"#)) {
            ParsedRequest::Render {
                request_id,
                text,
                spec,
            } => {
                assert_eq!(request_id, "abc-123");
                assert_eq!(text, "flowchart LR");
                assert_eq!(spec.width_px(), 800);
                assert_eq!(spec.height_px(), 600);
                assert_eq!(spec.scale(), 2.0);
            }
            other => panic!("unexpected rejection: {other:?}"),
        }
    }

    #[test]
    fn defaults_the_scale_when_it_is_omitted() {
        match parse_request(&valid_line("")) {
            ParsedRequest::Render { spec, .. } => assert_eq!(spec.scale(), 1.0),
            other => panic!("unexpected rejection: {other:?}"),
        }
    }

    #[test]
    fn accepts_a_watch_request() {
        let line = r#"{"kind":"watch","requestId":"abc","widthPx":800,"heightPx":600,"scale":2}"#;
        match parse_request(line) {
            ParsedRequest::Watch { request_id, spec } => {
                assert_eq!(request_id, "abc");
                assert_eq!(spec.width_px(), 800);
            }
            other => panic!("expected a watch request, got {other:?}"),
        }
    }

    #[test]
    fn rejects_malformed_json() {
        match parse_request("not json at all") {
            ParsedRequest::Invalid { message, .. } => assert!(message.contains("malformed")),
            other => panic!("expected a rejection, got {other:?}"),
        }
    }

    #[test]
    fn rejects_a_missing_field() {
        match parse_request(r#"{"requestId":"abc","text":"flowchart LR"}"#) {
            ParsedRequest::Invalid { message, .. } => assert!(message.contains("malformed")),
            other => panic!("expected a rejection, got {other:?}"),
        }
    }

    #[test]
    fn rejects_an_unusable_size() {
        match parse_request(r#"{"requestId":"abc","text":"x","widthPx":0,"heightPx":600}"#) {
            ParsedRequest::Invalid { message, .. } => assert!(message.contains("size")),
            other => panic!("expected a rejection, got {other:?}"),
        }
    }

    #[test]
    fn rejects_a_request_id_that_would_not_be_safe_in_a_window_label() {
        match parse_request(r#"{"requestId":"../../etc/passwd","text":"x","widthPx":8,"heightPx":6}"#)
        {
            ParsedRequest::Invalid { message, .. } => assert!(message.contains("request id")),
            other => panic!("expected a rejection, got {other:?}"),
        }
    }
}
