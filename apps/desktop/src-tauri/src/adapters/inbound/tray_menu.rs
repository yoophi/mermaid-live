use tauri::menu::{CheckMenuItemBuilder, Menu, MenuBuilder, MenuItem, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Wry};

use crate::adapters::outbound::chart_inbox::StagedChart;
use crate::adapters::outbound::native_window_manager;
use crate::infrastructure::clipboard_watcher;

const TRAY_ID: &str = "clipboard-inbox";
const TRAY_ICON: &[u8] = include_bytes!("../../../icons/tray.png");

const OPEN_LATEST_ID: &str = "clipboard_open_latest";
const TOGGLE_WATCH_ID: &str = "clipboard_toggle_watch";
const NEW_WINDOW_ID: &str = "clipboard_new_window";
const QUIT_ID: &str = "clipboard_quit";
const OPEN_CHART_PREFIX: &str = "clipboard_open_chart:";

/// The menu bar entry is the reliable way to reach a detected chart: desktop
/// notifications cannot deliver clicks, and the app may have no window open.
pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(tauri::image::Image::from_bytes(TRAY_ICON)?)
        .icon_as_template(true)
        .tooltip("Mermaid Live")
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                clipboard_watcher::present_latest(tray.app_handle());
            }
        })
        .build(app)?;

    refresh(app);
    Ok(())
}

/// Rebuilds the menu and the badge from the current inbox contents.
pub fn refresh(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };

    match build_menu(app) {
        Ok(menu) => {
            if let Err(error) = tray.set_menu(Some(menu)) {
                eprintln!("[tray] failed to update the menu: {error}");
            }
        }
        Err(error) => eprintln!("[tray] failed to build the menu: {error}"),
    }

    let unopened = clipboard_watcher::unopened_count(app);
    if let Err(error) = tray.set_title((unopened > 0).then(|| unopened.to_string())) {
        eprintln!("[tray] failed to update the badge: {error}");
    }
}

/// Handles the ids owned by the tray, reporting whether the id belonged here.
pub fn handle_menu_event(app: &AppHandle, id: &str) -> bool {
    match id {
        OPEN_LATEST_ID => clipboard_watcher::present_latest(app),
        TOGGLE_WATCH_ID => clipboard_watcher::toggle_enabled(app),
        NEW_WINDOW_ID => {
            if let Err(error) = native_window_manager::open_editor_window(app) {
                eprintln!("[tray] failed to create an editor window: {error}");
            }
        }
        QUIT_ID => app.exit(0),
        _ => {
            let Some(fingerprint) = id.strip_prefix(OPEN_CHART_PREFIX) else {
                return false;
            };

            clipboard_watcher::present_chart(app, fingerprint);
        }
    }

    true
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let charts = clipboard_watcher::staged_charts(app);
    let unopened = charts.iter().filter(|staged| !staged.opened).count();

    let open_latest = MenuItemBuilder::with_id(OPEN_LATEST_ID, open_latest_text(unopened))
        .enabled(unopened > 0)
        .build(app)?;
    let toggle_watch = CheckMenuItemBuilder::with_id(TOGGLE_WATCH_ID, "클립보드 감시")
        .checked(clipboard_watcher::is_enabled(app))
        .build(app)?;

    let chart_items = charts
        .iter()
        .map(|staged| {
            MenuItemBuilder::with_id(
                format!("{OPEN_CHART_PREFIX}{}", staged.chart.fingerprint()),
                chart_item_text(staged),
            )
            .build(app)
        })
        .collect::<tauri::Result<Vec<MenuItem<Wry>>>>()?;

    let mut builder = MenuBuilder::new(app).item(&open_latest);

    if !chart_items.is_empty() {
        builder = builder.separator();
        for item in &chart_items {
            builder = builder.item(item);
        }
    }

    builder
        .separator()
        .item(&toggle_watch)
        .separator()
        .text(NEW_WINDOW_ID, "새 창")
        .text(QUIT_ID, "종료")
        .build()
}

fn open_latest_text(unopened: usize) -> String {
    match unopened {
        0 => "감지된 차트 없음".to_string(),
        1 => "차트 1개 감지됨 — 열기".to_string(),
        count => format!("차트 {count}개 감지됨 — 최신 열기"),
    }
}

fn chart_item_text(staged: &StagedChart) -> String {
    let marker = if staged.opened { '○' } else { '●' };

    format!("{marker} {}", staged.chart.label())
}
