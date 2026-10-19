//! `validate-sources`: every source ID in an opinion's frontmatter or a template's comment
//! header resolves to the roster in `AWESOME-HUMANS.md`, and is allowed to feed one.
//! Watch-list sources haven't been admitted yet, opted-out sources asked not to be cited, and
//! discovery-only sources are channels that lead to a primary source rather than being one.
//! `templates/` is held to the same gate as `opinions/`, because it's what people copy.
//!
//! `research/` is deliberately outside the gate. A topic may cite unvetted material as long as
//! the text flags it (research-topic), and resolve-research drops or re-sources every
//! unvetted claim when it promotes one.
//!
//! A house profile's ID, `house:<profile>`, isn't on the roster. Instead, each one in a file's
//! sources must match a house block in the opinion, or a house region in the template, and
//! every house opinion must sit inside one (awesome-humans/website: House opinions).
//!
//! It also holds the roster to its shape: each table sorted by ID, no ID used twice, and the
//! count sentence above the Citable table in step with its rows. The roster is parsed by ID,
//! so a duplicate would otherwise decide a source's bucket by whichever row came last.

use std::collections::hash_map::Entry as Slot;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;

use crate::repo::Repo;
use crate::{Report, comment_header, frontmatter, house};

pub const ROSTER: &str = "AWESOME-HUMANS.md";
const DISCOVERY_MARKER: &str = "**Discovery-only.**";
const CORROBORATE_MARKER: &str = "**Corroborate.**";
const NOTES_HEADING: &str = "## Source notes";
const FAILURE: &str = "Roster and source validation failed:";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Bucket {
    Citable,
    Watch,
    OptedOut,
}

// Which level-2 headings open which bucket. Anything outside them, such as the admission
// criteria and the decision log, carries no source rows.
const SECTIONS: &[(Bucket, &str)] = &[
    (Bucket::Citable, "## Citable"),
    (Bucket::Watch, "## Watch list"),
    (Bucket::OptedOut, "## Opted out"),
];

// A roster row's first cell is a bare source ID in backticks: `matklad`.
static SOURCE_ID: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\A`([a-z0-9-]+)`\z").expect("valid pattern"));

// A notes heading opens with the ID and then says where the source stands:
// '### `matklad`: Citable, **Corroborate.**'.
static NOTES_HEADING_ID: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\A### `([a-z0-9-]+)`").expect("valid pattern"));

// The count sentence above the Citable table.
static CITABLE_COUNT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\AThe table lists (\d+) sources: (\d+) people and (\d+) publications\.\z")
        .expect("valid pattern")
});

struct Entry {
    bucket: Bucket,
    notes: String,
}

pub fn run(repo: &Repo) -> Report {
    let mut errors = Vec::new();
    let citing = citing(repo, &mut errors);

    // The roster arrives in phase 3 of PLAN.md. Until then, nothing may cite a source.
    let Ok(text) = repo.read(ROSTER) else {
        if citing.is_empty() {
            return Report {
                failure: FAILURE,
                errors,
                summary: format!("No {ROSTER} yet, and no opinion or template cites a source."),
            };
        }

        errors.push(format!("{ROSTER} not found"));
        return failed(errors);
    };

    let roster = Roster::parse(&text, &mut errors);

    for (path, ids) in &citing {
        check_citations(path, ids, &roster, &mut errors);

        match repo.read(path) {
            Ok(text) => check_house(path, ids, &text, &mut errors),
            Err(error) => errors.push(format!("{path}: can't be read: {error}")),
        }
    }

    Report {
        failure: FAILURE,
        errors,
        summary: format!(
            "All sources in {} opinions and templates resolve to the roster ({} sources across {} tables, {} with notes, {} discovery-only, {} corroborate-only), and every table and the notes are sorted by id.",
            citing.len(),
            roster.entries.len(),
            roster.tables,
            roster.notes,
            roster.discovery.len(),
            roster.corroborate.len()
        ),
    }
}

fn failed(errors: Vec<String>) -> Report {
    Report {
        failure: FAILURE,
        errors,
        summary: String::new(),
    }
}

// Every resource held to the roster, with its declared IDs. A file with no frontmatter, or
// unreadable frontmatter, is reported in detail by validate-metadata, so this check stays
// quiet about it.
fn citing(repo: &Repo, errors: &mut Vec<String>) -> Vec<(String, Vec<String>)> {
    let mut citing = Vec::new();

    for path in repo.opinions() {
        let Ok(fields) = repo
            .read(&path)
            .map_err(|error| error.to_string())
            .and_then(|text| frontmatter::parse(&text))
        else {
            continue;
        };

        match fields.get("sources") {
            None | Some(serde_yaml_ng::Value::Null) => {}
            Some(serde_yaml_ng::Value::String(text)) if text.is_empty() => {}
            Some(serde_yaml_ng::Value::Sequence(items)) => citing.push((
                path,
                items
                    .iter()
                    .map(|item| frontmatter::scalar(item).unwrap_or_default())
                    .collect(),
            )),
            Some(_) => errors.push(format!("{path}: 'sources' must be a list")),
        }
    }

    // A template carries the same IDs comma-separated inside its one-line comment header.
    for path in repo.header_templates() {
        let Ok(fields) = repo
            .read(&path)
            .map_err(|error| error.to_string())
            .and_then(|text| comment_header::parse(&text))
        else {
            continue;
        };

        if let Some(ids) = fields.get("sources").and_then(frontmatter::list) {
            citing.push((path, ids));
        }
    }

    citing
}

struct Roster {
    entries: HashMap<String, Entry>,
    discovery: HashSet<String>,
    corroborate: HashSet<String>,
    tables: usize,
    notes: usize,
}

impl Roster {
    #[expect(
        clippy::too_many_lines,
        reason = "a straight port of the .NET script's one function, kept in its order so the two can be compared"
    )]
    fn parse(text: &str, errors: &mut Vec<String>) -> Self {
        let mut entries: HashMap<String, Entry> = HashMap::new();

        // Notes sections in file order, for the same alphabetical check the tables get.
        let mut note_ids: Vec<String> = Vec::new();
        let mut notes: HashMap<String, String> = HashMap::new();

        // None outside '## Source notes', Some("") inside it before the first '### `id`'.
        let mut notes_for: Option<String> = None;

        // IDs in the order they appear, one list per table. Order is checked per table, not
        // across the file, because the watch list starts over at 'a'.
        let mut tables: Vec<(String, Vec<String>)> = Vec::new();
        let mut bucket: Option<Bucket> = None;

        // The Citable section states how many people and publications its table holds, and
        // the site shows that sentence. A hand-kept count drifts with every admission, so the
        // rows are counted here and the sentence has to agree.
        let mut stated: Option<(String, String, String)> = None;
        let mut people = 0;
        let mut publications = 0;

        for line in text.split('\n') {
            if line.starts_with("### ") && notes_for.is_some() {
                if let Some(id) = NOTES_HEADING_ID.captures(line) {
                    let id = id[1].to_string();
                    note_ids.push(id.clone());
                    if notes.contains_key(&id) {
                        errors.push(format!(
                            "{ROSTER}: source id '{id}' has more than one notes section"
                        ));
                    } else {
                        notes.insert(id.clone(), line.to_string());
                    }
                    notes_for = Some(id);
                }
                continue;
            }

            if let Some(heading) = line.strip_prefix("## ") {
                notes_for = line.starts_with(NOTES_HEADING).then(String::new);
                bucket = SECTIONS
                    .iter()
                    .find(|(_, prefix)| line.starts_with(prefix))
                    .map(|(name, _)| *name);
                if bucket.is_some() {
                    tables.push((heading.trim().to_string(), Vec::new()));
                }
                continue;
            }

            if let Some(id) = notes_for.as_ref().filter(|id| !id.is_empty()) {
                if let Some(body) = notes.get_mut(id) {
                    body.push('\n');
                    body.push_str(line);
                }
                continue;
            }

            if bucket == Some(Bucket::Citable)
                && let Some(count) = CITABLE_COUNT.captures(line)
            {
                stated = Some((count[1].into(), count[2].into(), count[3].into()));
                continue;
            }

            let Some(current) = bucket else { continue };
            if !line.starts_with('|') {
                continue;
            }

            let cells: Vec<&str> = line
                .trim()
                .trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect();
            let Some(id) = SOURCE_ID.captures(cells[0]) else {
                continue; // header or separator row
            };

            let id = id[1].to_string();
            if let Some((_, ids)) = tables.last_mut() {
                ids.push(id.clone());
            }

            // The site reads the kind to decide how to take a source's name from its link,
            // so a row missing it, or one the site doesn't know, fails here rather than in
            // the site build.
            if cells.len() < 5 {
                errors.push(format!(
                    "{ROSTER}: the row for '{id}' has {} cells, not the 5 of id, Kind, Source, Focus and Since",
                    cells.len()
                ));
            } else if !matches!(cells[1], "person" | "publication") {
                errors.push(format!(
                    "{ROSTER}: the row for '{id}' has Kind '{}', not 'person' or 'publication'",
                    cells[1]
                ));
            } else if current == Bucket::Citable {
                if cells[1] == "person" {
                    people += 1;
                } else {
                    publications += 1;
                }
            }

            match entries.entry(id) {
                Slot::Occupied(slot) => errors.push(format!(
                    "{ROSTER}: source id '{}' appears on more than one row — ids are unique",
                    slot.key()
                )),
                Slot::Vacant(slot) => {
                    slot.insert(Entry {
                        bucket: current,
                        notes: String::new(),
                    });
                }
            }
        }

        let expected = format!(
            "{} sources: {people} people and {publications} publications",
            people + publications
        );
        match stated {
            None => errors.push(format!(
                "{ROSTER}: '## Citable' has no sentence of the form 'The table lists N sources: N people and N publications.'"
            )),
            Some((total, people, publications)) => {
                let stated = format!("{total} sources: {people} people and {publications} publications");
                if stated != expected {
                    errors.push(format!(
                        "{ROSTER}: '## Citable' says it lists {stated}, but the table has {expected}"
                    ));
                }
            }
        }

        if entries.is_empty() {
            errors.push(format!(
                "{ROSTER}: no source rows parsed — roster tables may have changed shape"
            ));
        }

        // Roster tables are kept alphabetical by ID (vet-source), so a new row lands in one
        // predictable place. Nothing about a 30-row table makes a misplaced row visible.
        for (heading, ids) in &tables {
            for pair in ids.windows(2) {
                if pair[0] > pair[1] {
                    errors.push(format!(
                        "{ROSTER}: '{heading}' is not sorted by id — '{}' follows '{}'",
                        pair[1], pair[0]
                    ));
                }
            }
        }

        for id in &note_ids {
            match entries.get(id) {
                // A notes section for an ID that no row declares is a leftover from a
                // demotion or a typo, and would look authoritative while applying to nothing.
                None => errors.push(format!(
                    "{ROSTER}: '{NOTES_HEADING}' has a section for '{id}', which is on no roster table"
                )),
                // An opted-out source's notes go with it: the evidence was about work it asked
                // us to stop citing, and its row and the decision log are all the roster keeps.
                Some(entry) if entry.bucket == Bucket::OptedOut => errors.push(format!(
                    "{ROSTER}: '{NOTES_HEADING}' has a section for '{id}', which has opted out — delete the section"
                )),
                Some(_) => {}
            }
        }

        for pair in note_ids.windows(2) {
            if pair[0] > pair[1] {
                errors.push(format!(
                    "{ROSTER}: '{NOTES_HEADING}' is not sorted by id — '{}' follows '{}'",
                    pair[1], pair[0]
                ));
            }
        }

        // The markings are read off the section, so fold it in before looking for them.
        for (id, body) in notes {
            if let Some(entry) = entries.get_mut(&id) {
                entry.notes = body;
            }
        }

        let marked = |marker: &str| -> HashSet<String> {
            entries
                .iter()
                .filter(|(_, entry)| entry.notes.contains(marker))
                .map(|(id, _)| id.clone())
                .collect()
        };
        let discovery = marked(DISCOVERY_MARKER);

        // Rows admitted on track record but thin on depth: citable, never the only citation.
        let corroborate = marked(CORROBORATE_MARKER);

        Self {
            entries,
            discovery,
            corroborate,
            tables: tables.len(),
            notes: note_ids.len(),
        }
    }
}

fn check_citations(path: &str, ids: &[String], roster: &Roster, errors: &mut Vec<String>) {
    for id in ids {
        // A house profile isn't on the roster. The pairing check holds it to its blocks.
        if house::profile_of(id).is_some() {
            continue;
        }

        if id == "house" {
            errors.push(format!(
                "{path}: 'house' names no profile — write {}<profile>, such as {}simon-mckenna",
                house::PREFIX,
                house::PREFIX
            ));
            continue;
        }

        let Some(entry) = roster.entries.get(id) else {
            errors.push(format!(
                "{path}: source '{id}' is not on the roster in {ROSTER}"
            ));
            continue;
        };

        match entry.bucket {
            Bucket::Watch => errors.push(format!(
                "{path}: source '{id}' is on the watch list and may not feed opinions or templates — corroborate with a citable source or admit it via vet-source"
            )),
            Bucket::OptedOut => errors.push(format!(
                "{path}: source '{id}' has opted out and may not feed opinions or templates — re-source the claim with research-topic, or cut it back to what other citable sources support"
            )),
            Bucket::Citable if roster.discovery.contains(id) => errors.push(format!(
                "{path}: source '{id}' is marked discovery-only and never appears in 'sources:' — cite the primary source it led you to"
            )),
            Bucket::Citable => {}
        }
    }

    // A Corroborate-marked source may carry the shape of a claim but never the claim alone,
    // so something unmarked has to stand beside it. A house profile doesn't count: it's its
    // owner's position, not a second reading of the evidence.
    let marked = ids.iter().any(|id| roster.corroborate.contains(id));
    let unmarked = ids.iter().any(|id| {
        !roster.corroborate.contains(id)
            && roster.entries.contains_key(id)
            && !roster.discovery.contains(id)
    });
    if marked && !unmarked {
        errors.push(format!(
            "{path}: every citable source here is marked '{CORROBORATE_MARKER}' — add a citable source that is not, or drop the claim"
        ));
    }
}

// A house profile in the sources and its blocks or regions always appear together, and every
// house opinion sits inside one, so removing them leaves no house opinion behind.
fn check_house(path: &str, ids: &[String], text: &str, errors: &mut Vec<String>) {
    let spans = house::spans(path, text, errors);
    let kind = if house::is_template(path) {
        "region"
    } else {
        "block"
    };
    let declared: BTreeSet<&str> = ids.iter().filter_map(|id| house::profile_of(id)).collect();
    let found: BTreeSet<&str> = spans.iter().map(|span| span.profile.as_str()).collect();

    for profile in declared.difference(&found) {
        errors.push(format!(
            "{path}: sources name {}{profile}, but no house {kind} here belongs to it",
            house::PREFIX
        ));
    }

    for profile in found.difference(&declared) {
        errors.push(format!(
            "{path}: a house {kind} belongs to {profile}, but sources don't name {}{profile}",
            house::PREFIX
        ));
    }

    for span in spans.iter().filter(|span| !span.marked) {
        errors.push(format!(
            "{path}:{}: the house {kind} for {} doesn't carry '{}'",
            span.start + 1,
            span.profile,
            house::MARKING
        ));
    }

    for (index, line) in text.split('\n').enumerate() {
        if line.contains(house::MARKING)
            && !spans
                .iter()
                .any(|span| span.start < index && index < span.end)
        {
            errors.push(format!(
                "{path}:{}: '{}' outside a house {kind}, so the default copy would carry it",
                index + 1,
                house::MARKING
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::Fixture;

    const ROSTER_TEXT: &str = "# Awesome humans\n\n## Citable: quotable in opinions and templates\n\nThe table lists 3 sources: 2 people and 1 publications.\n\n| ID | Kind | Source | Focus | Since |\n| --- | --- | --- | --- | --- |\n| `burntsushi` | person | [Andrew Gallant: Blog](https://blog.burntsushi.net/) | Regex, errors | 2014 |\n| `matklad` | person | [Alex Kladov: Blog](https://matklad.github.io/) | Tooling | 2017 |\n| `rust-docs` | publication | [The Rust Programming Language](https://doc.rust-lang.org/) | Language | 2015 |\n\n## Watch list\n\n| ID | Kind | Source | Focus | Since |\n| --- | --- | --- | --- | --- |\n| `newcomer` | person | [New Comer](https://example.org/) | Async | 2025 |\n\n## Opted out\n\n| ID | Kind | Source | Focus | Since |\n| --- | --- | --- | --- | --- |\n\n## Source notes\n\n### `matklad`: Citable, **Corroborate.**\n\nNotes.\n\n## Decision log\n";

    fn roster() -> String {
        ROSTER_TEXT.to_string()
    }

    fn opinion(sources: &str, body: &str) -> String {
        format!(
            "---\ntargets: [rust-1.99]\nlast-reviewed: 2026-10-14\nsources: [{sources}]\n---\n# Rust\n\n{body}"
        )
    }

    #[test]
    fn no_roster_and_no_citations_is_clean() {
        let report = run(&Fixture::new().repo());
        assert_eq!(report.errors, Vec::<String>::new());
    }

    #[test]
    fn a_citation_without_a_roster_is_a_finding() {
        let fixture = Fixture::new().file("opinions/rust.md", &opinion("rust-docs", ""));
        assert_eq!(run(&fixture.repo()).errors, ["AWESOME-HUMANS.md not found"]);
    }

    #[test]
    fn citable_sources_are_clean() {
        let fixture = Fixture::new()
            .file(ROSTER, &roster())
            .file("opinions/rust.md", &opinion("matklad, rust-docs", ""));
        assert_eq!(run(&fixture.repo()).errors, Vec::<String>::new());
    }

    #[test]
    fn a_wrong_count_sentence_is_a_finding() {
        let fixture = Fixture::new().file(ROSTER, &roster().replace("The table lists", "It lists"));
        assert_eq!(
            run(&fixture.repo()).errors,
            [
                "AWESOME-HUMANS.md: '## Citable' has no sentence of the form 'The table lists N sources: N people and N publications.'"
            ]
        );

        let fixture = Fixture::new().file(ROSTER, &roster().replace("2 people", "3 people"));
        assert_eq!(
            run(&fixture.repo()).errors,
            [
                "AWESOME-HUMANS.md: '## Citable' says it lists 3 sources: 3 people and 1 publications, but the table has 3 sources: 2 people and 1 publications"
            ]
        );
    }

    #[test]
    fn a_duplicate_roster_id_is_a_finding() {
        let duplicate = roster().replace("| `newcomer` |", "| `matklad` |");
        let fixture = Fixture::new().file(ROSTER, &duplicate);
        assert_eq!(
            run(&fixture.repo()).errors,
            [
                "AWESOME-HUMANS.md: source id 'matklad' appears on more than one row — ids are unique"
            ]
        );
    }

    #[test]
    fn an_unsorted_table_is_a_finding() {
        let unsorted = roster().replace("`burntsushi`", "`zz-top`");
        let fixture = Fixture::new().file(ROSTER, &unsorted);
        assert_eq!(
            run(&fixture.repo()).errors,
            [
                "AWESOME-HUMANS.md: 'Citable: quotable in opinions and templates' is not sorted by id — 'matklad' follows 'zz-top'"
            ]
        );
    }

    #[test]
    fn an_orphaned_notes_section_is_a_finding() {
        let orphan = roster().replace("### `matklad`", "### `nobody`");
        let fixture = Fixture::new().file(ROSTER, &orphan);
        assert_eq!(
            run(&fixture.repo()).errors,
            [
                "AWESOME-HUMANS.md: '## Source notes' has a section for 'nobody', which is on no roster table"
            ]
        );
    }

    #[test]
    fn sources_that_cant_be_cited_are_findings() {
        let fixture = Fixture::new()
            .file(ROSTER, &roster())
            .file("opinions/a.md", &opinion("newcomer, rust-docs", ""))
            .file("opinions/b.md", &opinion("matklad", ""))
            .file("opinions/c.md", &opinion("house, unknown", ""));
        assert_eq!(
            run(&fixture.repo()).errors,
            [
                "opinions/a.md: source 'newcomer' is on the watch list and may not feed opinions or templates — corroborate with a citable source or admit it via vet-source",
                "opinions/b.md: every citable source here is marked '**Corroborate.**' — add a citable source that is not, or drop the claim",
                "opinions/c.md: 'house' names no profile — write house:<profile>, such as house:simon-mckenna",
                "opinions/c.md: source 'unknown' is not on the roster in AWESOME-HUMANS.md",
            ]
        );
    }

    #[test]
    fn a_house_profile_pairs_with_its_blocks() {
        let block = "/// house | simon-mckenna\n\n**House:** Deny warnings.\n\n///\n";
        let fixture = Fixture::new()
            .file(ROSTER, &roster())
            .file(
                "opinions/a.md",
                &opinion("house:simon-mckenna, rust-docs", block),
            )
            .file(
                "opinions/b.md",
                &opinion("house:simon-mckenna, rust-docs", ""),
            )
            .file("opinions/c.md", &opinion("rust-docs", block))
            .file(
                "opinions/d.md",
                &opinion("rust-docs", "**House:** Loose.\n"),
            );
        assert_eq!(
            run(&fixture.repo()).errors,
            [
                "opinions/b.md: sources name house:simon-mckenna, but no house block here belongs to it",
                "opinions/c.md: a house block belongs to simon-mckenna, but sources don't name house:simon-mckenna",
                "opinions/d.md:8: '**House:**' outside a house block, so the default copy would carry it",
            ]
        );
    }

    #[test]
    fn a_house_region_needs_the_marking() {
        let template = "# awesome-humans/rust template | targets: rust-1.99 | last-reviewed: 2026-10-14 | sources: house:simon-mckenna, rust-docs\n# house:simon-mckenna begin\nwarnings = \"deny\"\n# house:simon-mckenna end\n";
        let fixture = Fixture::new()
            .file(ROSTER, &roster())
            .file("templates/Cargo.toml", template);
        assert_eq!(
            run(&fixture.repo()).errors,
            [
                "templates/Cargo.toml:2: the house region for simon-mckenna doesn't carry '**House:**'"
            ]
        );
    }
}
