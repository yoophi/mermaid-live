/// Rendering above the cell grid keeps the chart crisp on retina displays.
pub const DEFAULT_SCALE: f64 = 2.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    pub help: bool,
    pub probe: bool,
    pub force: bool,
    /// Stay connected and show every chart the app detects on the clipboard.
    pub watch: bool,
    /// In watch mode, replace the screen instead of appending.
    pub clear: bool,
    pub scale: f64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            help: false,
            probe: false,
            force: false,
            watch: false,
            clear: false,
            scale: DEFAULT_SCALE,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedArguments {
    pub options: Options,
    /// Everything that is not a flag or a flag's value, in order.
    pub positionals: Vec<String>,
}

fn parse_scale(value: Option<&str>, fallback: f64) -> f64 {
    match value.and_then(|value| value.parse::<f64>().ok()) {
        Some(parsed) if parsed.is_finite() && parsed > 0.0 => parsed,
        _ => fallback,
    }
}

/// Splits flags from positional arguments so input resolution never has to know
/// which flags carry a value.
pub fn parse_arguments<I, S>(argv: I) -> ParsedArguments
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let argv: Vec<String> = argv.into_iter().map(|arg| arg.as_ref().to_string()).collect();
    let mut options = Options::default();
    let mut positionals = Vec::new();
    let mut index = 0;

    while index < argv.len() {
        let arg = argv[index].as_str();

        match arg {
            "-h" | "--help" => options.help = true,
            "--probe" => options.probe = true,
            "--force" => options.force = true,
            "--watch" => options.watch = true,
            "--clear" => options.clear = true,
            "--scale" => {
                options.scale = parse_scale(argv.get(index + 1).map(String::as_str), options.scale);
                index += 1;
            }
            // A lone dash is the conventional marker for stdin, not a flag.
            "-" => positionals.push(arg.to_string()),
            _ if arg.starts_with("--scale=") => {
                options.scale = parse_scale(arg.strip_prefix("--scale="), options.scale);
            }
            _ if arg.starts_with('-') => {}
            _ => positionals.push(arg.to_string()),
        }

        index += 1;
    }

    ParsedArguments {
        options,
        positionals,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_arguments, DEFAULT_SCALE};

    #[test]
    fn recognises_flags() {
        let parsed = parse_arguments(["--probe", "--force", "-h"]);

        assert!(parsed.options.probe);
        assert!(parsed.options.force);
        assert!(parsed.options.help);
        assert!(parsed.positionals.is_empty());
    }

    #[test]
    fn recognises_watch_flags() {
        let parsed = parse_arguments(["--watch", "--clear"]);

        assert!(parsed.options.watch);
        assert!(parsed.options.clear);
        assert!(parsed.positionals.is_empty());
    }

    #[test]
    fn parses_scale_in_both_spellings() {
        assert_eq!(parse_arguments(["--scale", "3"]).options.scale, 3.0);
        assert_eq!(parse_arguments(["--scale=1.5"]).options.scale, 1.5);
    }

    #[test]
    fn falls_back_on_an_unusable_scale() {
        assert_eq!(parse_arguments(["--scale", "abc"]).options.scale, DEFAULT_SCALE);
        assert_eq!(parse_arguments(["--scale", "-1"]).options.scale, DEFAULT_SCALE);
        assert_eq!(parse_arguments(["--scale"]).options.scale, DEFAULT_SCALE);
    }

    #[test]
    fn does_not_mistake_a_flags_value_for_a_path() {
        assert!(parse_arguments(["--scale", "2"]).positionals.is_empty());
        assert!(parse_arguments(["--scale=3"]).positionals.is_empty());
    }

    #[test]
    fn keeps_a_lone_dash_as_a_positional() {
        assert_eq!(parse_arguments(["-"]).positionals, vec!["-".to_string()]);
        assert_eq!(
            parse_arguments(["--force", "-"]).positionals,
            vec!["-".to_string()]
        );
    }

    #[test]
    fn keeps_paths_as_positionals() {
        assert_eq!(
            parse_arguments(["--force", "/tmp/a.mmd"]).positionals,
            vec!["/tmp/a.mmd".to_string()]
        );
    }
}
