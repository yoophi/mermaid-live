use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use tauri::{AppHandle, Manager, Window, WindowEvent};

use crate::infrastructure::clipboard_watcher;

static APP_ACTIVE: OnceLock<Mutex<bool>> = OnceLock::new();

pub fn handle_window_event(window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::Focused(false) => schedule_inactive_check(window.app_handle().clone()),
        WindowEvent::Focused(true) => {
            if mark_app_activated() {
                clipboard_watcher::import_on_activation(window.app_handle());
            }
        }
        _ => {}
    }
}

/// Records that the app is in the foreground, for when it was brought there by
/// the tray rather than by the user switching to it.
pub fn note_app_activated() {
    set_app_active(true);
}

/// Whether the app is in the foreground.
///
/// This reads the tracked flag instead of asking AppKit, so the watcher thread
/// can call it: `is_application_active` only answers correctly on the main
/// thread and reports `true` everywhere else.
pub fn is_app_active() -> bool {
    with_app_active(|active| *active)
}

fn schedule_inactive_check(app: AppHandle) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(120));

        let app_for_main_thread = app.clone();
        let _ = app.run_on_main_thread(move || {
            if !is_application_active(&app_for_main_thread) {
                set_app_active(false);
            }
        });
    });
}

fn mark_app_activated() -> bool {
    with_app_active(|active| {
        if *active {
            false
        } else {
            *active = true;
            true
        }
    })
}

fn set_app_active(active: bool) {
    with_app_active(|state| {
        *state = active;
    });
}

fn with_app_active<T>(f: impl FnOnce(&mut bool) -> T) -> T {
    let state = APP_ACTIVE.get_or_init(|| Mutex::new(false));
    let mut active = state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut active)
}

#[cfg(target_os = "macos")]
fn is_application_active(_app: &AppHandle) -> bool {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSApplication;

    let Some(mtm) = MainThreadMarker::new() else {
        return true;
    };

    NSApplication::sharedApplication(mtm).isActive()
}

#[cfg(not(target_os = "macos"))]
fn is_application_active(app: &AppHandle) -> bool {
    app.webview_windows()
        .values()
        .any(|window| window.is_focused().unwrap_or(false))
}
