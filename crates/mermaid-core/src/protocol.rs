//! The render request path between the terminal CLI and the desktop app.
//!
//! One definition serves both ends, so the client cannot encode a request the
//! server would not accept. The shape is documented in
//! `specs/002-terminal-inline-render/contracts/render-socket.md`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Sits beside the staged `.mmd` files the clipboard route already writes.
const DIRECTORY: &str = "mermaid-live";
const SOCKET_FILE_NAME: &str = "render.sock";

const MAX_REQUEST_ID_LENGTH: usize = 64;

/// Where the app listens and the CLI connects.
///
/// `MERMAID_LIVE_SOCKET` overrides it, which keeps a second app instance from
/// fighting over the same path during development.
pub fn socket_path() -> PathBuf {
    if let Some(path) = std::env::var_os("MERMAID_LIVE_SOCKET") {
        return PathBuf::from(path);
    }

    std::env::temp_dir().join(DIRECTORY).join(SOCKET_FILE_NAME)
}

fn default_scale() -> f64 {
    1.0
}

/// What the client is asking for.
///
/// Defaults to `Render` so a client that predates watch mode, and therefore
/// sends no `kind`, still speaks a valid request.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RequestKind {
    #[default]
    Render,
    /// Keep the connection open and push every chart the app detects.
    Watch,
}

/// A request to render one chart, or to subscribe to detected charts.
///
/// `width_px` and `height_px` are device pixels and already include `scale`, so
/// the logical box is `width_px / scale`. `text` carries the chart for a render
/// request and is unused by a watch request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderRequest {
    #[serde(default)]
    pub kind: RequestKind,
    pub request_id: String,
    #[serde(default)]
    pub text: String,
    pub width_px: u32,
    pub height_px: u32,
    #[serde(default = "default_scale")]
    pub scale: f64,
}

/// What became of a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum RenderResponse {
    Rendered {
        request_id: String,
        fingerprint: String,
        png_base64: String,
        /// Short name for the chart, sent on watch pushes so the client can
        /// label them. Absent on one-shot renders.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
    /// A watch subscription is live. Carries the app's clipboard watch state so
    /// the client can say why nothing is arriving.
    Watching {
        request_id: String,
        clipboard_watch_enabled: bool,
    },
    /// The text was not a chart. Not an error: either side hands over whatever
    /// the user selected.
    NotAChart { request_id: String },
    Failed {
        request_id: String,
        message: String,
    },
}

impl RenderResponse {
    pub fn failed(request_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Failed {
            request_id: request_id.into(),
            message: message.into(),
        }
    }
}

/// Request ids become window labels and URL parameters, so only simple ones are
/// accepted. Checked on the server; the client generates ids that satisfy it.
pub fn is_valid_request_id(request_id: &str) -> bool {
    !request_id.is_empty()
        && request_id.len() <= MAX_REQUEST_ID_LENGTH
        && request_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
}

#[cfg(test)]
mod tests {
    use super::{is_valid_request_id, RenderRequest, RenderResponse, RequestKind};

    #[test]
    fn reads_a_request_in_wire_form() {
        let line = r#"{"requestId":"abc-123","text":"flowchart LR","widthPx":800,"heightPx":600,"scale":2}"#;
        let request: RenderRequest = serde_json::from_str(line).expect("valid request");

        assert_eq!(request.request_id, "abc-123");
        assert_eq!(request.width_px, 800);
        assert_eq!(request.scale, 2.0);
    }

    #[test]
    fn defaults_the_scale_when_it_is_omitted() {
        let line = r#"{"requestId":"abc","text":"x","widthPx":800,"heightPx":600}"#;
        let request: RenderRequest = serde_json::from_str(line).expect("valid request");

        assert_eq!(request.scale, 1.0);
    }

    #[test]
    fn a_request_without_a_kind_is_a_render_request() {
        let line = r#"{"requestId":"abc","text":"x","widthPx":8,"heightPx":6}"#;
        let request: RenderRequest = serde_json::from_str(line).expect("valid request");

        assert_eq!(request.kind, RequestKind::Render);
    }

    #[test]
    fn reads_a_watch_request_without_text() {
        let line = r#"{"kind":"watch","requestId":"abc","widthPx":800,"heightPx":600,"scale":2}"#;
        let request: RenderRequest = serde_json::from_str(line).expect("valid request");

        assert_eq!(request.kind, RequestKind::Watch);
        assert!(request.text.is_empty());
        assert_eq!(request.width_px, 800);
    }

    #[test]
    fn writes_a_watch_acknowledgement() {
        let watching = RenderResponse::Watching {
            request_id: "abc".to_string(),
            clipboard_watch_enabled: false,
        };

        assert_eq!(
            serde_json::to_string(&watching).expect("serialises"),
            r#"{"status":"watching","requestId":"abc","clipboardWatchEnabled":false}"#
        );
    }

    #[test]
    fn a_label_is_carried_only_when_present() {
        let labelled = RenderResponse::Rendered {
            request_id: "abc".to_string(),
            fingerprint: "ff".to_string(),
            png_base64: "AAAA".to_string(),
            label: Some("flowchart LR".to_string()),
        };
        assert!(serde_json::to_string(&labelled)
            .expect("serialises")
            .contains(r#""label":"flowchart LR""#));

        let plain = RenderResponse::Rendered {
            request_id: "abc".to_string(),
            fingerprint: "ff".to_string(),
            png_base64: "AAAA".to_string(),
            label: None,
        };
        assert!(!serde_json::to_string(&plain)
            .expect("serialises")
            .contains("label"));
    }

    #[test]
    fn writes_responses_in_wire_form() {
        let rendered = RenderResponse::Rendered {
            request_id: "abc".to_string(),
            fingerprint: "ff".to_string(),
            png_base64: "AAAA".to_string(),
            label: None,
        };
        let encoded = serde_json::to_string(&rendered).expect("serialises");

        assert!(encoded.contains(r#""status":"rendered""#));
        assert!(encoded.contains(r#""requestId":"abc""#));
        assert!(encoded.contains(r#""pngBase64":"AAAA""#));

        let not_a_chart = RenderResponse::NotAChart {
            request_id: "abc".to_string(),
        };
        assert_eq!(
            serde_json::to_string(&not_a_chart).expect("serialises"),
            r#"{"status":"notAChart","requestId":"abc"}"#
        );
    }

    #[test]
    fn a_response_survives_a_round_trip() {
        let original = RenderResponse::failed("abc", "boom");
        let encoded = serde_json::to_string(&original).expect("serialises");
        let decoded: RenderResponse = serde_json::from_str(&encoded).expect("deserialises");

        match decoded {
            RenderResponse::Failed {
                request_id,
                message,
            } => {
                assert_eq!(request_id, "abc");
                assert_eq!(message, "boom");
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }

    #[test]
    fn rejects_request_ids_that_would_not_be_safe_in_a_window_label() {
        assert!(is_valid_request_id("abc-123"));
        assert!(!is_valid_request_id(""));
        assert!(!is_valid_request_id("../../etc/passwd"));
        assert!(!is_valid_request_id("has space"));
        assert!(!is_valid_request_id(&"a".repeat(65)));
    }
}
