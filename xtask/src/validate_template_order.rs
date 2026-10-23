//! `validate-template-order`: the interchangeable entries in each TOML template are sorted by
//! name, ignoring case, within their group. The entries are the keys of a dependency table
//! (`[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`, and their `workspace.` and
//! `target.` forms) and of a lint table (`[lints.*]` and `[workspace.lints.*]`). Reordering
//! them doesn't change what Cargo does, so one order means a reader finds an entry where they
//! expect it, and two edits that add the same one land on the same line.
//!
//! A group is a run of keys between blank lines, and a comment stays with the key below it,
//! so a table can keep related entries together under one comment. Order that carries meaning
//! isn't checked. See awesome-humans/website: Writing style, Order.

use std::sync::LazyLock;

use regex::Regex;

use crate::Report;
use crate::repo::{self, Repo};

const DEPENDENCY_TABLES: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

static TABLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\[([^\[\]]+)\]\s*(#.*)?$").expect("valid pattern"));

static KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^([A-Za-z0-9_-]+|"[^"]+")\s*="#).expect("valid pattern"));

pub fn run(repo: &Repo) -> Report {
    let mut errors = Vec::new();
    let mut groups = 0;

    let templates: Vec<String> = repo
        .template_files()
        .into_iter()
        .filter(|path| repo::has_extension(path, "toml"))
        .collect();

    for path in &templates {
        match repo.read(path) {
            Ok(text) => groups += check(path, &text, &mut errors),
            Err(error) => errors.push(format!("{path}: can't be read: {error}")),
        }
    }

    Report {
        failure: "Template order validation failed:",
        errors,
        summary: format!(
            "All {groups} sortable key groups in {} TOML templates are in order.",
            templates.len()
        ),
    }
}

/// Checks one file, adds a finding for each group out of order, and returns how many groups
/// of two or more keys it checked.
fn check(path: &str, text: &str, errors: &mut Vec<String>) -> usize {
    let mut table: Option<String> = None;
    let mut group: Vec<(usize, String)> = Vec::new();
    let mut checked = 0;
    // The delimiter of the multiline string a line sits inside, if any. Its lines are text,
    // not headers or keys.
    let mut string: Option<&str> = None;

    for (index, line) in text.lines().enumerate() {
        if let Some(delimiter) = string {
            if line.matches(delimiter).count() % 2 == 1 {
                string = None;
            }
            continue;
        }
        // A full-line comment can mention a delimiter without opening a string.
        if line.trim_start().starts_with('#') {
            continue;
        }
        string = ["\"\"\"", "'''"]
            .into_iter()
            .find(|delimiter| line.matches(delimiter).count() % 2 == 1);

        // TOML allows indentation before a header or a key.
        let line = line.trim();
        if let Some(captures) = TABLE.captures(line) {
            checked += finish(path, table.as_deref(), &mut group, errors);
            let name = captures[1].trim().to_owned();
            table = is_sortable(&name).then_some(name);
        } else if line.starts_with("[[") || line.is_empty() {
            checked += finish(path, table.as_deref(), &mut group, errors);
            if line.starts_with("[[") {
                table = None;
            }
        } else if table.is_some()
            && let Some(captures) = KEY.captures(line)
        {
            group.push((index + 1, captures[1].trim_matches('"').to_owned()));
        }
    }

    checked + finish(path, table.as_deref(), &mut group, errors)
}

/// Checks the group just ended, if it belongs to a sortable table, and empties it.
fn finish(
    path: &str,
    table: Option<&str>,
    group: &mut Vec<(usize, String)>,
    errors: &mut Vec<String>,
) -> usize {
    let keys = std::mem::take(group);
    let Some(table) = table else {
        return 0;
    };
    if keys.len() < 2 {
        return 0;
    }

    let names: Vec<&str> = keys.iter().map(|(_, name)| name.as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort_by_key(|name| name.to_lowercase());
    if names != sorted {
        errors.push(format!(
            "{path}:{}: the keys in [{table}] are not sorted; expected {}",
            keys[0].0,
            sorted.join(", ")
        ));
    }
    1
}

/// Whether a table holds interchangeable entries: a dependency table at the root, under
/// `workspace`, or under a `target`, or a lint table. `[package.metadata.dependencies]` is
/// free-form metadata, not a dependency table.
fn is_sortable(name: &str) -> bool {
    DEPENDENCY_TABLES.iter().any(|table| {
        name == *table
            || name.strip_prefix("workspace.") == Some(table)
            || (name.starts_with("target.") && name.ends_with(&format!(".{table}")))
    }) || name.starts_with("lints.")
        || name.starts_with("workspace.lints.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::Fixture;

    fn errors(text: &str) -> Vec<String> {
        let fixture = Fixture::new().file("templates/Cargo.toml", text);
        run(&fixture.repo()).errors
    }

    #[test]
    fn sorted_tables_are_clean() {
        let fixture = Fixture::new().file(
            "templates/Cargo.toml",
            "[workspace.lints.clippy]\n# A comment stays with its key.\nallow_attributes = \"warn\"\nManual_let_else = \"warn\"\n\n[dependencies]\nregex = \"1\"\nserde = { version = \"1\" }\n",
        );
        let report = run(&fixture.repo());
        assert_eq!(report.errors, Vec::<String>::new());
        assert_eq!(
            report.summary,
            "All 2 sortable key groups in 1 TOML templates are in order."
        );
    }

    #[test]
    fn an_unsorted_lint_table_is_a_finding() {
        assert_eq!(
            errors(
                "[workspace.lints.clippy]\nmanual_let_else = \"warn\"\nallow_attributes = \"warn\"\n"
            ),
            [
                "templates/Cargo.toml:2: the keys in [workspace.lints.clippy] are not sorted; expected allow_attributes, manual_let_else"
            ]
        );
    }

    #[test]
    fn an_unsorted_target_dependency_table_is_a_finding() {
        assert_eq!(
            errors("[target.'cfg(unix)'.dependencies]\nlibc = \"0.2\"\nanyhow = \"1\"\n"),
            [
                "templates/Cargo.toml:2: the keys in [target.'cfg(unix)'.dependencies] are not sorted; expected anyhow, libc"
            ]
        );
    }

    #[test]
    fn a_blank_line_starts_a_new_group() {
        assert_eq!(
            errors("[dependencies]\ntokio = \"1\"\n\nanyhow = \"1\"\n"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn other_tables_keep_their_order() {
        assert_eq!(
            errors(
                "[workspace]\nmembers = []\nresolver = \"3\"\n\n[package]\nname = \"x\"\nedition = \"2024\"\n"
            ),
            Vec::<String>::new()
        );
    }

    #[test]
    fn indented_keys_are_checked() {
        assert_eq!(
            errors("[dependencies]\n  tokio = \"1\"\n  anyhow = \"1\"\n"),
            [
                "templates/Cargo.toml:2: the keys in [dependencies] are not sorted; expected anyhow, tokio"
            ]
        );
    }

    #[test]
    fn a_multiline_string_is_not_parsed() {
        assert_eq!(
            errors(
                "[package.metadata.notes]\ntext = \"\"\"\n[dependencies]\ntokio = \"1\"\nanyhow = \"1\"\n\"\"\"\n"
            ),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_delimiter_in_a_comment_opens_no_string() {
        assert_eq!(
            errors(
                "[dependencies]\n# Use \"\"\" for multiline values.\ntokio = \"1\"\nanyhow = \"1\"\n"
            ),
            [
                "templates/Cargo.toml:3: the keys in [dependencies] are not sorted; expected anyhow, tokio"
            ]
        );
    }

    #[test]
    fn package_metadata_dependencies_keep_their_order() {
        assert_eq!(
            errors("[package.metadata.dependencies]\ntokio = \"1\"\nanyhow = \"1\"\n"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn an_array_of_tables_ends_a_sortable_table() {
        assert_eq!(
            errors("[dependencies]\nanyhow = \"1\"\n[[bin]]\nname = \"z\"\npath = \"a\"\n"),
            Vec::<String>::new()
        );
    }
}
