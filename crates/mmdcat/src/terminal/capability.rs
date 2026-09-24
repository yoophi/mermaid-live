use std::time::Duration;

use super::tty::Tty;

const ESC: &str = "\x1b";

/// The protocol's own support probe: a graphics query followed by a primary
/// device attributes request. A graphics reply means support; a lone DA1 reply
/// means the terminal ignored the graphics command.
const GRAPHICS_QUERY: &str = "\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\\x1b[c";
const QUERY_TIMEOUT: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsSupport {
    Supported,
    Unsupported,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Capability {
    pub support: GraphicsSupport,
    /// How the verdict was reached, surfaced by `--probe`.
    pub reason: String,
    pub terminal_name: String,
    pub multiplexer: Option<String>,
}

/// An empty variable is not a signal: some shells export placeholders.
fn env_value(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|value| !value.is_empty())
}

/// Terminals known to implement the kitty graphics protocol.
fn known_terminal() -> Option<&'static str> {
    if env_value("TERM_PROGRAM").as_deref() == Some("ghostty") {
        return Some("Ghostty");
    }
    if env_value("TERM_PROGRAM").as_deref() == Some("WezTerm") {
        return Some("WezTerm");
    }
    if env_value("TERM").as_deref() == Some("xterm-kitty") || env_value("KITTY_WINDOW_ID").is_some()
    {
        return Some("kitty");
    }
    None
}

pub fn terminal_name() -> String {
    known_terminal()
        .map(str::to_string)
        .or_else(|| env_value("TERM_PROGRAM"))
        .or_else(|| env_value("TERM"))
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn detect_multiplexer() -> Option<String> {
    if env_value("TMUX").is_some() {
        return Some("tmux".to_string());
    }
    if env_value("TERM").is_some_and(|term| term.starts_with("screen")) {
        return Some("screen".to_string());
    }
    None
}

fn detect_from_env() -> GraphicsSupport {
    if known_terminal().is_some() {
        GraphicsSupport::Supported
    } else {
        GraphicsSupport::Unknown
    }
}

/// A DA1 reply looks like `ESC [ ? 62 ; ... c`.
fn has_device_attributes_reply(buffer: &str) -> bool {
    let mut rest = buffer;

    while let Some(position) = rest.find(&format!("{ESC}[")) {
        let tail = &rest[position + 2..];
        let terminator = tail.find('c');

        if let Some(terminator) = terminator {
            let body = &tail[..terminator];
            if body
                .chars()
                .all(|character| character.is_ascii_digit() || character == ';' || character == '?')
            {
                return true;
            }
        }

        rest = tail;
    }

    false
}

fn is_query_complete(buffer: &str) -> bool {
    buffer.contains("\x1b\\") || has_device_attributes_reply(buffer)
}

/// Asks the terminal directly when possible, and falls back to recognising the
/// terminal by environment so a redirected or multiplexed session still works.
pub fn detect_capability(tty: Option<&mut Tty>) -> Capability {
    let terminal_name = terminal_name();
    let multiplexer = detect_multiplexer();

    let Some(tty) = tty else {
        let support = detect_from_env();
        let reason = if support == GraphicsSupport::Supported {
            "no controlling terminal, recognised by environment"
        } else {
            "no controlling terminal and not a known supporting terminal"
        };

        return Capability {
            support,
            reason: reason.to_string(),
            terminal_name,
            multiplexer,
        };
    };

    let reply = tty.request(GRAPHICS_QUERY, is_query_complete, QUERY_TIMEOUT);

    if reply.contains("_G") {
        return Capability {
            support: GraphicsSupport::Supported,
            reason: "terminal answered the graphics query".to_string(),
            terminal_name,
            multiplexer,
        };
    }

    if has_device_attributes_reply(&reply) {
        return Capability {
            support: GraphicsSupport::Unsupported,
            reason: "terminal answered device attributes but ignored the graphics query".to_string(),
            terminal_name,
            multiplexer,
        };
    }

    let support = detect_from_env();
    let reason = if support == GraphicsSupport::Supported {
        "query timed out, recognised by environment"
    } else {
        "query timed out and not a known supporting terminal"
    };

    Capability {
        support,
        reason: reason.to_string(),
        terminal_name,
        multiplexer,
    }
}

#[cfg(test)]
mod tests {
    use super::{has_device_attributes_reply, is_query_complete};

    #[test]
    fn recognises_a_device_attributes_reply() {
        assert!(has_device_attributes_reply("\x1b[?62;4c"));
        assert!(has_device_attributes_reply("\x1b[?1;2c"));
        assert!(!has_device_attributes_reply("\x1b[4;800;600t"));
        assert!(!has_device_attributes_reply(""));
    }

    #[test]
    fn a_graphics_reply_completes_the_query() {
        assert!(is_query_complete("\x1b_Gi=31;OK\x1b\\"));
        assert!(is_query_complete("\x1b[?62;4c"));
        assert!(!is_query_complete("\x1b_Gi=31;OK"));
    }
}
