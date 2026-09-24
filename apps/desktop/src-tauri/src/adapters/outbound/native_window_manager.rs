use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

const EDITOR_WINDOW_TITLE: &str = "Mermaid Live";
const RASTERIZE_WINDOW_TITLE: &str = "Mermaid Live - Rasterize";
const CLIPBOARD_WINDOW_TITLE: &str = "Mermaid Live - Clipboard";
const EDITOR_WINDOW_WIDTH: f64 = 1280.0;
const EDITOR_WINDOW_HEIGHT: f64 = 800.0;
const EDITOR_WINDOW_MIN_WIDTH: f64 = 960.0;
const EDITOR_WINDOW_MIN_HEIGHT: f64 = 620.0;

#[cfg(target_os = "macos")]
const EDITOR_TABBING_IDENTIFIER: &str = "mermaid-live-editor";

static WINDOW_COUNTER: AtomicU64 = AtomicU64::new(1);
static TAB_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn open_editor_window(app: &AppHandle) -> tauri::Result<()> {
    let label = format!("editor-{}", WINDOW_COUNTER.fetch_add(1, Ordering::Relaxed));
    build_editor_window_with_url(
        app,
        label,
        WebviewUrl::App("index.html".into()),
        EDITOR_WINDOW_TITLE.to_string(),
        true,
    )
}

pub fn temp_diagram_window_label(fingerprint: &str) -> String {
    format!("clipboard-{fingerprint}")
}

/// Shows the chart in a window, reusing the one already holding it if any.
pub fn open_temp_diagram_window(
    app: &AppHandle,
    source_file: impl AsRef<Path>,
    fingerprint: &str,
) -> tauri::Result<()> {
    if reveal_temp_diagram_window(app, fingerprint) {
        return Ok(());
    }

    build_editor_window_with_url(
        app,
        temp_diagram_window_label(fingerprint),
        temp_diagram_window_url(source_file.as_ref()),
        CLIPBOARD_WINDOW_TITLE.to_string(),
        true,
    )
}

/// Builds the window hidden so the webview loads its bundle and renders the
/// chart ahead of time; revealing it later is then immediate.
pub fn prewarm_temp_diagram_window(
    app: &AppHandle,
    source_file: impl AsRef<Path>,
    fingerprint: &str,
) -> tauri::Result<()> {
    let label = temp_diagram_window_label(fingerprint);
    if app.get_webview_window(&label).is_some() {
        return Ok(());
    }

    build_editor_window_with_url(
        app,
        label,
        temp_diagram_window_url(source_file.as_ref()),
        CLIPBOARD_WINDOW_TITLE.to_string(),
        false,
    )
}

/// Brings the window holding a chart forward, reporting whether one existed.
pub fn reveal_temp_diagram_window(app: &AppHandle, fingerprint: &str) -> bool {
    let Some(window) = app.get_webview_window(&temp_diagram_window_label(fingerprint)) else {
        return false;
    };

    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
    true
}

pub fn close_temp_diagram_window(app: &AppHandle, fingerprint: &str) {
    if let Some(window) = app.get_webview_window(&temp_diagram_window_label(fingerprint)) {
        let _ = window.close();
    }
}

pub fn open_editor_tab(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    open_macos_editor_tab(app);

    #[cfg(not(target_os = "macos"))]
    {
        let _ = open_editor_window(app);
    }
}

pub fn merge_all_windows(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        use objc2::MainThreadMarker;
        use objc2_app_kit::NSApplication;

        let _ = app.run_on_main_thread(|| {
            if let Some(mtm) = MainThreadMarker::new() {
                let ns_app = NSApplication::sharedApplication(mtm);
                if let Some(window) = ns_app.keyWindow() {
                    window.mergeAllWindows(None);
                }
            }
        });
    }
}

fn temp_diagram_window_url(source_file: &Path) -> WebviewUrl {
    let encoded = utf8_percent_encode(&source_file.to_string_lossy(), NON_ALPHANUMERIC).to_string();

    WebviewUrl::App(format!("index.html?sourceFile={encoded}").into())
}

fn build_editor_window_with_url(
    app: &AppHandle,
    label: String,
    url: WebviewUrl,
    title: String,
    visible: bool,
) -> tauri::Result<()> {
    let mut builder = WebviewWindowBuilder::new(app, label, url)
        .title(title)
        .visible(visible)
        .inner_size(EDITOR_WINDOW_WIDTH, EDITOR_WINDOW_HEIGHT)
        .min_inner_size(EDITOR_WINDOW_MIN_WIDTH, EDITOR_WINDOW_MIN_HEIGHT);

    #[cfg(target_os = "macos")]
    {
        builder = builder.tabbing_identifier(EDITOR_TABBING_IDENTIFIER);
    }

    builder.build()?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn open_macos_editor_tab(app: &AppHandle) {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSWindow, NSWindowOrderingMode};

    let app = app.clone();
    let _ = app.clone().run_on_main_thread(move || {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };

        let base = NSApplication::sharedApplication(mtm).keyWindow();
        let label = format!("editor-tab-{}", TAB_COUNTER.fetch_add(1, Ordering::Relaxed));
        let built = WebviewWindowBuilder::new(&app, label, WebviewUrl::App("index.html".into()))
            .title(EDITOR_WINDOW_TITLE)
            .inner_size(EDITOR_WINDOW_WIDTH, EDITOR_WINDOW_HEIGHT)
            .min_inner_size(EDITOR_WINDOW_MIN_WIDTH, EDITOR_WINDOW_MIN_HEIGHT)
            .tabbing_identifier(EDITOR_TABBING_IDENTIFIER)
            .build();

        let new_window = match built {
            Ok(window) => window,
            Err(error) => {
                eprintln!("[window] failed to create tab window: {error}");
                return;
            }
        };

        if let Some(base) = base {
            if base.tabbingIdentifier().to_string() == EDITOR_TABBING_IDENTIFIER {
                if let Ok(ptr) = new_window.ns_window() {
                    let new_ns_window: &NSWindow = unsafe { &*ptr.cast::<NSWindow>() };
                    base.addTabbedWindow_ordered(new_ns_window, NSWindowOrderingMode::Above);
                }
            }
        }
    });
}

pub fn rasterize_window_label(request_id: &str) -> String {
    format!("rasterize-{request_id}")
}

/// Builds the hidden window a chart is rasterised in.
///
/// Deliberately outside the editor tab group: it must never surface as a tab
/// the user can reach, and it is closed as soon as the image is delivered.
pub fn open_rasterize_window(
    app: &AppHandle,
    source_file: &Path,
    request_id: &str,
    width_px: u32,
    height_px: u32,
    scale: f64,
) -> tauri::Result<()> {
    let source_path = source_file.to_string_lossy();
    let encoded_path = utf8_percent_encode(&source_path, NON_ALPHANUMERIC).to_string();
    let encoded_id = utf8_percent_encode(request_id, NON_ALPHANUMERIC).to_string();
    let url = WebviewUrl::App(
        format!(
            "index.html?rasterize=1&sourceFile={encoded_path}&requestId={encoded_id}&w={width_px}&h={height_px}&scale={scale}"
        )
        .into(),
    );

    WebviewWindowBuilder::new(app, rasterize_window_label(request_id), url)
        .title(RASTERIZE_WINDOW_TITLE)
        .visible(false)
        .inner_size(EDITOR_WINDOW_WIDTH, EDITOR_WINDOW_HEIGHT)
        .build()?;

    Ok(())
}

pub fn close_rasterize_window(app: &AppHandle, request_id: &str) {
    if let Some(window) = app.get_webview_window(&rasterize_window_label(request_id)) {
        let _ = window.close();
    }
}
