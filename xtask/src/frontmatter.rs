//! The YAML frontmatter on `opinions/` and `research/` files. `validate-metadata` reports
//! the error after the file's path, and `validate-sources` skips a file that has one, so the
//! two read the same wording from this one parser.

use serde_yaml_ng::{Mapping, Value};

/// Parses a Markdown file's frontmatter, or says why it can't, in the phrase the metadata
/// check prints after the file's path.
pub fn parse(text: &str) -> Result<Mapping, String> {
    let Some(rest) = text.strip_prefix("---\n") else {
        return Err("missing frontmatter block".into());
    };

    let Some(end) = rest.find("\n---") else {
        return Err("unterminated frontmatter block".into());
    };

    match serde_yaml_ng::from_str::<Value>(&rest[..end]) {
        Ok(Value::Mapping(mapping)) => Ok(mapping),
        Ok(_) => Err("frontmatter is not a YAML mapping".into()),
        Err(error) => {
            // The parser counts lines from the frontmatter body, which starts on line 2 of
            // the file, so offset them to point at the real line.
            let location = error.location().map_or_else(String::new, |location| {
                format!(
                    " (line {}, column {})",
                    location.line() + 1,
                    location.column()
                )
            });
            let message = error.to_string();
            let message = message.split(" at line ").next().unwrap_or(&message);
            Err(format!(
                "frontmatter is not valid YAML: {message}{location}"
            ))
        }
    }
}

/// A scalar's text, as a header would carry it, or `None` for a list, a mapping, or null.
pub fn scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

/// A value as a list of strings: a YAML list entry by entry, or a header's comma-separated
/// text. `None` for anything else, which the required-field and roster checks already report.
pub fn list(value: &Value) -> Option<Vec<String>> {
    match value {
        Value::Sequence(items) => Some(
            items
                .iter()
                .map(|item| scalar(item).unwrap_or_default())
                .collect(),
        ),
        Value::String(text) => Some(
            text.split(',')
                .map(str::trim)
                .filter(|id| !id.is_empty())
                .map(String::from)
                .collect(),
        ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_reads_a_mapping() {
        let fields = parse("---\ntargets: [rust-1.99]\nsources: [rust-docs]\n---\n# Title\n")
            .expect("valid frontmatter");
        assert_eq!(
            list(&fields["sources"]),
            Some(vec!["rust-docs".to_string()])
        );
    }

    #[test]
    fn parse_rejects_a_file_without_frontmatter() {
        assert_eq!(
            parse("# Title\n"),
            Err("missing frontmatter block".to_string())
        );
    }

    #[test]
    fn parse_rejects_an_unterminated_block() {
        assert_eq!(
            parse("---\ntargets: [rust-1.99]\n"),
            Err("unterminated frontmatter block".to_string())
        );
    }

    #[test]
    fn parse_points_at_the_file_line_of_a_yaml_error() {
        let error = parse("---\ntargets: [rust-1.99\n---\n").expect_err("invalid YAML");
        assert!(
            error.starts_with("frontmatter is not valid YAML: "),
            "{error}"
        );
        assert!(error.contains("(line "), "{error}");
    }

    #[test]
    fn parse_rejects_a_scalar() {
        assert_eq!(
            parse("---\njust text\n---\n"),
            Err("frontmatter is not a YAML mapping".to_string())
        );
    }

    #[test]
    fn list_splits_header_text() {
        assert_eq!(
            list(&Value::String("matklad, rust-docs".into())),
            Some(vec!["matklad".to_string(), "rust-docs".to_string()])
        );
    }
}
