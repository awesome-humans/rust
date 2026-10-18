//! `validate-metadata`: every resource carries the metadata fields AGENTS.md requires
//! (`targets`, `last-reviewed`, and `sources`), the dates are ISO 8601, and each sources
//! list is in ordinal order.
//!
//! Three resource kinds carry them two ways. `opinions/` and `research/` use YAML
//! frontmatter, and `templates/` use a first-line comment header. One required set spans
//! both syntaxes, so they can't drift apart.

use std::sync::LazyLock;

use jiff::civil::Date;
use regex::Regex;
use serde_yaml_ng::{Mapping, Value};

use crate::repo::{self, Repo};
use crate::{Report, comment_header, frontmatter};

const REQUIRED: &[&str] = &["targets", "last-reviewed", "sources"];

// last-used is retired everywhere: whether a resource is still referenced is read from the
// repository by audit-freshness, not stored. A research topic also must not carry a status:
// a topic on disk is unresolved by definition, and deletion is the only promotion signal
// (AGENTS.md). Both are rejected, not ignored, so the retired shapes can't come back.
const FORBIDDEN: &[&str] = &["last-used"];
const RESEARCH_FORBIDDEN: &[&str] = &["last-used", "status"];

const DATE_FIELDS: &[&str] = &["last-reviewed"];

static ISO_DATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\A\d{4}-\d{2}-\d{2}\z").expect("valid pattern"));

struct Resource {
    path: String,
    fields: Result<Mapping, String>,
    forbidden: &'static [&'static str],
}

pub fn run(repo: &Repo) -> Report {
    let mut errors = Vec::new();
    let mut resources = Vec::new();

    let opinions = repo.opinions();
    if repo.is_dir(repo::OPINIONS) && opinions.is_empty() {
        errors.push(format!("no markdown files found under {}/", repo::OPINIONS));
    }

    for path in &opinions {
        resources.push(Resource {
            fields: read(repo, path, frontmatter::parse),
            path: path.clone(),
            forbidden: FORBIDDEN,
        });
    }

    // research/ is staging, and legitimately empty once every topic has been resolved, because
    // resolve-research deletes the file it finishes with. So an empty or absent directory
    // isn't a finding.
    let research = repo.markdown_in(repo::RESEARCH);
    for path in &research {
        resources.push(Resource {
            fields: read(repo, path, frontmatter::parse),
            path: path.clone(),
            forbidden: RESEARCH_FORBIDDEN,
        });
    }

    let templates = repo.header_templates();
    if repo.is_dir(repo::TEMPLATES) && templates.is_empty() {
        errors.push(format!(
            "no header-carrying files found under {}/",
            repo::TEMPLATES
        ));
    }

    for path in &templates {
        resources.push(Resource {
            fields: read(repo, path, comment_header::parse),
            path: path.clone(),
            forbidden: FORBIDDEN,
        });
    }

    for resource in &resources {
        check(resource, &mut errors);
    }

    Report {
        failure: "Resource metadata validation failed:",
        errors,
        summary: format!(
            "All {} resources carry valid metadata ({} opinions, {} research, {} templates).",
            resources.len(),
            opinions.len(),
            research.len(),
            templates.len()
        ),
    }
}

fn read(
    repo: &Repo,
    path: &str,
    parse: fn(&str) -> Result<Mapping, String>,
) -> Result<Mapping, String> {
    repo.read(path)
        .map_err(|error| format!("can't be read: {error}"))
        .and_then(|text| parse(&text))
}

fn check(resource: &Resource, errors: &mut Vec<String>) {
    let path = &resource.path;
    let fields = match &resource.fields {
        Ok(fields) => fields,
        Err(error) => {
            errors.push(format!("{path}: {error}"));
            return;
        }
    };

    for field in REQUIRED {
        if fields.get(field).is_none_or(is_empty) {
            errors.push(format!("{path}: missing or empty metadata field '{field}'"));
        }
    }

    for field in resource.forbidden {
        if fields.contains_key(field) {
            errors.push(format!("{path}: field '{field}' is not allowed here"));
        }
    }

    for field in DATE_FIELDS {
        let Some(value) = fields.get(field).filter(|value| !value.is_null()) else {
            continue;
        };

        match frontmatter::scalar(value) {
            Some(text) if is_iso_date(&text) => {}
            Some(text) => errors.push(format!(
                "{path}: '{field}' is '{text}', not an ISO 8601 date (YYYY-MM-DD)"
            )),
            None => errors.push(format!(
                "{path}: '{field}' is not a single ISO 8601 date (YYYY-MM-DD)"
            )),
        }
    }

    // AGENTS.md: a sources list is sorted by ID, house profiles included. One order means two
    // edits that add the same source land on the same line, and a reader scanning for an ID
    // knows where to look.
    if let Some(ids) = fields.get("sources").and_then(frontmatter::list) {
        let mut sorted = ids.clone();
        sorted.sort();
        if ids != sorted {
            errors.push(format!(
                "{path}: 'sources' is not sorted; expected [{}]",
                sorted.join(", ")
            ));
        }
    }
}

// A key that is present but carries nothing (null, "", []) is as much a gap as a missing
// key, so both fail the same way.
fn is_empty(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(text) => text.is_empty(),
        Value::Sequence(items) => items.is_empty(),
        Value::Mapping(entries) => entries.is_empty(),
        _ => false,
    }
}

// AGENTS.md: dates are ISO 8601, always absolute, never relative. The exact shape is what
// rejects "yesterday", "August 2026", and 20-08-2026, and the parse rejects 2026-02-30.
fn is_iso_date(text: &str) -> bool {
    ISO_DATE.is_match(text) && text.parse::<Date>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::Fixture;

    const OPINION: &str = "---\ntargets: [rust-1.99, edition-2024]\nlast-reviewed: 2026-10-14\nsources: [matklad, rust-docs]\n---\n# Rust\n";
    const TEMPLATE: &str = "# awesome-humans/rust template | targets: rust-1.99 | last-reviewed: 2026-10-14 | sources: rust-docs\n[workspace]\n";

    #[test]
    fn a_repository_without_resources_is_clean() {
        let report = run(&Fixture::new().repo());
        assert_eq!(report.errors, Vec::<String>::new());
        assert_eq!(
            report.summary,
            "All 0 resources carry valid metadata (0 opinions, 0 research, 0 templates)."
        );
    }

    #[test]
    fn valid_resources_are_clean() {
        let fixture = Fixture::new()
            .file("opinions/rust.md", OPINION)
            .file("templates/Cargo.toml", TEMPLATE)
            .file("templates/example.json", "{}")
            .file("templates/target/debug/build.toml", "no header");
        let report = run(&fixture.repo());
        assert_eq!(report.errors, Vec::<String>::new());
        assert_eq!(
            report.summary,
            "All 2 resources carry valid metadata (1 opinions, 0 research, 1 templates)."
        );
    }

    #[test]
    fn an_empty_opinions_directory_is_a_finding() {
        let fixture = Fixture::new().dir("opinions");
        assert_eq!(
            run(&fixture.repo()).errors,
            ["no markdown files found under opinions/"]
        );
    }

    #[test]
    fn a_relative_date_is_a_finding() {
        let fixture = Fixture::new().file(
            "opinions/rust.md",
            &OPINION.replace("last-reviewed: 2026-10-14", "last-reviewed: yesterday"),
        );
        assert_eq!(
            run(&fixture.repo()).errors,
            ["opinions/rust.md: 'last-reviewed' is 'yesterday', not an ISO 8601 date (YYYY-MM-DD)"]
        );
    }

    #[test]
    fn an_impossible_date_is_a_finding() {
        let fixture = Fixture::new().file(
            "templates/Cargo.toml",
            &TEMPLATE.replace("last-reviewed: 2026-10-14", "last-reviewed: 2026-02-30"),
        );
        assert_eq!(
            run(&fixture.repo()).errors,
            [
                "templates/Cargo.toml: 'last-reviewed' is '2026-02-30', not an ISO 8601 date (YYYY-MM-DD)"
            ]
        );
    }

    #[test]
    fn a_missing_field_is_a_finding() {
        let fixture = Fixture::new().file(
            "templates/Cargo.toml",
            &TEMPLATE.replace(" | sources: rust-docs", ""),
        );
        assert_eq!(
            run(&fixture.repo()).errors,
            ["templates/Cargo.toml: missing or empty metadata field 'sources'"]
        );
    }

    #[test]
    fn an_empty_list_is_a_finding() {
        let fixture = Fixture::new().file(
            "opinions/rust.md",
            &OPINION.replace("targets: [rust-1.99, edition-2024]", "targets: []"),
        );
        assert_eq!(
            run(&fixture.repo()).errors,
            ["opinions/rust.md: missing or empty metadata field 'targets'"]
        );
    }

    #[test]
    fn unsorted_sources_are_a_finding() {
        let fixture = Fixture::new().file(
            "opinions/rust.md",
            &OPINION.replace(
                "[matklad, rust-docs]",
                "[rust-docs, house:simon-mckenna, matklad]",
            ),
        );
        assert_eq!(
            run(&fixture.repo()).errors,
            [
                "opinions/rust.md: 'sources' is not sorted; expected [house:simon-mckenna, matklad, rust-docs]"
            ]
        );
    }

    #[test]
    fn last_used_is_a_finding() {
        let fixture = Fixture::new()
            .file(
                "opinions/rust.md",
                &OPINION.replace("---\n# Rust", "last-used: 2026-10-14\n---\n# Rust"),
            )
            .file(
                "templates/Cargo.toml",
                &TEMPLATE.replace(" | sources", " | last-used: 2026-10-14 | sources"),
            );
        assert_eq!(
            run(&fixture.repo()).errors,
            [
                "opinions/rust.md: field 'last-used' is not allowed here",
                "templates/Cargo.toml: field 'last-used' is not allowed here",
            ]
        );
    }

    #[test]
    fn a_research_topic_rejects_the_retired_fields() {
        let fixture = Fixture::new().file(
            "research/async.md",
            &OPINION.replace(
                "---\n# Rust",
                "last-used: 2026-10-14\nstatus: open\n---\n# Rust",
            ),
        );
        assert_eq!(
            run(&fixture.repo()).errors,
            [
                "research/async.md: field 'last-used' is not allowed here",
                "research/async.md: field 'status' is not allowed here",
            ]
        );
    }

    #[test]
    fn a_template_without_a_header_is_a_finding() {
        let fixture = Fixture::new().file("templates/rustfmt.toml", "edition = \"2024\"\n");
        assert_eq!(
            run(&fixture.repo()).errors,
            ["templates/rustfmt.toml: first line carries no comment header"]
        );
    }
}
