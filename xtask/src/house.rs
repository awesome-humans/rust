//! House opinions, which `validate-sources` and `build-site` both read. A house opinion
//! belongs to a house profile, and sits beside the sourced text it changes, never inside it
//! (awesome-humans/website: House opinions). In an opinion it's a block:
//!
//! ```text
//! /// house | simon-mckenna
//!
//! **House:** Treat warnings as errors on every machine, not only in CI. …
//!
//! ///
//! ```
//!
//! In a template it's a region between two comment lines, in the file's comment syntax:
//!
//! ```text
//! # house:simon-mckenna begin
//! …
//! # house:simon-mckenna end
//! ```
//!
//! The file's sources list names the profile as `house:<profile>`. Removing every block and
//! region gives the default copy, which carries no house opinions, and keeping one profile's
//! gives that profile's copy.

use std::sync::LazyLock;

use regex::Regex;

/// The prefix of a house profile's id in a sources list: `house:simon-mckenna`.
pub const PREFIX: &str = "house:";

/// The fixed string every house opinion opens with, so grep can find it.
pub const MARKING: &str = "**House:**";

static PROFILE_ID: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\A[a-z0-9]+(?:-[a-z0-9]+)*\z").expect("valid pattern"));
static BLOCK_OPENING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\A/// house \|(?<profile>.*)\z").expect("valid pattern"));
static REGION_MARKER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\A\s*(?:#|//)\s*house:(?<profile>\S+) (?<edge>begin|end)\s*\z")
        .expect("valid pattern")
});
static FENCE_OPENING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\A\s*(?<fence>`{3,}|~{3,})").expect("valid pattern"));

/// One house block or region: its profile, its first and last line (the opening and closing
/// lines included, counted from 0), and whether it carries the marking.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub profile: String,
    pub start: usize,
    pub end: usize,
    pub marked: bool,
}

/// The profile a sources id names, or `None` for any other id.
pub fn profile_of(source_id: &str) -> Option<&str> {
    source_id.strip_prefix(PREFIX)
}

/// Whether a profile id is lowercase words joined by hyphens, as a roster id is. `none` is
/// reserved for the default, which the site's `?house=none` and the `raw/none/` copies name.
pub fn is_valid_profile(profile: &str) -> bool {
    PROFILE_ID.is_match(profile) && profile != "none"
}

/// Whether a path is a template, whose house opinions are regions rather than blocks.
pub fn is_template(path: &str) -> bool {
    !crate::repo::has_extension(path, "md")
}

/// The house blocks in an opinion, or the house regions in a template, in file order. A
/// malformed opening or closing line goes to `errors`, prefixed with the path and line.
pub fn spans(path: &str, text: &str, errors: &mut Vec<String>) -> Vec<Span> {
    let template = is_template(path);
    let lines: Vec<&str> = text.split('\n').collect();
    let mut spans = Vec::new();
    let mut open: Option<String> = None;
    let mut start = 0;
    let mut fence: Option<String> = None;

    for (index, line) in lines.iter().enumerate() {
        // A fenced code block in an opinion can hold a /// line of its own, such as a Rust
        // documentation comment, so markers only count outside one. Templates have no fences.
        if !template && let Some(opening) = FENCE_OPENING.captures(line) {
            let marker = &opening["fence"];
            match &fence {
                None => fence = Some(marker.to_string()),
                Some(current)
                    if marker.as_bytes()[0] == current.as_bytes()[0]
                        && marker.len() >= current.len()
                        && line.trim() == marker =>
                {
                    fence = None;
                }
                Some(_) => {}
            }
            continue;
        }

        if fence.is_some() {
            continue;
        }

        let edge = if template {
            region_edge(line)
        } else {
            block_edge(line, open.is_some())
        };
        let Some((profile, begin)) = edge else {
            continue;
        };

        let number = index + 1;
        if begin {
            if let Some(current) = &open {
                errors.push(format!(
                    "{path}:{number}: house:{profile} opens inside house:{current}, which hasn't closed"
                ));
                continue;
            }

            if !is_valid_profile(&profile) {
                errors.push(format!(
                    "{path}:{number}: '{profile}' isn't a profile id: use lowercase words joined by hyphens, other than none"
                ));
            }

            open = Some(profile);
            start = index;
        } else if let Some(current) = open.take() {
            if template && profile != current {
                errors.push(format!(
                    "{path}:{number}: house:{profile} ends, but the open region is house:{current}"
                ));
                open = Some(current);
            } else {
                let marked = lines[start..index]
                    .iter()
                    .any(|content| content.contains(MARKING));
                spans.push(Span {
                    profile: current,
                    start,
                    end: index,
                    marked,
                });
            }
        } else {
            let kind = if template { "region" } else { "block" };
            errors.push(format!(
                "{path}:{number}: a house {kind} closes, but none is open"
            ));
        }
    }

    if let Some(current) = open {
        errors.push(format!(
            "{path}:{}: house:{current} never closes",
            start + 1
        ));
    }

    spans
}

/// The text with every house block or region removed except the given profile's, or all of
/// them for `None`. A kept template region loses its two marker lines, which only staging
/// reads. The sources list loses every other profile's id, so a copy names only the house
/// opinions it carries.
pub fn for_profile(path: &str, text: &str, profile: Option<&str>) -> String {
    let template = is_template(path);
    let mut lines: Vec<String> = text.split('\n').map(String::from).collect();
    let spans = spans(path, text, &mut Vec::new());

    // From the end, so earlier line numbers stay valid.
    for span in spans.iter().rev() {
        if Some(span.profile.as_str()) == profile {
            if template {
                lines.remove(span.end);
                lines.remove(span.start);
            }
            continue;
        }

        // A removed block takes the blank line after it, so no double blank is left behind.
        let mut end = span.end + 1;
        if !template && lines.get(end).is_some_and(String::is_empty) {
            end += 1;
        }

        lines.drain(span.start..end);
    }

    let mut removed: Vec<String> = spans
        .iter()
        .map(|span| span.profile.as_str())
        .filter(|other| Some(*other) != profile)
        .map(|other| format!("{PREFIX}{other}"))
        .collect();
    removed.sort();
    removed.dedup();

    if !removed.is_empty() {
        if template {
            if let Some(header) = lines.first_mut() {
                *header = without_ids_in_header(header, &removed);
            }
        } else {
            without_ids_in_frontmatter(&mut lines, &removed);
        }
    }

    lines.join("\n")
}

// An opinion's block opens with '/// house | <profile>' and closes with a bare '///'.
fn block_edge(line: &str, open: bool) -> Option<(String, bool)> {
    if let Some(opening) = BLOCK_OPENING.captures(line) {
        return Some((opening["profile"].trim().to_string(), true));
    }

    (open && line.trim_end() == "///").then(|| (String::new(), false))
}

// A template's region opens and closes with a comment line naming the profile.
fn region_edge(line: &str) -> Option<(String, bool)> {
    REGION_MARKER
        .captures(line)
        .map(|marker| (marker["profile"].to_string(), &marker["edge"] == "begin"))
}

// The comma-separated ids in a list, without the removed ones.
fn without_ids(list: &str, removed: &[String]) -> String {
    list.split(',')
        .map(str::trim)
        .filter(|id| !id.is_empty() && !removed.iter().any(|other| other == id))
        .collect::<Vec<_>>()
        .join(", ")
}

// The header's `sources:` segment runs to the next '|', or to the end of the line.
fn without_ids_in_header(header: &str, removed: &[String]) -> String {
    let Some(at) = header.find("sources:") else {
        return header.to_string();
    };

    let value_start = at + "sources:".len();
    let value_end = header[value_start..]
        .find('|')
        .map_or(header.len(), |offset| value_start + offset);
    let ids = without_ids(&header[value_start..value_end], removed);
    let tail = &header[value_end..];
    let value = match (ids.is_empty(), tail.is_empty()) {
        (true, true) => String::new(),
        (true, false) => " ".to_string(),
        (false, true) => format!(" {ids}"),
        (false, false) => format!(" {ids} "),
    };
    format!("{}sources:{value}{tail}", &header[..at])
}

// The frontmatter's `sources:` is a flow list, `[a, house:b]`, or a block list of `- id` lines.
fn without_ids_in_frontmatter(lines: &mut Vec<String>, removed: &[String]) {
    if lines.first().map(String::as_str) != Some("---") {
        return;
    }

    let Some(close) = lines
        .iter()
        .skip(1)
        .position(|line| line.starts_with("---"))
    else {
        return;
    };

    let mut index = 1;
    let mut end = close + 1;
    let mut in_sources = false;
    while index < end {
        let line = &lines[index];
        if let Some(value) = line.strip_prefix("sources:") {
            in_sources = true;
            if let (Some(open), Some(close)) = (value.find('['), value.find(']')) {
                let ids = without_ids(&value[open + 1..close], removed);
                lines[index] = format!("sources:{}[{ids}]{}", &value[..open], &value[close + 1..]);
            }
        } else if in_sources && line.trim_start().starts_with('-') {
            let id = line.trim_start()[1..].trim();
            if removed.iter().any(|other| other == id) {
                lines.remove(index);
                end -= 1;
                continue;
            }
        } else {
            in_sources = false;
        }

        index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPINION: &str = "---\nsources: [house:alice, house:bob, rust-docs]\n---\n# Title\n\nSourced.\n\n/// house | alice\n\n**House:** Alice's.\n\n///\n\n/// house | bob\n\n**House:** Bob's.\n\n///\n\nMore sourced.\n";

    #[test]
    fn spans_finds_each_block() {
        let mut errors = Vec::new();
        let found = spans("opinions/x.md", OPINION, &mut errors);
        assert!(errors.is_empty(), "{errors:?}");
        let profiles: Vec<&str> = found.iter().map(|span| span.profile.as_str()).collect();
        assert_eq!(profiles, ["alice", "bob"]);
        assert!(found.iter().all(|span| span.marked));
    }

    #[test]
    fn spans_ignores_a_doc_comment_inside_a_fence() {
        let text = "# Title\n\n```rust\n///\n/// house | nobody\n```\n";
        let mut errors = Vec::new();
        assert_eq!(spans("opinions/x.md", text, &mut errors), []);
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn spans_reports_a_block_that_never_closes() {
        let mut errors = Vec::new();
        spans(
            "opinions/x.md",
            "/// house | alice\n\n**House:** x\n",
            &mut errors,
        );
        assert_eq!(errors, ["opinions/x.md:1: house:alice never closes"]);
    }

    #[test]
    fn spans_reports_an_invalid_profile() {
        let mut errors = Vec::new();
        spans(
            "opinions/x.md",
            "/// house | none\n**House:** x\n///\n",
            &mut errors,
        );
        assert_eq!(
            errors,
            [
                "opinions/x.md:1: 'none' isn't a profile id: use lowercase words joined by hyphens, other than none"
            ]
        );
    }

    #[test]
    fn spans_reports_a_mismatched_region_end() {
        let text = "# header\n# house:alice begin\n# **House:** x\n# house:bob end\n";
        let mut errors = Vec::new();
        spans("templates/Cargo.toml", text, &mut errors);
        assert_eq!(
            errors,
            [
                "templates/Cargo.toml:4: house:bob ends, but the open region is house:alice",
                "templates/Cargo.toml:2: house:alice never closes",
            ]
        );
    }

    #[test]
    fn for_profile_without_one_removes_every_block_and_id() {
        assert_eq!(
            for_profile("opinions/x.md", OPINION, None),
            "---\nsources: [rust-docs]\n---\n# Title\n\nSourced.\n\nMore sourced.\n"
        );
    }

    #[test]
    fn for_profile_keeps_only_that_profiles_block() {
        assert_eq!(
            for_profile("opinions/x.md", OPINION, Some("bob")),
            "---\nsources: [house:bob, rust-docs]\n---\n# Title\n\nSourced.\n\n/// house | bob\n\n**House:** Bob's.\n\n///\n\nMore sourced.\n"
        );
    }

    #[test]
    fn for_profile_drops_block_list_ids() {
        let text = "---\nsources:\n  - house:alice\n  - rust-docs\n---\n/// house | alice\n**House:** x\n///\n\nSourced.\n";
        assert_eq!(
            for_profile("opinions/x.md", text, None),
            "---\nsources:\n  - rust-docs\n---\nSourced.\n"
        );
    }

    #[test]
    fn for_profile_keeps_a_template_regions_content_without_its_markers() {
        let text = "# awesome-humans/rust template | sources: house:alice, rust-docs\n[lints]\n# house:alice begin\n# **House:** deny warnings\nwarnings = \"deny\"\n# house:alice end\n";
        assert_eq!(
            for_profile("templates/Cargo.toml", text, Some("alice")),
            "# awesome-humans/rust template | sources: house:alice, rust-docs\n[lints]\n# **House:** deny warnings\nwarnings = \"deny\"\n"
        );
        assert_eq!(
            for_profile("templates/Cargo.toml", text, None),
            "# awesome-humans/rust template | sources: rust-docs\n[lints]\n"
        );
    }

    #[test]
    fn for_profile_empties_a_header_that_named_only_the_profile() {
        let text = "// awesome-humans/rust template | sources: house:alice\n// house:alice begin\n// **House:** x\n// house:alice end\n";
        assert_eq!(
            for_profile("templates/lib.rs", text, None),
            "// awesome-humans/rust template | sources:\n"
        );
    }
}
