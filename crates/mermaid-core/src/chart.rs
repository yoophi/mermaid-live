const MERMAID_START_TOKENS: &[&str] = &[
    "graph",
    "flowchart",
    "sequenceDiagram",
    "classDiagram",
    "stateDiagram",
    "stateDiagram-v2",
    "erDiagram",
    "journey",
    "gantt",
    "pie",
    "quadrantChart",
    "requirementDiagram",
    "gitGraph",
    "mindmap",
    "timeline",
    "zenuml",
    "sankey-beta",
    "xychart-beta",
    "block-beta",
    "packet-beta",
    "kanban",
    "architecture-beta",
    "radar-beta",
    "eventModel",
    "treemap-beta",
    "venn",
    "ishikawa",
    "wardley",
    "tree",
    "info",
];

pub fn is_likely_mermaid_chart(text: &str) -> bool {
    let normalized = strip_mermaid_prelude(text);
    let Some(first_token) = normalized.split_whitespace().next() else {
        return false;
    };

    MERMAID_START_TOKENS.contains(&first_token) || first_token.starts_with("C4")
}

pub fn extract_mermaid_chart_source(text: &str) -> Option<String> {
    if let Some(source) = extract_mermaid_fenced_code_block(text) {
        return Some(source);
    }

    if is_likely_mermaid_chart(text) {
        return Some(text.trim().to_string());
    }

    None
}

fn extract_mermaid_fenced_code_block(text: &str) -> Option<String> {
    let mut in_mermaid_block = false;
    let mut block_lines: Vec<&str> = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();

        if in_mermaid_block {
            if trimmed.starts_with("```") {
                let source = block_lines.join("\n");
                if is_likely_mermaid_chart(&source) {
                    return Some(source.trim().to_string());
                }
                return None;
            }

            block_lines.push(line);
            continue;
        }

        let Some(info) = trimmed.strip_prefix("```") else {
            continue;
        };

        let language = info.split_whitespace().next().unwrap_or_default();
        if language.eq_ignore_ascii_case("mermaid") {
            in_mermaid_block = true;
            block_lines.clear();
        }
    }

    None
}

fn strip_mermaid_prelude(text: &str) -> &str {
    let mut remaining = text.trim_start();

    if let Some(stripped) = strip_frontmatter(remaining) {
        remaining = stripped.trim_start();
    }

    loop {
        let next = remaining.trim_start();

        if let Some(stripped) = strip_directive(next) {
            remaining = stripped;
            continue;
        }

        if let Some(stripped) = strip_comment(next) {
            remaining = stripped;
            continue;
        }

        return next;
    }
}

fn strip_frontmatter(text: &str) -> Option<&str> {
    let rest = text
        .strip_prefix("---\r\n")
        .or_else(|| text.strip_prefix("---\n"))?;

    let mut offset = text.len() - rest.len();
    for line in rest.split_inclusive('\n') {
        let line_body = line.trim_end_matches(['\r', '\n']);
        offset += line.len();
        if line_body == "---" {
            return Some(&text[offset..]);
        }
    }

    None
}

fn strip_directive(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("%%{")?;
    let end = rest.find("}%%")?;
    Some(&rest[end + 3..])
}

fn strip_comment(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("%%")?;
    let newline = rest.find('\n')?;
    Some(&rest[newline + 1..])
}

#[cfg(test)]
mod tests {
    use super::{extract_mermaid_chart_source, is_likely_mermaid_chart};

    #[test]
    fn detects_standard_mermaid_start_tokens() {
        assert!(is_likely_mermaid_chart("flowchart TD\nA --> B"));
        assert!(is_likely_mermaid_chart("sequenceDiagram\nAlice->>Bob: hi"));
        assert!(is_likely_mermaid_chart("C4Context\nPerson(user, User)"));
    }

    #[test]
    fn ignores_keywords_inside_regular_text() {
        assert!(!is_likely_mermaid_chart(
            "this text mentions graph but is not a diagram"
        ));
    }

    #[test]
    fn strips_supported_mermaid_prelude() {
        let source = "---\ntitle: Demo\n---\n%%{init: {}}%%\n%% comment\nflowchart LR\nA --> B";
        assert!(is_likely_mermaid_chart(source));
    }

    #[test]
    fn extracts_mermaid_fenced_code_block_from_markdown() {
        let markdown = r#"Some markdown

```mermaid
flowchart LR
  A --> B
```
"#;

        assert_eq!(
            extract_mermaid_chart_source(markdown),
            Some("flowchart LR\n  A --> B".to_string())
        );
    }

    #[test]
    fn ignores_non_mermaid_fenced_code_blocks() {
        let markdown = r#"```ts
const graph = "not a diagram";
```"#;

        assert_eq!(extract_mermaid_chart_source(markdown), None);
    }
}
