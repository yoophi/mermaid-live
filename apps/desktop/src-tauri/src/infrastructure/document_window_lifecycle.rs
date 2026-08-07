use std::collections::HashSet;
use std::sync::Mutex;

use tauri::{Emitter, Manager, Window, WindowEvent};

pub const CLOSE_DOCUMENT_REQUEST_EVENT: &str = "close-chart-document-request";

#[derive(Default)]
pub struct CloseAuthorizations {
    window_labels: Mutex<HashSet<String>>,
}

impl CloseAuthorizations {
    pub fn authorize(&self, window_label: &str) {
        self.with_labels(|labels| {
            labels.insert(window_label.to_owned());
        });
    }

    pub fn consume(&self, window_label: &str) -> bool {
        self.with_labels(|labels| labels.remove(window_label))
    }

    fn with_labels<T>(&self, operation: impl FnOnce(&mut HashSet<String>) -> T) -> T {
        let mut labels = self
            .window_labels
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        operation(&mut labels)
    }
}

pub fn handle_window_event(window: &Window, event: &WindowEvent) {
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    let authorizations = window.state::<CloseAuthorizations>();
    if authorizations.consume(window.label()) {
        return;
    }

    api.prevent_close();
    if let Err(error) = window.emit(CLOSE_DOCUMENT_REQUEST_EVENT, ()) {
        eprintln!("[document] failed to request protected close: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::CloseAuthorizations;

    #[test]
    fn authorization_is_window_scoped_and_consumed_once() {
        let authorizations = CloseAuthorizations::default();
        authorizations.authorize("main");

        assert!(!authorizations.consume("other"));
        assert!(authorizations.consume("main"));
        assert!(!authorizations.consume("main"));
    }
}
