//! The one-line comment header on `templates/` files. A TOML or Rust file can't open with a
//! `---` block and stay valid for the tools that read it, so the metadata goes in a comment on
//! the first line instead (AGENTS.md: Metadata):
//!
//! ```text
//! # awesome-humans/rust template | targets: rust-1.99, edition-2024 | last-reviewed: … | …
//! // awesome-humans/rust template | targets: rust-1.99, edition-2024 | last-reviewed: … | …
//! ```
//!
//! The header parses to the same mapping as frontmatter, with text values, so one
//! required-field check covers both syntaxes and reports the same wording.

use serde_yaml_ng::{Mapping, Value};

/// The literal every header opens with, inside its comment marker.
pub const MARKER: &str = "awesome-humans/rust template";

/// Parses a template's first line, or says why it carries no valid header.
pub fn parse(text: &str) -> Result<Mapping, String> {
    let line = text
        .strip_prefix('\u{feff}')
        .unwrap_or(text)
        .lines()
        .next()
        .unwrap_or("")
        .trim();

    // `//` first, because a Rust file's header opens with it, and nothing else does.
    let Some(body) = line
        .strip_prefix("//")
        .or_else(|| line.strip_prefix('#'))
        .map(str::trim)
    else {
        return Err("first line carries no comment header".into());
    };

    let Some(segments) = body.strip_prefix(MARKER) else {
        return Err(format!("header does not open with '{MARKER}'"));
    };

    let mut fields = Mapping::new();
    for segment in segments
        .split('|')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
    {
        // Split on the first colon only: `targets: rust-1.99, edition-2024` is one field
        // whose value happens to be a list.
        let Some((key, value)) = segment.split_once(':') else {
            return Err(format!("header segment '{segment}' is not 'key: value'"));
        };

        let key = key.trim();
        if fields.contains_key(key) {
            return Err(format!("header repeats the field '{key}'"));
        }

        fields.insert(key.into(), Value::String(value.trim().into()));
    }

    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "awesome-humans/rust template | targets: rust-1.99, edition-2024 | last-reviewed: 2026-10-14 | sources: rust-docs";

    #[test]
    fn parse_reads_a_toml_header() {
        let fields = parse(&format!("# {HEADER}\n[workspace]\n")).expect("valid header");
        assert_eq!(
            fields["targets"],
            Value::String("rust-1.99, edition-2024".into())
        );
        assert_eq!(fields["sources"], Value::String("rust-docs".into()));
    }

    #[test]
    fn parse_reads_a_rust_header_after_a_byte_order_mark() {
        assert!(parse(&format!("\u{feff}// {HEADER}\r\nfn main() {{}}\n")).is_ok());
    }

    #[test]
    fn parse_rejects_a_first_line_without_a_comment() {
        assert_eq!(
            parse("[workspace]\n"),
            Err("first line carries no comment header".to_string())
        );
    }

    #[test]
    fn parse_rejects_another_marker() {
        assert_eq!(
            parse("# awesome-humans/dotnet template | targets: net10.0\n"),
            Err(format!("header does not open with '{MARKER}'"))
        );
    }

    #[test]
    fn parse_rejects_a_segment_without_a_colon() {
        assert_eq!(
            parse(&format!("# {MARKER} | targets\n")),
            Err("header segment 'targets' is not 'key: value'".to_string())
        );
    }

    #[test]
    fn parse_rejects_a_repeated_field() {
        assert_eq!(
            parse(&format!("# {MARKER} | sources: a | sources: b\n")),
            Err("header repeats the field 'sources'".to_string())
        );
    }
}
