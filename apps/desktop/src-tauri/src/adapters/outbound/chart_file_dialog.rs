use std::path::PathBuf;

use tauri::{Runtime, Window};
use tauri_plugin_dialog::{
    DialogExt, MessageDialogButtons, MessageDialogKind, MessageDialogResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsavedChangesDecision {
    Save,
    Discard,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalConflictDecision {
    Reload,
    Overwrite,
    Cancel,
}

pub fn pick_diagram_file<R: Runtime>(window: &Window<R>) -> Result<Option<PathBuf>, String> {
    window
        .dialog()
        .file()
        .set_parent(window)
        .set_title("Mermaid 차트 열기")
        .add_filter("Mermaid chart", &["mmd", "mermaid"])
        .blocking_pick_file()
        .map(|path| path.into_path().map_err(|error| error.to_string()))
        .transpose()
}

pub fn pick_save_path<R: Runtime>(
    window: &Window<R>,
    suggested_file_name: &str,
) -> Result<Option<PathBuf>, String> {
    window
        .dialog()
        .file()
        .set_parent(window)
        .set_title("Mermaid 차트 저장")
        .set_file_name(suggested_file_name)
        .add_filter("Mermaid chart", &["mmd", "mermaid"])
        .blocking_save_file()
        .map(|path| path.into_path().map_err(|error| error.to_string()))
        .transpose()
}

pub fn prompt_unsaved_changes<R: Runtime>(
    window: &Window<R>,
    file_name: &str,
) -> UnsavedChangesDecision {
    let result = window
        .dialog()
        .message(format!("{file_name}의 변경사항을 저장하시겠습니까?"))
        .title("저장하지 않은 변경사항")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::YesNoCancelCustom(
            "저장".into(),
            "저장 안 함".into(),
            "취소".into(),
        ))
        .blocking_show_with_result();
    map_unsaved_result(result)
}

pub fn prompt_external_conflict<R: Runtime>(
    window: &Window<R>,
    file_name: &str,
) -> ExternalConflictDecision {
    let result = window
        .dialog()
        .message(format!(
            "{file_name}이(가) 다른 프로그램에서 변경되었습니다."
        ))
        .title("외부 변경 충돌")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::YesNoCancelCustom(
            "다시 불러오기".into(),
            "덮어쓰기".into(),
            "취소".into(),
        ))
        .blocking_show_with_result();
    map_conflict_result(result)
}

pub fn show_error<R: Runtime>(window: &Window<R>, title: &str, message: &str) {
    window
        .dialog()
        .message(message)
        .title(title)
        .kind(MessageDialogKind::Error)
        .buttons(MessageDialogButtons::Ok)
        .blocking_show();
}

fn map_unsaved_result(result: MessageDialogResult) -> UnsavedChangesDecision {
    match result {
        MessageDialogResult::Yes => UnsavedChangesDecision::Save,
        MessageDialogResult::No => UnsavedChangesDecision::Discard,
        MessageDialogResult::Custom(value) if value == "저장" => UnsavedChangesDecision::Save,
        MessageDialogResult::Custom(value) if value == "저장 안 함" => {
            UnsavedChangesDecision::Discard
        }
        _ => UnsavedChangesDecision::Cancel,
    }
}

fn map_conflict_result(result: MessageDialogResult) -> ExternalConflictDecision {
    match result {
        MessageDialogResult::Yes => ExternalConflictDecision::Reload,
        MessageDialogResult::No => ExternalConflictDecision::Overwrite,
        MessageDialogResult::Custom(value) if value == "다시 불러오기" => {
            ExternalConflictDecision::Reload
        }
        MessageDialogResult::Custom(value) if value == "덮어쓰기" => {
            ExternalConflictDecision::Overwrite
        }
        _ => ExternalConflictDecision::Cancel,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        map_conflict_result, map_unsaved_result, ExternalConflictDecision, UnsavedChangesDecision,
    };
    use tauri_plugin_dialog::MessageDialogResult;

    #[test]
    fn maps_custom_unsaved_buttons() {
        assert_eq!(
            map_unsaved_result(MessageDialogResult::Custom("저장".into())),
            UnsavedChangesDecision::Save
        );
        assert_eq!(
            map_unsaved_result(MessageDialogResult::Custom("저장 안 함".into())),
            UnsavedChangesDecision::Discard
        );
        assert_eq!(
            map_unsaved_result(MessageDialogResult::Cancel),
            UnsavedChangesDecision::Cancel
        );
    }

    #[test]
    fn maps_custom_conflict_buttons() {
        assert_eq!(
            map_conflict_result(MessageDialogResult::Custom("다시 불러오기".into())),
            ExternalConflictDecision::Reload
        );
        assert_eq!(
            map_conflict_result(MessageDialogResult::Custom("덮어쓰기".into())),
            ExternalConflictDecision::Overwrite
        );
        assert_eq!(
            map_conflict_result(MessageDialogResult::Cancel),
            ExternalConflictDecision::Cancel
        );
    }
}
