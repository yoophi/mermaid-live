use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use tauri::{AppHandle, Manager, Window, WindowEvent};

use crate::adapters::outbound::{clipboard_reader, native_window_manager, temp_diagram_file};
use crate::domain::mermaid_chart;

static APP_ACTIVE: OnceLock<Mutex<bool>> = OnceLock::new();
static TEMP_WINDOW_INDEX: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

pub fn handle_window_event(window: &Window, event: &WindowEvent) {
    if matches!(event, WindowEvent::Destroyed) {
        remove_temp_window(window.label());
        return;
    }

    match event {
        WindowEvent::Focused(false) => {
            schedule_inactive_check(window.app_handle().clone());
            return;
        }
        WindowEvent::Focused(true) => {
            if !mark_app_activated() {
                return;
            }
        }
        _ => return,
    }

    import_clipboard_mermaid_chart(window);
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

fn import_clipboard_mermaid_chart(window: &Window) {
    let app = window.app_handle().clone();
    let Ok(source) = clipboard_reader::read_clipboard_text() else {
        return;
    };

    let Some(source) = mermaid_chart::extract_mermaid_chart_source(&source) else {
        return;
    };

    let md5 = temp_diagram_file::diagram_source_md5(&source);
    if focus_existing_temp_window(&app, &md5) {
        return;
    }

    let path = match temp_diagram_file::write_temp_diagram_file(&source, &md5) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("[clipboard] failed to write temp diagram: {error}");
            return;
        }
    };

    register_temp_window(md5.clone());

    if let Err(error) = native_window_manager::open_temp_diagram_window(&app, path, &md5) {
        remove_temp_window(&native_window_manager::temp_diagram_window_label(&md5));
        eprintln!("[clipboard] failed to open temp diagram window: {error}");
    }
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

fn focus_existing_temp_window(app: &tauri::AppHandle, md5: &str) -> bool {
    let label = temp_window_label_for_md5(md5)
        .unwrap_or_else(|| native_window_manager::temp_diagram_window_label(md5));

    if let Some(window) = app.get_webview_window(&label) {
        register_temp_window(md5.to_string());
        let _ = window.set_focus();
        return true;
    }

    remove_temp_window(&label);
    false
}

fn register_temp_window(md5: String) {
    let label = native_window_manager::temp_diagram_window_label(&md5);
    with_temp_window_index(|index| {
        index.insert(md5, label);
    });
}

fn remove_temp_window(label: &str) {
    with_temp_window_index(|index| {
        index.retain(|_, window_label| window_label != label);
    });
}

fn temp_window_label_for_md5(md5: &str) -> Option<String> {
    with_temp_window_index(|index| index.get(md5).cloned())
}

fn with_temp_window_index<T>(f: impl FnOnce(&mut HashMap<String, String>) -> T) -> T {
    let state = TEMP_WINDOW_INDEX.get_or_init(|| Mutex::new(HashMap::new()));
    let mut index = state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut index)
}
