use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::process::{Command, Stdio};
use std::time::Duration;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use mermaid_core::protocol::{socket_path, RenderRequest, RenderResponse};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(1);
/// Creating the render webview on a cold app can take a while.
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);
const LAUNCH_RETRY_DELAY: Duration = Duration::from_millis(700);
const LAUNCH_RETRY_ATTEMPTS: usize = 20;

fn app_name() -> String {
    std::env::var("MERMAID_LIVE_APP").unwrap_or_else(|_| "Mermaid Live".to_string())
}

#[derive(Debug)]
pub enum RenderOutcome {
    Rendered { png: Vec<u8> },
    NotAChart,
    Failed(String),
    /// The render service could not be reached at all.
    Unavailable(String),
}

/// Distinguishes "nobody is listening" from "the service said no".
enum SendError {
    Unavailable,
    Failed(String),
}

fn send_once(request: &RenderRequest) -> Result<RenderOutcome, SendError> {
    let path = socket_path();
    let stream = UnixStream::connect(&path)
        .map_err(|_| SendError::Unavailable)?;

    stream
        .set_write_timeout(Some(CONNECT_TIMEOUT))
        .and_then(|()| stream.set_read_timeout(Some(RESPONSE_TIMEOUT)))
        .map_err(|error| SendError::Failed(error.to_string()))?;

    let line = serde_json::to_string(request)
        .map_err(|error| SendError::Failed(error.to_string()))?;

    {
        let mut writer = &stream;
        writer
            .write_all(format!("{line}\n").as_bytes())
            .and_then(|()| writer.flush())
            .map_err(|_| SendError::Unavailable)?;
    }

    let mut reply = String::new();
    BufReader::new(&stream)
        .read_line(&mut reply)
        .map_err(|error| SendError::Failed(error.to_string()))?;

    if reply.trim().is_empty() {
        return Err(SendError::Unavailable);
    }

    let response: RenderResponse = serde_json::from_str(reply.trim()).map_err(|_| {
        SendError::Failed("the render service sent an unexpected response".to_string())
    })?;

    Ok(match response {
        RenderResponse::Rendered { png_base64, .. } => {
            match BASE64.decode(png_base64.as_bytes()) {
                Ok(png) => RenderOutcome::Rendered { png },
                Err(error) => {
                    RenderOutcome::Failed(format!("the image data was unreadable: {error}"))
                }
            }
        }
        RenderResponse::NotAChart { .. } => RenderOutcome::NotAChart,
        RenderResponse::Failed { message, .. } => RenderOutcome::Failed(message),
        // Only a watch subscription is answered this way.
        RenderResponse::Watching { .. } => {
            RenderOutcome::Failed("the render service answered a render with a subscription".to_string())
        }
    })
}

/// True when the socket accepts a connection right now.
pub fn is_service_reachable() -> bool {
    UnixStream::connect(socket_path()).is_ok()
}

pub fn launch_app() -> bool {
    Command::new("open")
        .args(["-a", &app_name()])
        // `open` complains on its own stderr when the app is missing; the CLI
        // reports that in one line of its own.
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Sends the request, starting the desktop app once if the socket is not there.
///
/// Rendering runs inside the app's webview, so an app that is not running is a
/// recoverable condition rather than an error.
pub fn request_render(request: &RenderRequest) -> RenderOutcome {
    match send_once(request) {
        Ok(outcome) => return outcome,
        Err(SendError::Failed(message)) => return RenderOutcome::Failed(message),
        Err(SendError::Unavailable) => {}
    }

    let name = app_name();

    if !launch_app() {
        return RenderOutcome::Unavailable(format!(
            "could not start {name}; start it and try again (socket: {})",
            socket_path().display()
        ));
    }

    for _ in 0..LAUNCH_RETRY_ATTEMPTS {
        std::thread::sleep(LAUNCH_RETRY_DELAY);

        match send_once(request) {
            Ok(outcome) => return outcome,
            Err(SendError::Failed(message)) => return RenderOutcome::Failed(message),
            Err(SendError::Unavailable) => {}
        }
    }

    RenderOutcome::Unavailable(format!(
        "{name} did not open a render service; check that it is running (socket: {})",
        socket_path().display()
    ))
}

/// Opens a subscription and returns the stream the app will push charts on.
///
/// The read timeout is what lets the caller notice a resize between pushes.
pub fn open_subscription(
    request: &RenderRequest,
    read_timeout: Duration,
) -> Result<UnixStream, String> {
    let path = socket_path();
    let stream = UnixStream::connect(&path).map_err(|error| error.to_string())?;

    stream
        .set_write_timeout(Some(CONNECT_TIMEOUT))
        .and_then(|()| stream.set_read_timeout(Some(read_timeout)))
        .map_err(|error| error.to_string())?;

    let line = serde_json::to_string(request).map_err(|error| error.to_string())?;

    let mut writer = &stream;
    writer
        .write_all(format!("{line}\n").as_bytes())
        .and_then(|()| writer.flush())
        .map_err(|error| error.to_string())?;

    Ok(stream)
}
