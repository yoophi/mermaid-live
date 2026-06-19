pub fn read_clipboard_text() -> Result<String, String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|error| format!("clipboard unavailable: {error}"))?;

    clipboard
        .get_text()
        .map_err(|error| format!("clipboard text unavailable: {error}"))
}
