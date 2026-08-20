use std::sync::atomic::{AtomicU64, Ordering};

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

const EDITOR_WINDOW_TITLE: &str = "Mermaid Live";
const EDITOR_WINDOW_WIDTH: f64 = 1280.0;
const EDITOR_WINDOW_HEIGHT: f64 = 800.0;
const EDITOR_WINDOW_MIN_WIDTH: f64 = 960.0;
const EDITOR_WINDOW_MIN_HEIGHT: f64 = 620.0;
const ABOUT_WINDOW_LABEL: &str = "about-window";
const ABOUT_WINDOW_TITLE: &str = "About Mermaid Live";
const ABOUT_WINDOW_WIDTH: f64 = 404.0;
const ABOUT_WINDOW_HEIGHT: f64 = 248.0;

#[cfg(target_os = "macos")]
const EDITOR_TABBING_IDENTIFIER: &str = "mermaid-live-editor";
#[cfg(target_os = "macos")]
const ABOUT_TABBING_IDENTIFIER: &str = "mermaid-live-about";

static WINDOW_COUNTER: AtomicU64 = AtomicU64::new(1);
static TAB_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn open_editor_window(app: &AppHandle) -> tauri::Result<()> {
    let label = format!("editor-{}", WINDOW_COUNTER.fetch_add(1, Ordering::Relaxed));
    build_default_editor_window(app, label)?;
    Ok(())
}

pub fn open_about_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(ABOUT_WINDOW_LABEL) {
        let _ = window.unminimize();
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }

    let mut builder = WebviewWindowBuilder::new(
        app,
        ABOUT_WINDOW_LABEL,
        WebviewUrl::App("index.html?view=about".into()),
    )
    .title(ABOUT_WINDOW_TITLE)
    .inner_size(ABOUT_WINDOW_WIDTH, ABOUT_WINDOW_HEIGHT)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .decorations(false)
    .visible(true)
    .focused(true)
    .center();

    #[cfg(target_os = "macos")]
    {
        builder = builder.tabbing_identifier(ABOUT_TABBING_IDENTIFIER);
    }

    let window = builder.build()?;

    #[cfg(target_os = "macos")]
    {
        use objc2_app_kit::{NSWindow, NSWindowTabbingMode};

        let window_pointer = window.ns_window()?;
        let native_window: &NSWindow = unsafe { &*window_pointer.cast::<NSWindow>() };
        native_window.setTabbingMode(NSWindowTabbingMode::Disallowed);
    }

    window.set_size(tauri::LogicalSize::new(
        ABOUT_WINDOW_WIDTH,
        ABOUT_WINDOW_HEIGHT,
    ))?;
    window.center()?;
    window.show()?;
    window.set_focus()?;

    Ok(())
}

pub fn open_temp_diagram_window(
    app: &AppHandle,
    source_file: impl AsRef<std::path::Path>,
    md5: &str,
) -> tauri::Result<()> {
    let label = temp_diagram_window_label(md5);
    if let Some(window) = app.get_webview_window(&label) {
        let _ = window.set_focus();
        return Ok(());
    }

    let encoded =
        utf8_percent_encode(&source_file.as_ref().to_string_lossy(), NON_ALPHANUMERIC).to_string();

    build_editor_window_with_url(
        app,
        label,
        WebviewUrl::App(format!("index.html?sourceFile={encoded}").into()),
        "Mermaid Live - Clipboard".to_string(),
    )?;
    Ok(())
}

pub fn temp_diagram_window_label(md5: &str) -> String {
    format!("clipboard-{md5}")
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

fn build_default_editor_window(app: &AppHandle, label: String) -> tauri::Result<()> {
    build_editor_window_with_url(
        app,
        label,
        WebviewUrl::App("index.html".into()),
        EDITOR_WINDOW_TITLE.to_string(),
    )
}

fn build_editor_window_with_url(
    app: &AppHandle,
    label: String,
    url: WebviewUrl,
    title: String,
) -> tauri::Result<()> {
    let mut builder = WebviewWindowBuilder::new(app, label, url)
        .title(title)
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
