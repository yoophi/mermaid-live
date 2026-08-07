use tauri::menu::{Menu, MenuItemBuilder, MenuItemKind, Submenu, SubmenuBuilder};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::adapters::outbound::native_window_manager;

const ABOUT_MENU_ID: &str = "about_mermaid_live";
const APP_DISPLAY_NAME: &str = "Mermaid Live";
const APP_VERSION: &str = env!("MERMAID_LIVE_PACKAGE_VERSION");
const BUILD_COMMIT_HASH: &str = env!("MERMAID_LIVE_GIT_COMMIT_HASH");
const BUILD_COMMIT_TAG: &str = env!("MERMAID_LIVE_GIT_COMMIT_TAG");
const BUILD_METADATA_FALLBACK: &str = "unknown";
const OPEN_DOCUMENT_EVENT: &str = "open-chart-document-request";
const SAVE_DOCUMENT_EVENT: &str = "save-chart-document-request";
const SAVE_DOCUMENT_AS_EVENT: &str = "save-chart-document-as-request";
const NEW_WINDOW_MENU_ID: &str = "new_window";
const NEW_TAB_MENU_ID: &str = "new_tab";
const OPEN_FILE_MENU_ID: &str = "open_file";
const SAVE_FILE_MENU_ID: &str = "save_file";
const SAVE_FILE_AS_MENU_ID: &str = "save_file_as";
const MERGE_ALL_WINDOWS_MENU_ID: &str = "merge_all_windows";

pub fn setup_window_menu(app: &AppHandle) -> tauri::Result<()> {
    let menu = Menu::default(app)?;

    install_about_menu_item(app, &menu)?;

    let new_window_item = MenuItemBuilder::new("새 창")
        .id(NEW_WINDOW_MENU_ID)
        .accelerator("CmdOrCtrl+N")
        .build(app)?;
    let new_tab_item = MenuItemBuilder::new("새 탭")
        .id(NEW_TAB_MENU_ID)
        .accelerator("CmdOrCtrl+T")
        .build(app)?;
    let open_item = MenuItemBuilder::new("열기...")
        .id(OPEN_FILE_MENU_ID)
        .accelerator("CmdOrCtrl+O")
        .build(app)?;
    let save_item = MenuItemBuilder::new("저장")
        .id(SAVE_FILE_MENU_ID)
        .accelerator("CmdOrCtrl+S")
        .build(app)?;
    let save_as_item = MenuItemBuilder::new("다른 이름으로 저장...")
        .id(SAVE_FILE_AS_MENU_ID)
        .accelerator("CmdOrCtrl+Shift+S")
        .build(app)?;

    let mut placed_in_file_menu = false;
    for item in menu.items()? {
        if let MenuItemKind::Submenu(submenu) = item {
            if submenu.text().map(|text| text == "File").unwrap_or(false) {
                submenu.insert(&new_window_item, 0)?;
                submenu.insert(&new_tab_item, 1)?;
                submenu.insert(&open_item, 2)?;
                submenu.insert(&save_item, 3)?;
                submenu.insert(&save_as_item, 4)?;
                placed_in_file_menu = true;
                break;
            }
        }
    }

    if !placed_in_file_menu {
        let file_menu = SubmenuBuilder::new(app, "파일")
            .item(&new_window_item)
            .item(&new_tab_item)
            .item(&open_item)
            .item(&save_item)
            .item(&save_as_item)
            .build()?;
        menu.append(&file_menu)?;
    }

    let merge_all_item = MenuItemBuilder::new("모든 창 합치기")
        .id(MERGE_ALL_WINDOWS_MENU_ID)
        .accelerator("Ctrl+Cmd+M")
        .build(app)?;

    let window_menu = SubmenuBuilder::new(app, "창")
        .item(&merge_all_item)
        .build()?;

    menu.append(&window_menu)?;
    app.set_menu(menu)?;
    Ok(())
}

pub fn handle_window_menu_event(app: &AppHandle, id: &str) {
    match id {
        ABOUT_MENU_ID => show_about_dialog(app),
        NEW_WINDOW_MENU_ID => {
            if let Err(error) = native_window_manager::open_editor_window(app) {
                eprintln!("[window] failed to create editor window: {error}");
            }
        }
        NEW_TAB_MENU_ID => native_window_manager::open_editor_tab(app),
        OPEN_FILE_MENU_ID => emit_document_request(app, OPEN_DOCUMENT_EVENT),
        SAVE_FILE_MENU_ID => emit_document_request(app, SAVE_DOCUMENT_EVENT),
        SAVE_FILE_AS_MENU_ID => emit_document_request(app, SAVE_DOCUMENT_AS_EVENT),
        MERGE_ALL_WINDOWS_MENU_ID => native_window_manager::merge_all_windows(app),
        _ => {}
    }
}

fn install_about_menu_item(app: &AppHandle, menu: &Menu<tauri::Wry>) -> tauri::Result<()> {
    let about_item = MenuItemBuilder::new(format!("About {APP_DISPLAY_NAME}"))
        .id(ABOUT_MENU_ID)
        .build(app)?;

    #[cfg(target_os = "macos")]
    let target_menu = find_submenu(menu, APP_DISPLAY_NAME)?;
    #[cfg(not(target_os = "macos"))]
    let target_menu = find_submenu(menu, "Help")?;

    if let Some(target_menu) = target_menu {
        target_menu.remove_at(0)?;
        target_menu.insert(&about_item, 0)?;
    }

    Ok(())
}

fn find_submenu(
    menu: &Menu<tauri::Wry>,
    title: &str,
) -> tauri::Result<Option<Submenu<tauri::Wry>>> {
    for item in menu.items()? {
        if let MenuItemKind::Submenu(submenu) = item {
            if submenu.text().map(|text| text == title).unwrap_or(false) {
                return Ok(Some(submenu));
            }
        }
    }

    Ok(None)
}

fn show_about_dialog(app: &AppHandle) {
    app.dialog()
        .message(format!(
            "{APP_DISPLAY_NAME}\n\nVersion: {APP_VERSION}\nCommit: {}\nTag: {}",
            display_build_metadata(BUILD_COMMIT_HASH),
            display_build_metadata(BUILD_COMMIT_TAG)
        ))
        .title(format!("About {APP_DISPLAY_NAME}"))
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::Ok)
        .show(|_| {});
}

fn display_build_metadata(value: &'static str) -> &'static str {
    if value.trim().is_empty() {
        BUILD_METADATA_FALLBACK
    } else {
        value
    }
}

fn emit_document_request(app: &AppHandle, event: &str) {
    let Some((_, window)) = app
        .webview_windows()
        .into_iter()
        .find(|(_, window)| window.is_focused().unwrap_or(false))
    else {
        return;
    };

    if let Err(error) = window.emit(event, ()) {
        eprintln!("[window] failed to emit document request {event}: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::{
        display_build_metadata, BUILD_METADATA_FALLBACK, OPEN_DOCUMENT_EVENT, OPEN_FILE_MENU_ID,
        SAVE_DOCUMENT_AS_EVENT, SAVE_DOCUMENT_EVENT, SAVE_FILE_AS_MENU_ID, SAVE_FILE_MENU_ID,
    };

    #[test]
    fn blank_build_metadata_uses_fallback() {
        assert_eq!(display_build_metadata("  "), BUILD_METADATA_FALLBACK);
    }

    #[test]
    fn available_build_metadata_is_preserved() {
        assert_eq!(display_build_metadata("abc123"), "abc123");
    }

    #[test]
    fn document_menu_items_have_registered_event_contracts() {
        assert_eq!(OPEN_FILE_MENU_ID, "open_file");
        assert_eq!(OPEN_DOCUMENT_EVENT, "open-chart-document-request");
        assert_eq!(SAVE_FILE_MENU_ID, "save_file");
        assert_eq!(SAVE_DOCUMENT_EVENT, "save-chart-document-request");
        assert_eq!(SAVE_FILE_AS_MENU_ID, "save_file_as");
        assert_eq!(SAVE_DOCUMENT_AS_EVENT, "save-chart-document-as-request");
    }
}
