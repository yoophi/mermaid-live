const MAX_LABEL_LENGTH: usize = 40;
const FALLBACK_LABEL: &str = "Mermaid chart";

/// Opaque revision of the system clipboard. Two equal revisions mean the
/// clipboard has not changed between the reads that produced them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClipboardRevision(u64);

impl ClipboardRevision {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
}

/// A Mermaid chart found on the clipboard, together with the identity and
/// display name used to track and list it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectedChart {
    source: String,
    fingerprint: String,
    label: String,
}

impl DetectedChart {
    pub fn from_source(source: String) -> Self {
        let fingerprint = chart_fingerprint(&source);
        let label = chart_label(&source);

        Self {
            source,
            fingerprint,
            label,
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    /// Stable identity of the chart contents, used to avoid surfacing the same
    /// chart twice and to name its window and staging file.
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    /// Short human readable name for menus and notifications.
    pub fn label(&self) -> &str {
        &self.label
    }
}

pub fn chart_fingerprint(source: &str) -> String {
    format!("{:x}", md5::compute(source.trim()))
}

/// Uses the first meaningful line, which for Mermaid is the diagram
/// declaration or the frontmatter title.
fn chart_label(source: &str) -> String {
    let line = source
        .lines()
        .map(str::trim)
        .find(|line| is_label_candidate(line))
        .unwrap_or(FALLBACK_LABEL);

    truncate(line, MAX_LABEL_LENGTH)
}

fn is_label_candidate(line: &str) -> bool {
    !line.is_empty() && line != "---" && !line.starts_with("%%")
}

fn truncate(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }

    let mut truncated: String = value.chars().take(limit).collect();
    truncated.push('…');
    truncated
}

#[cfg(test)]
mod tests {
    use super::{chart_fingerprint, DetectedChart, MAX_LABEL_LENGTH};

    #[test]
    fn labels_chart_with_its_declaration_line() {
        let chart = DetectedChart::from_source("flowchart LR\n  A --> B".to_string());

        assert_eq!(chart.label(), "flowchart LR");
    }

    #[test]
    fn skips_prelude_lines_when_labelling() {
        let chart =
            DetectedChart::from_source("%% a comment\n\nsequenceDiagram\nA->>B: hi".to_string());

        assert_eq!(chart.label(), "sequenceDiagram");
    }

    #[test]
    fn truncates_long_labels_on_character_boundaries() {
        let chart = DetectedChart::from_source(format!("flowchart {}", "가".repeat(80)));

        assert_eq!(chart.label().chars().count(), MAX_LABEL_LENGTH + 1);
        assert!(chart.label().ends_with('…'));
    }

    #[test]
    fn fingerprint_ignores_surrounding_whitespace() {
        assert_eq!(
            chart_fingerprint("flowchart LR"),
            chart_fingerprint("\n  flowchart LR\n\n")
        );
    }
}
