use std::io::Read;
use std::process::Command;

/// Matches the clipboard path's limit in the desktop app, so text the app would
/// refuse to scan never leaves the CLI either.
pub const MAX_INPUT_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputSource {
    Clipboard,
    Stdin,
    File(String),
}

pub fn resolve_input_source(positionals: &[String]) -> InputSource {
    match positionals.first().map(String::as_str) {
        None => InputSource::Clipboard,
        Some("-") => InputSource::Stdin,
        Some(path) => InputSource::File(path.to_string()),
    }
}

/// Returns the text, or `None` when the source could not be read at all.
pub fn read_input(source: &InputSource) -> Option<String> {
    match source {
        InputSource::Clipboard => {
            let output = Command::new("pbpaste").output().ok()?;
            if !output.status.success() {
                return None;
            }
            String::from_utf8(output.stdout).ok()
        }
        InputSource::Stdin => {
            let mut text = String::new();
            std::io::stdin().read_to_string(&mut text).ok()?;
            Some(text)
        }
        InputSource::File(path) => std::fs::read_to_string(path).ok(),
    }
}

pub fn is_blank(text: &str) -> bool {
    text.trim().is_empty()
}

pub fn exceeds_limit(text: &str) -> bool {
    text.len() > MAX_INPUT_BYTES
}

#[cfg(test)]
mod tests {
    use super::{exceeds_limit, is_blank, resolve_input_source, InputSource, MAX_INPUT_BYTES};

    fn positionals(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| arg.to_string()).collect()
    }

    #[test]
    fn reads_the_clipboard_when_given_nothing() {
        assert_eq!(resolve_input_source(&[]), InputSource::Clipboard);
    }

    #[test]
    fn a_lone_dash_means_stdin() {
        assert_eq!(resolve_input_source(&positionals(&["-"])), InputSource::Stdin);
    }

    #[test]
    fn a_path_means_a_file() {
        assert_eq!(
            resolve_input_source(&positionals(&["/tmp/a.mmd"])),
            InputSource::File("/tmp/a.mmd".to_string())
        );
    }

    #[test]
    fn detects_blank_and_oversized_input() {
        assert!(is_blank("   \n\t "));
        assert!(!is_blank("flowchart LR"));
        assert!(exceeds_limit(&"x".repeat(MAX_INPUT_BYTES + 1)));
        assert!(!exceeds_limit("flowchart LR"));
    }
}
