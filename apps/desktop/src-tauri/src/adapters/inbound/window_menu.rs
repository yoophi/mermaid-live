use tauri::menu::{Menu, MenuItemBuilder, MenuItemKind, SubmenuBuilder};
use tauri::AppHandle;

use crate::adapters::outbound::native_window_manager;

pub fn setup_window_menu(app: &AppHandle) -> tauri::Result<()> {
    let menu = Menu::default(app)?;

    let new_window_item = MenuItemBuilder::new("새 창")
        .id("new_window")
        .accelerator("Cmd+N")
        .build(app)?;
    let new_tab_item = MenuItemBuilder::new("새 탭")
        .id("new_tab")
        .accelerator("Cmd+T")
        .build(app)?;

    let mut placed_in_file_menu = false;
    for item in menu.items()? {
        if let MenuItemKind::Submenu(submenu) = item {
            if submenu.text().map(|text| text == "File").unwrap_or(false) {
                submenu.insert(&new_window_item, 0)?;
                submenu.insert(&new_tab_item, 1)?;
                placed_in_file_menu = true;
                break;
            }
        }
    }

    if !placed_in_file_menu {
        let file_menu = SubmenuBuilder::new(app, "파일")
            .item(&new_window_item)
            .item(&new_tab_item)
            .build()?;
        menu.append(&file_menu)?;
    }

    let merge_all_item = MenuItemBuilder::new("모든 창 합치기")
        .id("merge_all_windows")
        .accelerator("Ctrl+Cmd+M")
        .build(app)?;
    let toggle_tab_bar_item = MenuItemBuilder::new("탭 바 표시/숨기기")
        .id("toggle_tab_bar")
        .accelerator("Shift+Cmd+\\")
        .build(app)?;

    let window_menu = SubmenuBuilder::new(app, "창")
        .item(&merge_all_item)
        .item(&toggle_tab_bar_item)
        .build()?;

    menu.append(&window_menu)?;
    app.set_menu(menu)?;
    Ok(())
}

pub fn handle_window_menu_event(app: &AppHandle, id: &str) {
    match id {
        "new_window" => {
            if let Err(error) = native_window_manager::open_editor_window(app) {
                eprintln!("[window] failed to create editor window: {error}");
            }
        }
        "new_tab" => native_window_manager::open_editor_tab(app),
        "merge_all_windows" => native_window_manager::merge_all_windows(app),
        "toggle_tab_bar" => native_window_manager::toggle_tab_bar(app),
        _ => {}
    }
}
