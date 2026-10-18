//! `validate-readme-index`: README.md indexes the repository in both directions. Every file
//! under `opinions/` has a linked Scope bullet and a Repository layout entry, every directory
//! under `skills/` has a row in the Maintenance via skills table, and none of those three
//! places points at something that no longer exists. All three are held to their order too:
//! Scope by title, the tree by file name, and the skills table by skill name.
//!
//! A section is required only once the directory it indexes exists, because this repository
//! gains `opinions/` and `skills/` phase by phase (PLAN.md).

use std::collections::BTreeSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::Report;
use crate::repo::{self, Repo};

const README: &str = "README.md";
const FAILURE: &str = "README index validation failed:";

// A level-2 heading line; group 1 is the heading text.
static SECTION_HEADING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^## (.*)$").expect("valid pattern"));

// An inline link to an opinion file, with an optional #anchor and optional "title".
static INLINE_OPINION_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"]\(\s*(opinions/[\w.-]+\.md)(?:#[^)\s]*)?(?:\s+"[^"]*")?\s*\)"#)
        .expect("valid pattern")
});

// A reference-style link definition: [label]: opinions/x.md
static REFERENCE_OPINION_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*\[[^\]]+]:\s*(opinions/[\w.-]+\.md)").expect("valid pattern")
});

// A Scope bullet's bold title: group 1 when it's a link, as in **[Title](...):**, and group 2
// when it's plain text, as in **Crates:**.
static SCOPE_BULLET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^- \*\*(?:\[([^\]]+)]|([^*]+?):?\*\*)").expect("valid pattern")
});

static TREE_OPINIONS_NODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[├└]── opinions/").expect("valid pattern"));
static TREE_NODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[├└]── ").expect("valid pattern"));
static MARKDOWN_FILE_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([\w.-]+\.md)").expect("valid pattern"));

// A first cell is a bare ID, `vet-source`, or a linked one: [`vet-source`](skills/…/SKILL.md).
static TABLE_ID: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\A(?:\[\s*)?`([\w.-]+)`(?:\s*\]\([^)]*\))?\z").expect("valid pattern")
});

pub fn run(repo: &Repo) -> Report {
    let Ok(readme) = repo.read(README) else {
        return Report {
            failure: FAILURE,
            errors: vec!["README.md not found".into()],
            summary: String::new(),
        };
    };

    let mut errors = Vec::new();
    let has_opinions = repo.is_dir(repo::OPINIONS);
    let has_skills = repo.is_dir(repo::SKILLS);

    let scope = section(&readme, "Scope");
    let layout = section(&readme, "Repository layout");
    let skills_table = section(&readme, "Maintenance via skills");

    for (name, body, required) in [
        ("Scope", scope, true),
        ("Repository layout", layout, true),
        ("Maintenance via skills", skills_table, has_skills),
    ] {
        if required && body.is_none() {
            errors.push(format!(
                "README.md: '## {name}' section not found — the index checks below cannot run"
            ));
        }
    }

    let opinions = repo.opinions();
    if has_opinions && opinions.is_empty() {
        errors.push(format!("no markdown files found under {}/", repo::OPINIONS));
    }

    let paths: BTreeSet<&str> = opinions.iter().map(String::as_str).collect();
    let names: BTreeSet<&str> = opinions
        .iter()
        .filter_map(|path| path.strip_prefix("opinions/"))
        .collect();

    if let Some(scope) = scope {
        check_scope(scope, &paths, &mut errors);
    }

    if let Some(layout) = layout {
        check_layout(layout, &names, has_opinions, &mut errors);
    }

    let skills = repo.skills();
    if has_skills && skills.is_empty() {
        errors.push(format!(
            "no skill directories found under {}/",
            repo::SKILLS
        ));
    }

    if let Some(table) = skills_table {
        check_skills(table, &skills, &mut errors);
    }

    Report {
        failure: FAILURE,
        errors,
        summary: format!(
            "README indexes all {} opinion files and {} skills, in order, with no stale entries.",
            opinions.len(),
            skills.len()
        ),
    }
}

fn check_scope(scope: &str, paths: &BTreeSet<&str>, errors: &mut Vec<String>) {
    // Opinion paths the Scope section links to, in any Markdown link style.
    let linked: BTreeSet<&str> = INLINE_OPINION_LINK
        .captures_iter(scope)
        .chain(REFERENCE_OPINION_LINK.captures_iter(scope))
        .filter_map(|link| link.get(1))
        .map(|path| path.as_str())
        .collect();

    for path in paths.difference(&linked) {
        errors.push(format!(
            "README.md: Scope has no bullet linking to {path} — add one when adding an opinion"
        ));
    }

    for stale in linked.difference(paths) {
        errors.push(format!(
            "README.md: Scope links to {stale}, which no longer exists — drop or repoint the bullet"
        ));
    }

    // Scope is kept alphabetical so a new bullet lands in one predictable place. By title, not
    // file name, and ignoring case. An unlinked bullet is a summary across the files above it
    // rather than a topic of its own, so it stays last.
    let mut previous: Option<&str> = None;
    let mut unlinked: Option<&str> = None;

    for bullet in SCOPE_BULLET.captures_iter(scope) {
        let (title, is_link) = match (bullet.get(1), bullet.get(2)) {
            (Some(title), _) => (title.as_str(), true),
            (None, Some(title)) => (title.as_str(), false),
            (None, None) => continue,
        };

        if !is_link {
            unlinked.get_or_insert(title);
            continue;
        }

        if let Some(unlinked) = unlinked {
            errors.push(format!(
                "README.md: Scope bullet '{title}' follows the unlinked '{unlinked}' bullet — unlinked bullets stay last"
            ));
        } else if let Some(previous) = previous
            && previous.to_lowercase() > title.to_lowercase()
        {
            errors.push(format!(
                "README.md: Scope is not sorted by title — '{title}' follows '{previous}'"
            ));
        }

        previous = Some(title);
    }
}

fn check_layout(
    layout: &str,
    names: &BTreeSet<&str>,
    has_opinions: bool,
    errors: &mut Vec<String>,
) {
    let listed = tree_opinions(layout);

    if has_opinions && listed.is_empty() {
        errors.push(format!(
            "README.md: Repository layout tree has no `{}/` node — the tree check cannot run",
            repo::OPINIONS
        ));
    }

    let listed_set: BTreeSet<&str> = listed.iter().copied().collect();
    for missing in names.difference(&listed_set) {
        errors.push(format!(
            "README.md: Repository layout tree does not list opinions/{missing}"
        ));
    }

    for stale in listed_set.difference(names) {
        errors.push(format!(
            "README.md: Repository layout tree lists opinions/{stale}, which no longer exists"
        ));
    }

    // Sorted by file name, the order a directory listing uses.
    for pair in listed.windows(2) {
        if pair[0] > pair[1] {
            errors.push(format!(
                "README.md: Repository layout tree is not sorted under `{}/` — {} follows {}",
                repo::OPINIONS,
                pair[1],
                pair[0]
            ));
        }
    }
}

fn check_skills(table: &str, skills: &[String], errors: &mut Vec<String>) {
    let rows = table_ids(table);

    if !skills.is_empty() && rows.is_empty() {
        errors
            .push("README.md: no skill rows parsed from the 'Maintenance via skills' table".into());
    }

    let skill_set: BTreeSet<&str> = skills.iter().map(String::as_str).collect();
    let row_set: BTreeSet<&str> = rows.iter().copied().collect();

    for missing in skill_set.difference(&row_set) {
        errors.push(format!(
            "README.md: 'Maintenance via skills' table has no row for `{missing}`"
        ));
    }

    for stale in row_set.difference(&skill_set) {
        errors.push(format!(
            "README.md: 'Maintenance via skills' table has a row for `{stale}`, which is not a skill directory"
        ));
    }

    // Sorted by skill name, the order a directory listing of skills/ uses, so a new row lands
    // in one predictable place. The pipeline diagram above the table shows the workflow order.
    for pair in rows.windows(2) {
        if pair[0] > pair[1] {
            errors.push(format!(
                "README.md: 'Maintenance via skills' table is not sorted by skill name — `{}` follows `{}`",
                pair[1], pair[0]
            ));
        }
    }
}

/// The body of a level-2 section, up to the next level-2 heading.
pub fn section<'a>(text: &'a str, heading: &str) -> Option<&'a str> {
    let headings: Vec<regex::Captures> = SECTION_HEADING.captures_iter(text).collect();
    let index = headings
        .iter()
        .position(|found| found[1].trim() == heading)?;
    let start = headings[index].get(0)?.end();
    let end = headings
        .get(index + 1)
        .and_then(|next| next.get(0))
        .map_or(text.len(), |next| next.start());
    Some(&text[start..end])
}

// Files listed under the `opinions/` node of the repository layout tree, in tree order.
fn tree_opinions(layout: &str) -> Vec<&str> {
    let mut listed = Vec::new();
    let mut inside = false;

    for line in layout.split('\n') {
        if TREE_OPINIONS_NODE.is_match(line) {
            inside = true;
            continue;
        }

        if !inside {
            continue;
        }

        if TREE_NODE.is_match(line) || line.starts_with("```") {
            break;
        }

        if let Some(name) = MARKDOWN_FILE_NAME
            .captures(line)
            .and_then(|found| found.get(1))
        {
            listed.push(name.as_str());
        }
    }

    listed
}

// First-cell `code` IDs from the Markdown table rows in a section, in table order.
fn table_ids(body: &str) -> Vec<&str> {
    body.split('\n')
        .filter(|line| line.starts_with('|'))
        .filter_map(|line| line.trim().trim_matches('|').split('|').next())
        .filter_map(|first| TABLE_ID.captures(first.trim()))
        .filter_map(|found| found.get(1))
        .map(|id| id.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::Fixture;

    const README_TEXT: &str = "# Awesome Humans of Rust\n\n## Scope\n\n- **[Error handling](opinions/error-handling.md):** errors\n- **[Rust](opinions/rust.md):** the language\n- **Crates:** covered above\n\n## Repository layout\n\n```text\n├── README.md\n├── opinions/\n│   ├── error-handling.md\n│   └── rust.md\n└── skills/\n```\n\n## Maintenance via skills\n\n| Skill | Purpose |\n| --- | --- |\n| [`audit-freshness`](skills/audit-freshness/SKILL.md) | Report |\n| [`vet-source`](skills/vet-source/SKILL.md) | Vet |\n\n## Licence\n";

    fn fixture(readme: &str) -> Fixture {
        Fixture::new()
            .file("README.md", readme)
            .file("opinions/error-handling.md", "")
            .file("opinions/rust.md", "")
            .file("skills/audit-freshness/SKILL.md", "")
            .file("skills/vet-source/SKILL.md", "")
    }

    #[test]
    fn a_complete_index_is_clean() {
        assert_eq!(
            run(&fixture(README_TEXT).repo()).errors,
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_readme_before_opinions_and_skills_exist_is_clean() {
        let readme = "# Rust\n\n## Scope\n\nNone yet.\n\n## Repository layout\n\n```text\n├── README.md\n```\n";
        let fixture = Fixture::new().file("README.md", readme);
        assert_eq!(run(&fixture.repo()).errors, Vec::<String>::new());
    }

    #[test]
    fn an_out_of_order_scope_bullet_is_a_finding() {
        let readme = README_TEXT.replace(
            "- **[Error handling](opinions/error-handling.md):** errors\n- **[Rust](opinions/rust.md):** the language\n",
            "- **[Rust](opinions/rust.md):** the language\n- **[Error handling](opinions/error-handling.md):** errors\n",
        );
        assert_eq!(
            run(&fixture(&readme).repo()).errors,
            ["README.md: Scope is not sorted by title — 'Error handling' follows 'Rust'"]
        );
    }

    #[test]
    fn a_linked_bullet_after_an_unlinked_one_is_a_finding() {
        let readme = README_TEXT.replace(
            "- **Crates:** covered above\n",
            "- **Crates:** covered above\n- **[Testing](opinions/testing.md):** tests\n",
        );
        let fixture = fixture(&readme).file("opinions/testing.md", "");
        let errors = run(&fixture.repo()).errors;
        assert!(errors.contains(&"README.md: Scope bullet 'Testing' follows the unlinked 'Crates' bullet — unlinked bullets stay last".to_string()), "{errors:?}");
    }

    #[test]
    fn an_opinion_missing_from_the_index_is_a_finding() {
        let fixture = fixture(README_TEXT).file("opinions/testing.md", "");
        assert_eq!(
            run(&fixture.repo()).errors,
            [
                "README.md: Scope has no bullet linking to opinions/testing.md — add one when adding an opinion",
                "README.md: Repository layout tree does not list opinions/testing.md",
            ]
        );
    }

    #[test]
    fn a_stale_entry_is_a_finding() {
        let fixture = Fixture::new()
            .file("README.md", README_TEXT)
            .file("opinions/rust.md", "")
            .file("skills/audit-freshness/SKILL.md", "")
            .file("skills/vet-source/SKILL.md", "");
        assert_eq!(
            run(&fixture.repo()).errors,
            [
                "README.md: Scope links to opinions/error-handling.md, which no longer exists — drop or repoint the bullet",
                "README.md: Repository layout tree lists opinions/error-handling.md, which no longer exists",
            ]
        );
    }

    #[test]
    fn an_out_of_order_skill_row_is_a_finding() {
        let readme = README_TEXT.replace(
            "| [`audit-freshness`](skills/audit-freshness/SKILL.md) | Report |\n| [`vet-source`](skills/vet-source/SKILL.md) | Vet |\n",
            "| [`vet-source`](skills/vet-source/SKILL.md) | Vet |\n| [`audit-freshness`](skills/audit-freshness/SKILL.md) | Report |\n",
        );
        assert_eq!(
            run(&fixture(&readme).repo()).errors,
            [
                "README.md: 'Maintenance via skills' table is not sorted by skill name — `audit-freshness` follows `vet-source`"
            ]
        );
    }

    #[test]
    fn a_missing_skills_section_is_a_finding_once_skills_exist() {
        let readme = README_TEXT.replace("## Maintenance via skills", "## Skills");
        assert_eq!(
            run(&fixture(&readme).repo()).errors,
            [
                "README.md: '## Maintenance via skills' section not found — the index checks below cannot run"
            ]
        );
    }

    #[test]
    fn an_out_of_order_tree_is_a_finding() {
        let readme = README_TEXT.replace(
            "│   ├── error-handling.md\n│   └── rust.md\n",
            "│   ├── rust.md\n│   └── error-handling.md\n",
        );
        assert_eq!(
            run(&fixture(&readme).repo()).errors,
            [
                "README.md: Repository layout tree is not sorted under `opinions/` — error-handling.md follows rust.md"
            ]
        );
    }
}
