//! `build-site`: stages the Rust ecosystem's pages of awesome-humans.net, under `/rust/`.
//! They're generated from this repository's own files, so the site never keeps a second copy
//! to drift: README.md up to its layout section becomes the overview, `opinions/`,
//! `research/`, and `skills/` are copied in full, the roster is split into a page per section, and every file
//! under `templates/` gets a page showing it. `llms.txt` points agents at raw copies of the
//! files under `raw/`, on the site itself, so an agent allowed to fetch only the site can read
//! every one. The agent instructions the skills link to, AGENTS.md and `agents/`, get copies
//! too, and a copy links to a file's page on the site when the file has a page and no copy.
//! Relative links are rewritten to the page they now point at, or to GitHub for a file the
//! site doesn't publish. The navigation is generated too: the opinions take their titles and
//! order from README.md's Scope list, and the skills theirs from its Maintenance via skills
//! table, which `validate-readme-index` holds to the directories.
//!
//! It follows the awesome-humans/website staging contract
//! (<https://github.com/awesome-humans/website#staging-contract>): everything goes under
//! `<dir>/rust/`, with the diagrams in `rust/assets/`, the section's navigation in `nav.json`,
//! the roster in `sources.json`, and this ecosystem's own `llms.txt`. The website repository
//! adds the shared pages, the configuration, and the root `llms.txt`, and builds the site.
//!
//! House opinions stay in the pages as house blocks, and the site shows a profile's only to a
//! reader who chooses it. A template with house regions shows its default copy, with each
//! profile's copy in a house block below it. Agents get the same choice as files: `llms.txt`
//! links copies with every house block and region removed, under `raw/none/`, and
//! `llms-<profile>.txt` links copies with only that profile's, under `raw/<profile>/`.
//!
//! The repository gains the roster, opinions, templates, and skills phase by phase (PLAN.md),
//! so a part that doesn't exist yet stages no pages and no navigation, rather than failing.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use jiff::Timestamp;
use jiff::civil::Date;
use jiff::tz::TimeZone;
use regex::{Captures, Regex};
use serde_json::{Map, Value, json};

use crate::repo::{self, Repo};
use crate::validate_sources::ROSTER;
use crate::{export_diagrams, frontmatter, house};

pub const ECOSYSTEM: &str = "rust";
const ECOSYSTEM_NAME: &str = "Rust";
const REPOSITORY: &str = "awesome-humans/rust";
const SCHEMA_BASE: &str = "https://raw.githubusercontent.com/awesome-humans/website/main/schemas";
const SITE_URL: &str = "https://awesome-humans.net/";

// Where a file links a shared convention, and the copy of it the site serves. The website
// stages each convention to raw/none/conventions/, so an agent that can fetch only the site
// reads it there.
const CONVENTION_SOURCE: &str =
    "https://github.com/awesome-humans/website/blob/main/pages/conventions/";
const CONVENTION_COPY: &str = "https://awesome-humans.net/raw/none/conventions/";
const HOUSE_OPINIONS: &str = "HOUSE-OPINIONS.md";
const README: &str = "README.md";
const AGENTS: &str = "AGENTS.md";
const AGENT_INSTRUCTIONS: &str = "agents";

// The default profile, which has no house opinions. Its copies go under raw/none/.
const NO_PROFILE: &str = "none";

// Days after its last-reviewed date that an opinion counts as stale, from audit-freshness.
const OPINION_TOLERANCE: i32 = 180;

// The anchor of the Citable table's heading, for a source with no notes section to link to.
const CITABLE_ANCHOR: &str = "citable-quotable-in-opinions-and-templates";

// Fetch tools summarise a page unless asked not to, and a summary paraphrases the rules.
const LLMS_READING: &str = "When you read a linked file, read it in full, not a summary, because its exact wording is the rule. A file under `raw/none/` is the default copy, with no house opinions, and one under `raw/<profile>/` is that profile's copy.";

const LLMS_SUMMARY: &str = "Opinionated best practices for building with Rust, distilled from the published writing of vetted people and publications with a multi-year track record. Each opinion states its recommendation first, then the rationale, then its sources, and targets the latest stable release.";

// A bullet in README.md's Scope list: - **[Title](opinions/file.md):** what it covers.
static SCOPE_BULLET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^- \*\*\[(?<title>[^\]]+)\]\((?<path>opinions/[^)]+)\):\*\*(?<summary>.*)$")
        .expect("valid pattern")
});

// A row in README.md's Maintenance via skills table: | [`name`](skills/name/SKILL.md) | purpose |.
static SKILL_ROW: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\| \[`(?<name>[^`]+)`\]\(skills/[^)]+\)\s*\|\s*(?<purpose>.*?)\s*\|$")
        .expect("valid pattern")
});

static HEADING_TRAILING_HASH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?<text>#{1,6} .*[^\s\\#])(?<hashes>#+)$").expect("valid pattern")
});

// A heading, or the opening of a source's collapsed notes, which stands in for one.
static HEADING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)^(?:(?<hashes>#{1,6}) (?<text>.+)|<details id="(?<id>[^"]+)".*)$"#)
        .expect("valid pattern")
});

// The ID cell that opens a roster table row: | `id` |.
static TABLE_ID: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\| `(?<id>[a-z0-9-]+)` +\|").expect("valid pattern"));

// A whole roster table row: | `id` | Kind | Source | Focus | Since |.
static ROSTER_ROW: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\| `(?<id>[a-z0-9-]+)` +\| (?<kind>person|publication) +\| (?<source>[^|]+)\| (?<focus>[^|]+)\| (?<since>[^|]+)\|$")
        .expect("valid pattern")
});

static FIRST_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[(?<text>[^\]]+)\]\((?<url>https://[^)\s]+)\)").expect("valid pattern")
});
static PERSON_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?<name>.+?)(?::\s| \(|'s\s|$)").expect("valid pattern"));
static YEAR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^~?\d{4}").expect("valid pattern"));
static CODE: LazyLock<Regex> = LazyLock::new(|| Regex::new("`([^`]+)`").expect("valid pattern"));
static STRONG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\*(.+?)\*\*").expect("valid pattern"));
static MARKDOWN_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\]\((?<target>[^)\s]+)\)").expect("valid pattern"));
static BACKTICK_RUN: LazyLock<Regex> = LazyLock::new(|| Regex::new("`+").expect("valid pattern"));
static NOT_SLUG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^\p{L}\p{Nd}\p{Pc} -]").expect("valid pattern"));

/// Stages the pages into `<out>/rust/`, and returns the problems that stopped a page or link
/// resolving. Empty means everything staged.
pub fn run(repo: &Repo, out: &Path) -> Vec<String> {
    let mut site = Site {
        repo,
        out: out.to_path_buf(),
        pages: HashMap::new(),
        roster: None,
        roster_anchors: HashMap::new(),
        copied: BTreeMap::new(),
        errors: Vec::new(),
    };
    site.stage();
    site.errors
}

struct Site<'a> {
    repo: &'a Repo,
    out: PathBuf,
    // Where each repository file lands on the site. A link to anything else goes to GitHub.
    pages: HashMap<String, String>,
    roster: Option<String>,
    // Each heading anchor in the roster, and the pages holding it, flagging a page's title.
    roster_anchors: HashMap<String, Vec<(String, bool)>>,
    // The files with a copy under raw/ on the site, which llms.txt and the copies link to
    // instead of GitHub, so an agent that can fetch only the site can read every one. Each has
    // the profiles whose house opinions it holds, which is none for most files.
    copied: BTreeMap<String, Vec<String>>,
    errors: Vec<String>,
}

struct Skill {
    name: String,
    purpose: String,
}

impl Site<'_> {
    #[expect(
        clippy::too_many_lines,
        reason = "a straight port of the .NET script's one function, kept in its order so the two can be compared"
    )]
    fn stage(&mut self) {
        // Other ecosystems might already have staged beside this one, so only this ecosystem's
        // directory is cleared.
        let cleared = self.out.join(ECOSYSTEM);
        if cleared.exists()
            && let Err(error) = fs::remove_dir_all(&cleared)
        {
            self.errors
                .push(format!("{}: can't be cleared: {error}", cleared.display()));
            return;
        }

        let Some(readme) = self.read(README) else {
            return;
        };

        let opinions = self.repo.opinions();
        let research = self.research_topics();
        let mut templates = self.repo.template_files();
        templates.sort_by_key(|path| path.to_lowercase());
        let diagrams = self.diagrams();

        // validate-sources holds each house opinion to its sources list, so a malformed one is
        // an error here too, rather than a page that shows the wrong thing. A profile ID names
        // a raw/<profile>/ directory, so profiles() leaves out an invalid one, such as ../..
        for path in opinions.iter().chain(&templates) {
            let Some(text) = self.read(path) else {
                continue;
            };
            let found = house::profiles(path, &text, &mut self.errors);
            self.copied.insert(path.clone(), found);
        }

        let profiles: Vec<String> = self
            .copied
            .values()
            .flatten()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();

        let skills: Vec<Skill> = SKILL_ROW
            .captures_iter(&readme)
            .map(|row| Skill {
                name: row["name"].to_string(),
                purpose: row["purpose"].to_string(),
            })
            .collect();

        // The files copied as they are. The loop above doesn't scan them for house opinions,
        // and the ones that describe house opinions show a block as an example, so removing
        // house blocks from a copy could take text that belongs in it. The skills link to the
        // agent instructions, so those are copied too.
        let mut verbatim: Vec<String> =
            skills.iter().map(|skill| skill_path(&skill.name)).collect();
        verbatim.extend([ROSTER, HOUSE_OPINIONS, AGENTS].map(String::from));
        verbatim.extend(self.repo.markdown_in(AGENT_INSTRUCTIONS));
        verbatim.retain(|path| self.repo.is_file(path));
        for path in &verbatim {
            self.copied.insert(path.clone(), Vec::new());
        }

        self.page(README, "index.md");
        self.page(HOUSE_OPINIONS, "house-opinions.md");
        if self.repo.is_file(ROSTER) {
            self.page(ROSTER, "awesome-humans/index.md");
        }
        if !opinions.is_empty() {
            self.page(repo::OPINIONS, "opinions/index.md");
        }
        if !templates.is_empty() {
            self.page(repo::TEMPLATES, "templates/index.md");
            self.page("templates/projects", "templates/index.md");
        }
        if !research.is_empty() {
            self.page(repo::RESEARCH, "research/index.md");
        }
        if !skills.is_empty() {
            self.page(repo::SKILLS, "skills/index.md");
        }

        for skill in &skills {
            let page = format!("skills/{}.md", skill.name);
            self.page(&format!("skills/{}", skill.name), &page);
            self.page(&skill_path(&skill.name), &page);
        }

        for path in &opinions {
            self.page(path, &format!("opinions/{}", file_name(path)));
        }

        for (path, _) in &research {
            self.page(path, &format!("research/{}", file_name(path)));
        }

        for template in &templates {
            self.page(template, &format!("templates/{}", template_page(template)));
        }

        for diagram in &diagrams {
            self.page(
                diagram,
                &format!("{}/{}", export_diagrams::OUTPUT, file_name(diagram)),
            );
        }

        // The diagrams that the generated pages, and the website repository's own pages, show.
        for diagram in &diagrams {
            if let Some(text) = self.read(diagram) {
                let page = self.pages[diagram].clone();
                self.write(&page, &text);
            }
        }

        let roster_pages = self.stage_roster();

        // The overview: README.md up to its layout section, which is about the repository
        // rather than the opinions.
        match readme.find("\n## Repository layout") {
            Some(layout) => {
                let overview =
                    self.rewrite(README, &format!("{}\n", &readme[..layout]), true, None);
                let page = self.pages[README].clone();
                self.write(&page, &overview);
            }
            None => self.errors.push(format!(
                "{README}: no '## Repository layout' section to end the overview at"
            )),
        }

        if let Some(text) = self.read(HOUSE_OPINIONS) {
            let staged = self.rewrite(HOUSE_OPINIONS, &text, false, None);
            let page = self.pages[HOUSE_OPINIONS].clone();
            self.write(&page, &staged);
        }

        for path in &opinions {
            self.stage_with_meta(path, true, "");
        }

        let scope: Vec<(String, String, String)> = SCOPE_BULLET
            .captures_iter(&readme)
            .map(|bullet| {
                (
                    bullet["title"].to_string(),
                    bullet["path"].to_string(),
                    bullet["summary"].to_string(),
                )
            })
            .collect();

        // An index of the opinions, in the order and with the summaries of the Scope list.
        if !opinions.is_empty() {
            let mut index = String::from(
                "# Opinions\n\nOne topic per page. Each opinion states its recommendation first, then the rationale, then its sources, and targets the latest stable release.\n\n",
            );
            for (title, path, summary) in &scope {
                let _ = writeln!(index, "- **[{title}]({}):**{summary}", file_name(path));
            }
            let page = self.pages[repo::OPINIONS].clone();
            self.write(&page, &index);
        }

        if !templates.is_empty() {
            self.stage_templates(&templates);
        }

        if !research.is_empty() {
            self.stage_research(&research);
        }

        if !skills.is_empty() {
            self.stage_skills(&skills);
        }

        // Copies of each opinion, template, and skill, the roster, HOUSE-OPINIONS.md, and the
        // agent instructions, for agents that can fetch the site but not GitHub. An opinion or
        // template gets one with every house block or region removed, under raw/none/, and one
        // per profile it holds, with only that profile's. Any other file gets the one copy,
        // unchanged. A Markdown file's copy ends in .md.txt, because the site renders every
        // .md file as a page, and its relative links point at the other copies.
        let mut taken: HashMap<String, String> = HashMap::new();
        for (path, found) in self.copied.clone() {
            let Some(text) = self.read(&path) else {
                continue;
            };
            let scanned = !verbatim.contains(&path);
            for profile in
                std::iter::once(None).chain(found.iter().map(|found| Some(found.as_str())))
            {
                // Two names that differ only by a leading dot share a copy, and the second
                // would replace the first without a word.
                let page = raw_page(&path, profile);
                if let Some(other) = taken.insert(page.clone(), path.clone()) {
                    self.errors.push(format!(
                        "{path}: its raw copy lands on {page}, as the copy of {other} does; rename one of them"
                    ));
                    continue;
                }

                let copy = if scanned {
                    house::for_profile(&path, &text, profile)
                } else {
                    text.clone()
                };
                let copy = if repo::has_extension(&path, "md") {
                    self.absolute_links(&path, &copy, profile)
                } else {
                    copy
                };
                self.write(&page, &copy);
            }
        }

        self.stage_llms(&scope, &templates, &skills, &profiles);
        self.stage_nav(
            &scope,
            &opinions,
            &templates,
            &research,
            &skills,
            roster_pages.as_ref(),
        );
        self.stage_sources();
    }

    // Records where a repository path lands, relative to rust/.
    fn page(&mut self, path: &str, page: &str) {
        self.pages
            .insert(path.to_string(), format!("{ECOSYSTEM}/{page}"));
    }

    fn diagrams(&self) -> Vec<String> {
        let Ok(entries) = fs::read_dir(self.repo.path(export_diagrams::OUTPUT)) else {
            return Vec::new();
        };

        let mut diagrams: Vec<String> = entries
            .filter_map(Result::ok)
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| repo::has_extension(name, "svg"))
            .map(|name| format!("{}/{name}", export_diagrams::OUTPUT))
            .collect();
        diagrams.sort();
        diagrams
    }

    // The roster, split so that no page holds all of it. The overview holds the admission
    // criteria and the shape of a source's notes, each roster table gets a page beside the
    // notes for the sources in it, each collapsed so the page reads as a list of sources, and
    // the decision log gets its own. A heading's anchor resolves to the page now holding it,
    // preferring the linking page when two pages share one. A source's collapsed notes keep
    // the anchor its heading had, and the site's open-details.js opens them when linked to.
    #[expect(
        clippy::too_many_lines,
        reason = "a straight port of the .NET script's one function, kept in its order so the two can be compared"
    )]
    fn stage_roster(&mut self) -> Option<BTreeMap<&'static str, String>> {
        if !self.repo.is_file(ROSTER) {
            return None;
        }

        let roster = self.read(ROSTER)?;
        let overview = self.pages[ROSTER].clone();
        let roster_pages: BTreeMap<&'static str, String> = [
            ("citable", format!("{ECOSYSTEM}/awesome-humans/citable.md")),
            (
                "watch-list",
                format!("{ECOSYSTEM}/awesome-humans/watch-list.md"),
            ),
            (
                "decision-log",
                format!("{ECOSYSTEM}/awesome-humans/decision-log.md"),
            ),
        ]
        .into_iter()
        .collect();

        // Pages in the order they're first filled, which decides the fallback when an anchor
        // is on more than one page.
        let sections = split_before(&roster, "## ");
        let mut texts: Vec<(String, String)> = vec![(overview.clone(), sections[0].clone())];
        let append = |texts: &mut Vec<(String, String)>, page: &str, text: &str| {
            if let Some((_, body)) = texts.iter_mut().find(|(name, _)| name == page) {
                body.push_str(text);
            } else {
                texts.push((page.to_string(), text.to_string()));
            }
        };

        for section in &sections[1..] {
            let first_line = section.lines().next().unwrap_or_default();
            if section.starts_with("## Admission criteria")
                || section.starts_with("## Named references")
                || section.starts_with("## Opted out")
            {
                append(&mut texts, &overview, section);
            } else if section.starts_with("## Citable") {
                append(
                    &mut texts,
                    &roster_pages["citable"],
                    &format!("{}\n## Source notes\n\n", &section[1..]),
                );
            } else if section.starts_with("## Watch list") {
                append(
                    &mut texts,
                    &roster_pages["watch-list"],
                    &format!("{}\n## Source notes\n\n", &section[1..]),
                );
            } else if section.starts_with("## Decision log") {
                append(&mut texts, &roster_pages["decision-log"], &section[1..]);
            } else if section.starts_with("## Source notes") {
                let notes = split_before(section, "### ");
                append(&mut texts, &overview, &notes[0]);
                for note in &notes[1..] {
                    let heading = note.lines().next().unwrap_or_default();
                    let bucket = if heading.starts_with("### `") && heading.contains("`: Citable") {
                        Some("citable")
                    } else if heading.starts_with("### `") && heading.contains("`: Watch list") {
                        Some("watch-list")
                    } else {
                        None
                    };

                    match bucket.map(|bucket| &roster_pages[bucket]) {
                        Some(page) if texts.iter().any(|(name, _)| name == page) => {
                            append(&mut texts, page, &collapsible(note));
                        }
                        _ => self.errors.push(format!(
                            "{ROSTER}: no roster page for the notes headed {heading}"
                        )),
                    }
                }
            } else {
                self.errors.push(format!(
                    "{ROSTER}: no site page for the section headed {first_line}; add one in xtask/src/build_site.rs"
                ));
            }
        }

        // Each ID in a roster table links to the source's notes on the same page, when it has
        // any.
        for bucket in ["citable", "watch-list"] {
            let page = &roster_pages[bucket];
            if let Some((_, text)) = texts.iter_mut().find(|(name, _)| name == page) {
                let original = text.clone();
                *text = TABLE_ID
                    .replace_all(&original, |row: &Captures| {
                        let id = &row["id"];
                        let note = Regex::new(&format!(
                            r#"(?m)^<details id="({}-(?:citable|watch-list)[^"]*)""#,
                            regex::escape(id)
                        ))
                        .ok()
                        .and_then(|pattern| {
                            pattern
                                .captures(&original)
                                .map(|found| found[1].to_string())
                        });
                        note.map_or_else(
                            || row[0].to_string(),
                            |anchor| format!("| [`{id}`](#{anchor}) |"),
                        )
                    })
                    .into_owned();
            }
        }

        for (page, text) in &texts {
            for heading in HEADING.captures_iter(text) {
                let slug = heading
                    .name("id")
                    .map_or_else(|| slug(&heading["text"]), |id| id.as_str().to_string());
                let title = heading
                    .name("hashes")
                    .is_some_and(|hashes| hashes.len() == 1);
                self.roster_anchors
                    .entry(slug)
                    .or_default()
                    .push((page.clone(), title));
            }
        }

        self.roster = Some(roster);
        for (page, text) in texts {
            let staged = self.rewrite(
                ROSTER,
                &format!("{}\n", text.trim_end_matches('\n')),
                false,
                Some(&page),
            );
            self.write(&page, &staged);
        }

        Some(roster_pages)
    }

    // An opinion or research topic, with the design system's OpinionMeta strip under its
    // title, from its frontmatter: a paragraph marked { .ah-meta }, as the staging contract
    // defines, then the lead, if any. An opinion's freshness chip turns stale past
    // audit-freshness's 180-day tolerance, counted from the day the site is staged, which the
    // weekly rebuild keeps current.
    fn stage_with_meta(&mut self, path: &str, freshness: bool, lead: &str) {
        let Some(text) = self.read(path) else { return };
        let meta = match frontmatter::parse(&text) {
            Ok(meta) => meta,
            Err(error) => {
                self.errors.push(format!("{path}: {error}"));
                return;
            }
        };

        let Some(reviewed) = meta
            .get("last-reviewed")
            .and_then(frontmatter::scalar)
            .and_then(|text| text.parse::<Date>().ok())
        else {
            self.errors
                .push(format!("{path}: last-reviewed isn't a YYYY-MM-DD date"));
            return;
        };

        let today = Timestamp::now().to_zoned_datetime(TimeZone::UTC).date();
        let age = reviewed.until(today).map_or(0, |span| span.get_days());
        let state = if age > OPINION_TOLERANCE {
            "stale"
        } else {
            "fresh"
        };
        let chip = if freshness {
            format!(" <span class=\"ah-chip ah-chip--{state} ah-chip--dot\">{state}</span>")
        } else {
            String::new()
        };

        let targets = values(meta.get("targets"))
            .iter()
            .map(|target| format!("`{target}`"))
            .collect::<Vec<_>>()
            .join(" ");
        let sources = values(meta.get("sources"))
            .iter()
            .map(|id| {
                format!(
                    "{}{{ .ah-chip{} }}",
                    self.source_link(path, id),
                    house_attribute(id)
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        let strip = format!(
            "{}{}{}\n{{ .ah-meta }}",
            meta_item("Targets", &targets),
            meta_item("Last reviewed", &format!("{reviewed}{chip}")),
            meta_item("Sources", &sources)
        );

        let staged = self.rewrite(path, &text, false, None);
        let Some(title) = staged.find("\n# ") else {
            self.errors
                .push(format!("{path}: no '# ' title to put the metadata under"));
            return;
        };

        let title_end = staged[title + 1..]
            .find('\n')
            .map_or(staged.len(), |offset| title + 1 + offset);
        let lead = if lead.is_empty() {
            String::new()
        } else {
            format!("\n\n{}", lead.trim_end_matches('\n'))
        };
        let page = self.pages[path].clone();
        self.write(
            &page,
            &format!(
                "{}\n\n{strip}{lead}{}",
                &staged[..title_end],
                &staged[title_end..]
            ),
        );
    }

    // The unresolved research topics, with their titles, ordered by title. resolve-research
    // deletes a topic once it's resolved, so every file here is open.
    fn research_topics(&mut self) -> Vec<(String, String)> {
        let mut topics = Vec::new();
        for path in self.repo.markdown_in(repo::RESEARCH) {
            // research/index.md would land on the research overview's page.
            if file_name(&path) == "index.md" {
                self.errors.push(format!(
                    "{path}: the research overview takes this page; rename the topic"
                ));
                continue;
            }
            let Some(text) = self.read(&path) else {
                continue;
            };
            match HEADING.captures_iter(&text).find(|heading| {
                heading
                    .name("hashes")
                    .is_some_and(|hashes| hashes.len() == 1)
            }) {
                Some(title) => topics.push((path, title["text"].to_string())),
                None => self.errors.push(format!("{path}: no '# ' title")),
            }
        }
        topics.sort_by_key(|(_, title)| title.to_lowercase());
        topics
    }

    // A page per research topic, with the same strip as an opinion but no freshness chip,
    // since a topic is evidence waiting on resolve-research rather than guidance to keep
    // current. An index lists them. llms.txt leaves them out, so agents follow only the
    // opinions.
    fn stage_research(&mut self, research: &[(String, String)]) {
        // Before the first opinion, there's no opinions page to point at.
        let follow = if self.pages.contains_key(repo::OPINIONS) {
            " Follow the [opinions](../opinions/index.md) instead."
        } else {
            ""
        };
        let lead = format!(
            "**Unresolved research.** This topic is evidence that `research-topic` gathered, waiting on `resolve-research`. It isn't an opinion, and it can cite unvetted material.{follow}"
        );
        for (path, _) in research {
            self.stage_with_meta(path, false, &lead);
        }

        let mut index = String::from(
            "# Research\n\nTopics under research: the evidence `research-topic` gathered, before `resolve-research` decides what becomes an opinion. Resolving a topic deletes it, so every topic here is still open. None of it is guidance yet.\n\n",
        );
        for (path, title) in research {
            let _ = writeln!(index, "- [{title}]({})", file_name(path));
        }
        let page = self.pages[repo::RESEARCH].clone();
        self.write(&page, &index);
    }

    // A page per template file, and an index of them.
    fn stage_templates(&mut self, templates: &[String]) {
        let mut index = String::from(
            "# Templates\n\nCopy-paste-ready files that encode the opinions. Copy one verbatim and trim it, rather than writing your own from scratch. Each opens with a comment header giving its targets, review date and sources.\n",
        );

        let mut groups: Vec<(String, Vec<&String>)> = Vec::new();
        for template in templates {
            let directory = parent(template).to_string();
            match groups.iter_mut().find(|(key, _)| *key == directory) {
                Some((_, members)) => members.push(template),
                None => groups.push((directory, vec![template])),
            }
        }

        for (directory, members) in groups {
            let _ = writeln!(index, "\n## `{directory}/`\n");
            for template in members {
                let _ = writeln!(
                    index,
                    "- [`{}`]({})",
                    file_name(template),
                    template_page(template)
                );

                let Some(text) = self.read(template) else {
                    continue;
                };
                let content = house::for_profile(template, &text, None);
                let fence = "`".repeat((longest_backtick_run(&text) + 1).max(3));
                let language = language(template);
                let mut variants = String::new();
                for profile in self.copied.get(template).cloned().unwrap_or_default() {
                    let _ = writeln!(
                        variants,
                        "\n/// house | {profile}\n\n**House:** With this house profile, the file reads:\n\n{fence}{language}\n{}\n{fence}\n\n///",
                        house::for_profile(template, &text, Some(&profile)).trim_end_matches('\n')
                    );
                }

                let page = self.pages[template].clone();
                let raw = relative(parent(&page), &raw_page(template, None));
                self.write(
                    &page,
                    &format!(
                        "# `{}`\n\n[View on GitHub]({}) · [Raw]({raw})\n\n{fence}{language}\n{}\n{fence}\n{variants}",
                        file_name(template),
                        github(template, false),
                        content.trim_end_matches('\n')
                    ),
                );
            }
        }

        let page = self.pages[repo::TEMPLATES].clone();
        self.write(&page, &index);
    }

    // A page per skill, its SKILL.md without the frontmatter, and an index of them in the
    // order of README.md's table, which validate-readme-index holds to the skills directory.
    fn stage_skills(&mut self, skills: &[Skill]) {
        let mut index = format!(
            "# Skills\n\nAgent skills that apply the opinions in a project, check a project against them, and keep them current. They follow the [Agent Skills specification](https://agentskills.io/specification), so any compliant agent can run them. To run one, clone the repository and ask your agent to run the skill by name. If your agent only discovers skills in a directory of its own, see [AGENTS.md: Skills]({}#skills).\n\n`review-project` checks your own project, and `research-topic` answers a Rust question from the opinions and their sources. The others maintain this repository.\n\n| Skill | Purpose |\n| --- | --- |\n",
            github("AGENTS.md", false)
        );

        for skill in skills {
            let path = skill_path(&skill.name);
            if !self.repo.is_file(&path) {
                self.errors.push(format!(
                    "{README}: the skills table lists {}, but {path} doesn't exist",
                    skill.name
                ));
                continue;
            }

            let _ = writeln!(
                index,
                "| [`{0}`]({0}.md) | {1} |",
                skill.name, skill.purpose
            );

            let Some(text) = self.read(&path) else {
                continue;
            };
            let Some(end) = text
                .strip_prefix("---\n")
                .and_then(|rest| rest.find("\n---\n"))
                .map(|end| end + 4)
            else {
                self.errors
                    .push(format!("{path}: no frontmatter block to strip"));
                continue;
            };

            let body = self.rewrite(&path, text[end + 5..].trim_start_matches('\n'), false, None);
            let title_end = body.find('\n').unwrap_or(body.len());
            let page = self.pages[&path].clone();
            self.write(
                &page,
                &format!(
                    "{}\n\n[View on GitHub]({}) · [Raw]({}){}",
                    &body[..title_end],
                    github(&path, false),
                    relative(parent(&page), &raw_page(&path, None)),
                    &body[title_end..]
                ),
            );
        }

        let page = self.pages[repo::SKILLS].clone();
        self.write(&page, &index);
    }

    // llms.txt, per llmstxt.org: a summary and links an agent can follow, to the site's raw
    // copies of each opinion, template, and skill rather than rendered pages. It describes this
    // ecosystem alone, at /rust/llms.txt, and the website repository's root llms.txt links to
    // it. It has no house opinions, and llms-<profile>.txt has one profile's.
    fn stage_llms(
        &mut self,
        scope: &[(String, String, String)],
        templates: &[String],
        skills: &[Skill],
        profiles: &[String],
    ) {
        let skill_links = skills
            .iter()
            .map(|skill| {
                format!(
                    "- [{}]({}): {}",
                    skill.name,
                    self.raw_link(&skill_path(&skill.name), None),
                    skill.purpose
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        for profile in
            std::iter::once(None).chain(profiles.iter().map(|profile| Some(profile.as_str())))
        {
            let opinion_links = scope
                .iter()
                .map(|(title, path, summary)| {
                    format!(
                        "- [{title}]({}): {}",
                        self.raw_link(path, profile),
                        summary.trim()
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            let template_links = templates
                .iter()
                .map(|template| {
                    format!(
                        "- [{}]({})",
                        template.strip_prefix("templates/").unwrap_or(template),
                        self.raw_link(template, profile)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            let house = match profile {
                None => "These files have no house opinions.".to_string(),
                Some(profile) => format!(
                    "These files include the {profile} house profile's opinions. Each is a block that opens with `/// house | {profile}`, starts with `**House:**`, and closes with `///`, or a template region between `house:{profile} begin` and `house:{profile} end` comments. A house opinion is its owner's preference from experience, not sourced community practice. Where it conflicts with the sourced text beside it, follow the house opinion. For the default, which has no house opinions, read [llms.txt]({SITE_URL}{ECOSYSTEM}/llms.txt)."
                ),
            };

            let title = profile.map_or_else(String::new, |profile| {
                format!(", with the {profile} house profile")
            });
            let mut llms = format!(
                "# Awesome Humans of {ECOSYSTEM_NAME}{title}\n\n> {LLMS_SUMMARY}\n\n{LLMS_READING}\n\n{house}\n"
            );
            for (heading, intro, links) in [
                (
                    "opinions",
                    "Follow these when writing or reviewing Rust code. Target the versions in each file's `targets:` frontmatter, never an older one.",
                    &opinion_links,
                ),
                (
                    "templates",
                    "Copy-paste-ready starting points that encode the opinions. Copy one verbatim and trim it.",
                    &template_links,
                ),
                (
                    "skills",
                    "Agent Skills procedures. Run follow-opinions to apply the opinions while writing Rust code, and review-project to check a Rust project against them. The others maintain the repository.",
                    &skill_links,
                ),
            ] {
                if !links.is_empty() {
                    let _ = writeln!(
                        llms,
                        "\n## {ECOSYSTEM_NAME} {heading}\n\n{intro}\n\n{links}"
                    );
                }
            }

            if profile.is_none() {
                llms += &profile_links(profiles);
            }

            // llmstxt.org puts the Optional section last, so an agent short of context can
            // drop the end.
            llms += &self.optional_links();

            let name = profile.map_or_else(
                || "llms.txt".to_string(),
                |profile| format!("llms-{profile}.txt"),
            );
            self.write(&format!("{ECOSYSTEM}/{name}"), &llms);
        }
    }

    // The llms.txt section an agent can skip: the files behind the opinions and skills, which
    // have no house opinions, so every profile's file links the same copies.
    fn optional_links(&self) -> String {
        let mut section = "\n## Optional\n\n".to_string();
        for (path, title, summary) in [
            (
                ROSTER,
                "Awesome humans",
                "the vetted sources the opinions cite, and the admission criteria",
            ),
            (
                HOUSE_OPINIONS,
                "House opinions",
                "the house profiles, and the record of every house opinion woven in",
            ),
            (
                AGENTS,
                "Agent instructions",
                "how an agent uses and maintains the repository, which the skills link to",
            ),
        ] {
            if path == HOUSE_OPINIONS || self.repo.is_file(path) {
                let _ = writeln!(
                    section,
                    "- [{title}]({}): {summary}",
                    self.raw_link(path, None)
                );
            }
        }
        section
    }

    // The navigation. This ecosystem's section goes to nav.json, with its paths relative to
    // rust/, and the website repository places it.
    fn stage_nav(
        &mut self,
        scope: &[(String, String, String)],
        opinions: &[String],
        templates: &[String],
        research: &[(String, String)],
        skills: &[Skill],
        roster_pages: Option<&BTreeMap<&'static str, String>>,
    ) {
        let mut opinion_entries = Vec::new();
        for (title, path, _) in scope {
            match self.pages.get(path) {
                Some(page) => opinion_entries.push(entry(title, page)),
                None => self.errors.push(format!(
                    "{README}: Scope links {path}, which isn't an opinion; run just validate-readme-index"
                )),
            }
        }

        if opinion_entries.len() != opinions.len() {
            self.errors.push(format!(
                "{README}: Scope lists {} opinions but opinions/ holds {}; run just validate-readme-index",
                opinion_entries.len(),
                opinions.len()
            ));
        }

        let mut nav = vec![entry("Overview", &self.pages[README])];
        if let Some(roster_pages) = roster_pages {
            nav.push(section(
                "Awesome humans",
                vec![
                    entry("Overview", &self.pages[ROSTER]),
                    entry("Citable", &roster_pages["citable"]),
                    entry("Watch list", &roster_pages["watch-list"]),
                    entry("Decision log", &roster_pages["decision-log"]),
                ],
            ));
        }
        if !opinions.is_empty() {
            let mut items = vec![entry("Overview", &self.pages[repo::OPINIONS])];
            items.extend(opinion_entries);
            nav.push(section("Opinions", items));
        }
        if !templates.is_empty() {
            let mut items = vec![entry("Overview", &self.pages[repo::TEMPLATES])];
            items.extend(
                templates
                    .iter()
                    .map(|template| entry(file_name(template), &self.pages[template])),
            );
            nav.push(section("Templates", items));
        }
        if !research.is_empty() {
            let mut items = vec![entry("Overview", &self.pages[repo::RESEARCH])];
            items.extend(
                research
                    .iter()
                    .map(|(path, title)| entry(&nav_title(title), &self.pages[path])),
            );
            nav.push(section("Research", items));
        }
        if !skills.is_empty() {
            let mut items = vec![entry("Overview", &self.pages[repo::SKILLS])];
            items.extend(skills.iter().filter_map(|skill| {
                self.pages
                    .get(&skill_path(&skill.name))
                    .map(|page| entry(&skill.name, page))
            }));
            nav.push(section("Skills", items));
        }
        nav.push(entry("House opinions", &self.pages[HOUSE_OPINIONS]));

        let document = json!({
            "$schema": format!("{SCHEMA_BASE}/nav.schema.json"),
            "version": 1,
            "nav": nav,
        });
        self.write_json(&format!("{ECOSYSTEM}/nav.json"), &document);
    }

    // The roster export. An opted-out source is exported too, so intake can turn away a
    // suggestion of it. Before the roster exists, the export is empty.
    fn stage_sources(&mut self) {
        let mut sources: Vec<Value> = Vec::new();
        if let Some(roster) = self.roster.clone() {
            let sections = split_before(&roster, "## ");
            for (bucket, heading) in [
                ("citable", "## Citable"),
                ("watch-list", "## Watch list"),
                ("opted-out", "## Opted out"),
            ] {
                let Some(text) = sections.iter().find(|section| section.starts_with(heading))
                else {
                    self.errors
                        .push(format!("{ROSTER}: no '{heading}' section to export"));
                    continue;
                };

                for row in ROSTER_ROW.captures_iter(text) {
                    if let Some(source) = self.source(&row, bucket, &roster) {
                        sources.push(source);
                    }
                }
            }
        }

        sources.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        let document = json!({
            "$schema": format!("{SCHEMA_BASE}/sources.schema.json"),
            "version": 1,
            "sources": sources,
        });
        self.write_json(&format!("{ECOSYSTEM}/sources.json"), &document);
    }

    // One source in sources.json, from its roster row and the heading of its notes, which
    // holds the marking. A person's name is the start of their Source link text, before
    // ": ", " (", or "'s". A publication's is the whole link text. Since keeps only its
    // leading year, so "~2016 (old docs)" exports as "~2016".
    fn source(&mut self, row: &Captures, bucket: &str, roster: &str) -> Option<Value> {
        let id = &row["id"];
        let kind = &row["kind"];
        let Some(link) = FIRST_LINK.captures(&row["source"]) else {
            self.errors
                .push(format!("{ROSTER}: the Source cell for '{id}' has no link"));
            return None;
        };

        let text = &link["text"];
        let name = if kind == "person" {
            PERSON_NAME.captures(text).map_or(text, |found| {
                found.name("name").map_or(text, |name| name.as_str())
            })
        } else {
            text
        };

        let since = row["since"].trim();
        let year = YEAR.find(since).map(|year| year.as_str().to_string());
        if year.is_none() && since != "—" {
            self.errors.push(format!(
                "{ROSTER}: the Since cell for '{id}' is '{since}', not a year, a ~year, or —"
            ));
        }

        let heading = Regex::new(&format!(r"(?m)^### `{}`:(?<rest>.*)$", regex::escape(id)))
            .ok()
            .and_then(|pattern| {
                pattern
                    .captures(roster)
                    .map(|found| found["rest"].to_string())
            })
            .unwrap_or_default();
        let marking = if heading.contains("**Corroborate.**") {
            Some("corroborate")
        } else if heading.contains("**Discovery-only.**") {
            Some("discovery-only")
        } else {
            None
        };

        let mut source = Map::new();
        source.insert("id".into(), id.into());
        source.insert("name".into(), name.into());
        source.insert("kind".into(), kind.into());
        source.insert("url".into(), link["url"].into());
        source.insert("standing".into(), bucket.into());
        if let Some(marking) = marking {
            source.insert("marking".into(), marking.into());
        }
        source.insert("focus".into(), row["focus"].trim().into());
        source.insert("since".into(), year.map_or(Value::Null, Value::String));
        Some(Value::Object(source))
    }

    // A source ID links to its notes section in the roster, or to the Citable table when it
    // has nothing to record and so no section, and a house profile links to the house
    // opinions.
    fn source_link(&mut self, path: &str, id: &str) -> String {
        let from = parent(path).to_string();
        let page = self.pages[path].clone();
        if house::profile_of(id).is_some() {
            let target = self.target(path, &page, &relative(&from, HOUSE_OPINIONS), false);
            return format!("[{id}]({target})");
        }

        let anchor = self
            .roster
            .as_deref()
            .and_then(|roster| {
                Regex::new(&format!(r"(?m)^### (`{}`.*)$", regex::escape(id)))
                    .ok()?
                    .captures(roster)
                    .map(|heading| slug(&heading[1]))
            })
            .unwrap_or_else(|| CITABLE_ANCHOR.to_string());
        let target = self.target(
            path,
            &page,
            &format!("{}#{anchor}", relative(&from, ROSTER)),
            false,
        );
        format!("[{id}]({target})")
    }

    // Rewrites every relative link in a Markdown file, read from its repository path, to point
    // from its page on the site, and escapes a heading's trailing #, which GitHub keeps but
    // Zensical reads as a closing sequence. Fenced code blocks are left alone.
    fn rewrite(
        &mut self,
        source: &str,
        text: &str,
        anchors_to_github: bool,
        from: Option<&str>,
    ) -> String {
        let from = from.map_or_else(|| self.pages[source].clone(), String::from);
        let mut fenced = false;
        let mut lines = Vec::new();
        for line in text.split('\n') {
            if line.trim_start().starts_with("```") {
                fenced = !fenced;
            }

            if fenced {
                lines.push(line.to_string());
                continue;
            }

            let line = HEADING_TRAILING_HASH.replace(line, |heading: &Captures| {
                format!(
                    "{}{}",
                    &heading["text"],
                    heading["hashes"].replace('#', "\\#")
                )
            });
            let line = MARKDOWN_LINK
                .replace_all(&line, |link: &Captures| {
                    format!(
                        "]({})",
                        self.target(source, &from, &link["target"], anchors_to_github)
                    )
                })
                .into_owned();
            lines.push(line);
        }

        lines.join("\n")
    }

    fn target(
        &mut self,
        source: &str,
        from: &str,
        target: &str,
        anchors_to_github: bool,
    ) -> String {
        if target.contains("://") || target.starts_with("mailto:") {
            return target.to_string();
        }

        let (path, anchor) = target
            .find('#')
            .map_or((target, ""), |hash| (&target[..hash], &target[hash..]));
        if path.is_empty() && anchors_to_github {
            return github(source, false) + anchor;
        }

        let resolved = if path.is_empty() {
            source.to_string()
        } else {
            combine(parent(source), path)
        };

        if resolved == ROSTER && anchor.len() > 1 {
            let Some(found) = self.roster_anchors.get(&anchor[1..]) else {
                self.errors.push(format!(
                    "{source}: link to {target} names no heading in {ROSTER}"
                ));
                return target.to_string();
            };

            let (page, title) = found
                .iter()
                .find(|(page, _)| page == from)
                .unwrap_or(&found[0]);
            let relative = if page == from {
                String::new()
            } else {
                relative(parent(from), page)
            };
            return match (title, relative.is_empty()) {
                (true, true) => "#".to_string(),
                (true, false) => relative,
                (false, _) => relative + anchor,
            };
        }

        if path.is_empty() {
            return target.to_string();
        }

        if let Some(page) = self.pages.get(&resolved) {
            return relative(parent(from), page) + anchor;
        }

        if self.repo.exists(&resolved) {
            return github(&resolved, self.repo.is_dir(&resolved)) + anchor;
        }

        self.errors.push(format!(
            "{source}: link to {target} resolves to {resolved}, which doesn't exist"
        ));
        target.to_string()
    }

    // A raw copy is served from rust/raw/<profile>/, where a relative link to another file in
    // the repository leads nowhere, so each one becomes absolute: the file raw_link gives, or
    // the GitHub tree for a directory with no page on the site. A link to a shared convention
    // on GitHub becomes its copy on the site. Fenced code blocks are left
    // alone.
    fn absolute_links(&self, source: &str, text: &str, profile: Option<&str>) -> String {
        let mut fenced = false;
        let lines: Vec<String> = text
            .split('\n')
            .map(|line| {
                if line.trim_start().starts_with("```") {
                    fenced = !fenced;
                }
                if fenced {
                    return line.to_string();
                }

                MARKDOWN_LINK
                    .replace_all(line, |link: &Captures| {
                        let target = &link["target"];
                        let (path, anchor) = target
                            .find('#')
                            .map_or((target, ""), |hash| (&target[..hash], &target[hash..]));
                        if let Some(convention) = path
                            .strip_prefix(CONVENTION_SOURCE)
                            .and_then(|file| file.strip_suffix(".md"))
                            && !convention.contains('/')
                        {
                            return format!("]({CONVENTION_COPY}{convention}.md.txt{anchor})");
                        }
                        if target.contains("://")
                            || target.starts_with("mailto:")
                            || target.starts_with('#')
                        {
                            return link[0].to_string();
                        }

                        let resolved = combine(parent(source), path);
                        // A link shown as an example, such as one to opinions/<file>.md,
                        // names no file, and stays as written.
                        if !self.repo.exists(&resolved) {
                            return link[0].to_string();
                        }
                        let url =
                            if self.repo.is_dir(&resolved) && !self.pages.contains_key(&resolved) {
                                github(&resolved, true)
                            } else {
                                self.raw_link(&resolved, profile)
                            };
                        format!("]({url}{anchor})")
                    })
                    .into_owned()
            })
            .collect();
        lines.join("\n")
    }

    // The raw file an agent reads for a profile, or for the default with None. A file the site
    // copies is this profile's copy, or the default copy when none of its house opinions are
    // this profile's, and a file without house opinions has only the default. A file with a
    // page on the site and no copy, such as README.md or a research topic, is that page, so
    // the agent stays on the site. Any other file is GitHub's raw copy.
    fn raw_link(&self, path: &str, profile: Option<&str>) -> String {
        let Some(found) = self.copied.get(path) else {
            return self
                .pages
                .get(path)
                .map_or_else(|| raw(path), |page| site_url(page));
        };
        let profile = profile.filter(|profile| found.iter().any(|other| other == profile));
        format!("{SITE_URL}{}", raw_page(path, profile))
    }

    fn read(&mut self, path: &str) -> Option<String> {
        match self.repo.read(path) {
            Ok(text) => Some(text),
            Err(error) => {
                self.errors.push(format!("{path}: can't be read: {error}"));
                None
            }
        }
    }

    fn write(&mut self, page: &str, content: &str) {
        let path = self.out.join(page);
        let result = path
            .parent()
            .map_or(Ok(()), fs::create_dir_all)
            .and_then(|()| fs::write(&path, content));
        if let Err(error) = result {
            self.errors
                .push(format!("{}: can't be written: {error}", path.display()));
        }
    }

    fn write_json(&mut self, page: &str, document: &Value) {
        match serde_json::to_string_pretty(document) {
            Ok(text) => self.write(page, &(text + "\n")),
            Err(error) => self
                .errors
                .push(format!("{page}: can't be serialised: {error}")),
        }
    }
}

// A navigation entry, with the ecosystem's own prefix dropped from its page.
fn entry(title: &str, page: &str) -> Value {
    let prefix = format!("{ECOSYSTEM}/");
    let mut item = Map::new();
    item.insert(
        title.into(),
        page.strip_prefix(&prefix).unwrap_or(page).into(),
    );
    Value::Object(item)
}

fn section(title: &str, items: Vec<Value>) -> Value {
    let mut item = Map::new();
    item.insert(title.into(), Value::Array(items));
    Value::Object(item)
}

// A research topic's sidebar label: its title up to any ": ", which names the topic, without
// the ecosystem the section already sets, such as "in Rust code", or the backticks a label
// can't render.
fn nav_title(title: &str) -> String {
    let mut topic = title.split(": ").next().unwrap_or(title);
    for context in [
        format!(" in {ECOSYSTEM_NAME}"),
        format!(" for {ECOSYSTEM_NAME}"),
    ] {
        topic = topic.split(context.as_str()).next().unwrap_or(topic);
    }
    topic.replace('`', "")
}

// One labelled group in an opinion's metadata strip. Markdown inside the span still renders.
fn meta_item(label: &str, values: &str) -> String {
    format!("<span class=\"ah-meta__item\"><span class=\"ah-label\">{label}</span> {values}</span>")
}

// A house profile's chip names the profile, so the site shows it only with its blocks.
fn house_attribute(id: &str) -> String {
    house::profile_of(id).map_or_else(String::new, |profile| format!(" data-house=\"{profile}\""))
}

// A metadata value as its items: a list's entries, or a single scalar.
fn values(value: Option<&serde_yaml_ng::Value>) -> Vec<String> {
    match value {
        Some(serde_yaml_ng::Value::Sequence(items)) => {
            items.iter().filter_map(frontmatter::scalar).collect()
        }
        Some(value) => frontmatter::scalar(value).into_iter().collect(),
        None => Vec::new(),
    }
}

// Where a file's copy for a profile, or the default with None, lands under raw/. Zensical
// skips dotfiles, so a leading dot is dropped from each part of the path, as on a template's
// page. A Markdown file's copy ends in .md.txt.
fn raw_page(path: &str, profile: Option<&str>) -> String {
    let name = path.replace("/.", "/");
    let suffix = if repo::has_extension(path, "md") {
        ".txt"
    } else {
        ""
    };
    format!(
        "{ECOSYSTEM}/raw/{}/{name}{suffix}",
        profile.unwrap_or(NO_PROFILE)
    )
}

// The address the site serves a staged page at: a directory per page, and the directory
// itself for an index.
fn site_url(page: &str) -> String {
    match page.strip_suffix(".md") {
        None => format!("{SITE_URL}{page}"),
        Some(stem) => format!("{SITE_URL}{}/", stem.strip_suffix("/index").unwrap_or(stem)),
    }
}

fn github(path: &str, is_directory: bool) -> String {
    let kind = if is_directory { "tree" } else { "blob" };
    format!("https://github.com/{REPOSITORY}/{kind}/main/{path}")
}

fn raw(path: &str) -> String {
    format!("https://raw.githubusercontent.com/{REPOSITORY}/main/{path}")
}

fn skill_path(name: &str) -> String {
    format!("{}/{name}/SKILL.md", repo::SKILLS)
}

// Zensical skips dotfiles, so .editorconfig's page drops the leading dot, as does .cargo/'s.
fn template_page(template: &str) -> String {
    let relative = template.strip_prefix("templates/").unwrap_or(template);
    format!("{}.md", relative.replace("/.", "/").trim_start_matches('.'))
}

fn language(path: &str) -> &'static str {
    let name = file_name(path);
    match name.rsplit_once('.').map_or("", |(_, extension)| extension) {
        "toml" => "toml",
        "rs" => "rust",
        "json" => "json",
        "editorconfig" => "ini",
        _ => "text",
    }
}

// Turns a source's notes, headed '### `id`: Citable', into a collapsed block whose id is the
// anchor the heading had, so links to it still land.
fn collapsible(note: &str) -> String {
    let (heading, body) = note.split_once('\n').unwrap_or((note, ""));
    let heading = &heading[4..];
    let summary = STRONG
        .replace_all(
            &CODE.replace_all(&html_encode(heading), "<code>$1</code>"),
            "<strong>$1</strong>",
        )
        .into_owned();
    format!(
        "<details id=\"{}\" markdown=\"1\">\n<summary>{summary}</summary>\n\n{}\n\n</details>\n\n",
        slug(heading),
        body.trim_matches('\n')
    )
}

fn html_encode(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

// Splits Markdown before each line that starts with the prefix, keeping the text ahead of the
// first such line as the first part.
fn split_before(text: &str, prefix: &str) -> Vec<String> {
    let mut parts = vec![String::new()];
    for line in text.split_inclusive('\n') {
        if line.starts_with(prefix) {
            parts.push(String::new());
        }
        if let Some(part) = parts.last_mut() {
            part.push_str(line);
        }
    }
    parts
}

fn longest_backtick_run(text: &str) -> usize {
    BACKTICK_RUN
        .find_iter(text)
        .map(|run| run.len())
        .max()
        .unwrap_or(0)
}

/// GitHub's heading anchors, which the rest of the repository already links by: lower case,
/// punctuation dropped, spaces to hyphens.
fn slug(heading: &str) -> String {
    NOT_SLUG
        .replace_all(&heading.to_lowercase(), "")
        .replace(' ', "-")
}

fn file_name(path: &str) -> &str {
    path.rsplit_once('/').map_or(path, |(_, name)| name)
}

fn parent(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(parent, _)| parent)
}

// A repository-relative path with '.' and '..' resolved, and no trailing slash.
fn normalise(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." if parts.last().is_some_and(|last| *last != "..") => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    parts.join("/")
}

fn combine(directory: &str, path: &str) -> String {
    if directory.is_empty() {
        normalise(path)
    } else {
        normalise(&format!("{directory}/{path}"))
    }
}

// The path to `to` from the directory `from`, both repository-relative.
fn relative(from: &str, to: &str) -> String {
    let from = normalise(from);
    let to = normalise(to);
    let from: Vec<&str> = from.split('/').filter(|part| !part.is_empty()).collect();
    let to: Vec<&str> = to.split('/').filter(|part| !part.is_empty()).collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<&str> = vec![".."; from.len() - common];
    parts.extend(&to[common..]);
    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

// The default llms.txt lists each profile's file as a link of its own, because a fetch tool
// that summarises a page paraphrases prose and drops the URLs in it.
fn profile_links(profiles: &[String]) -> String {
    if profiles.is_empty() {
        return String::new();
    }
    let mut section = "\n## House profiles\n\nFollow a profile only when the user chooses one, by reading its file instead of this one.\n\n".to_string();
    for other in profiles {
        let _ = writeln!(
            section,
            "- [{other}]({SITE_URL}{ECOSYSTEM}/llms-{other}.txt): these opinions and templates, with that profile's house opinions"
        );
    }
    section
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::Fixture;

    const README_TEXT: &str = "# Awesome Humans of Rust\n\nSee [house opinions](HOUSE-OPINIONS.md), [metadata](AGENTS.md#metadata), and [scope](#scope).\n\n## Scope\n\n- **[Rust](opinions/rust.md):** the language\n\n## Repository layout\n\n[Licence](LICENSE)\n\n## Maintenance via skills\n\n| Skill | Purpose |\n| --- | --- |\n| [`vet-source`](skills/vet-source/SKILL.md) | Vet a source |\n";

    const ROSTER_TEXT: &str = "# Awesome humans\n\n## Admission criteria\n\nTwo years.\n\n## Citable: quotable in opinions and templates\n\nThe table lists 2 sources: 1 people and 1 publications.\n\n| ID | Kind | Source | Focus | Since |\n| --- | --- | --- | --- | --- |\n| `matklad` | person | [Alex Kladov: Blog](https://matklad.github.io/) | Tooling | 2017 |\n| `rust-docs` | publication | [The Rust Programming Language](https://doc.rust-lang.org/) | Language | ~2015 (book) |\n\n## Watch list\n\n| ID | Kind | Source | Focus | Since |\n| --- | --- | --- | --- | --- |\n\n## Named references\n\nRFCs.\n\n## Opted out\n\n| ID | Kind | Source | Focus | Since |\n| --- | --- | --- | --- | --- |\n\n## Source notes\n\nEach source's notes.\n\n### `matklad`: Citable, **Corroborate.**\n\nWrites about tooling.\n\n## Decision log\n\n- 2026-10-18: admitted both.\n";

    const OPINION: &str = "---\ntargets: [rust-1.99, edition-2024]\nlast-reviewed: 2026-10-14\nsources: [house:simon-mckenna, matklad, rust-docs]\n---\n\n# Rust\n\nUse edition 2024 ([matklad](../AWESOME-HUMANS.md#matklad-citable-corroborate)), and start from [the config](../templates/.cargo/config.toml).\n\n/// house | simon-mckenna\n\n**House:** Deny warnings.\n\n///\n\nMore.\n";

    fn staged(fixture: &Fixture, page: &str) -> String {
        fs::read_to_string(fixture.path().join("out").join(page)).expect(page)
    }

    #[test]
    fn a_repository_before_its_content_stages_an_overview_and_house_opinions() {
        let readme = "# Awesome Humans of Rust\n\nSee [house opinions](HOUSE-OPINIONS.md).\n\n## Scope\n\nNone yet.\n\n## Repository layout\n";
        let fixture = Fixture::new().file("README.md", readme).file(
            "HOUSE-OPINIONS.md",
            "# House opinions\n\nSee [Inbox](#inbox).\n\n## Inbox\n",
        );
        let errors = run(&fixture.repo(), &fixture.path().join("out"));
        assert_eq!(errors, Vec::<String>::new());
        assert!(staged(&fixture, "rust/index.md").contains("[house opinions](house-opinions.md)"));
        assert!(staged(&fixture, "rust/llms.txt").contains("These files have no house opinions."));
        assert!(staged(&fixture, "rust/llms.txt").contains(LLMS_READING));
        let nav: Value = serde_json::from_str(&staged(&fixture, "rust/nav.json")).expect("JSON");
        assert_eq!(
            nav["nav"],
            json!([{"Overview": "index.md"}, {"House opinions": "house-opinions.md"}])
        );
        let sources: Value =
            serde_json::from_str(&staged(&fixture, "rust/sources.json")).expect("JSON");
        assert_eq!(sources["sources"], json!([]));
    }

    #[test]
    #[expect(clippy::too_many_lines, reason = "one fixture checked page by page")]
    fn a_full_repository_stages_every_part() {
        let fixture = Fixture::new()
            .file("README.md", README_TEXT)
            .file("AGENTS.md", "# Agents\n\nSee [worktrees](agents/worktrees.md).\n")
            .file("agents/worktrees.md", "# Worktrees\n\nSee [the index](../README.md#scope) and [the skills](../skills). A bullet reads `[Title](opinions/<file>.md)`. Follow [Writing style](https://github.com/awesome-humans/website/blob/main/pages/conventions/writing-style.md#voice).\n")
            .file("LICENSE", "Apache")
            .file("HOUSE-OPINIONS.md", "# House opinions\n\nA block looks like this:\n\n/// house | simon-mckenna\n\n**House:** An example.\n\n///\n")
            .file(ROSTER, ROSTER_TEXT)
            .file("opinions/rust.md", OPINION)
            .file("research/unsafe.md", "---\ntargets: [rust-1.99]\nlast-reviewed: 2026-10-19\nsources: [matklad]\n---\n\n# `unsafe` code in Rust: when and how\n\nSee [the opinion](../opinions/rust.md).\n")
            .file("skills/vet-source/SKILL.md", "---\nname: vet-source\n---\n\n# Vet a source\n\nSee [the roster](../../AWESOME-HUMANS.md), [the rules](../../AGENTS.md#skills), [the topic](../../research/unsafe.md), and [the licence](../../LICENSE).\n")
            .file(
                "templates/.cargo/config.toml",
                "# awesome-humans/rust template | sources: house:simon-mckenna, rust-docs\n[build]\n# house:simon-mckenna begin\n# **House:** Deny warnings.\nrustflags = [\"-Dwarnings\"]\n# house:simon-mckenna end\n",
            )
            .file("templates/Cargo.toml", "# awesome-humans/rust template | sources: rust-docs\n[workspace]\n");
        let errors = run(&fixture.repo(), &fixture.path().join("out"));
        assert_eq!(errors, Vec::<String>::new());

        let overview = staged(&fixture, "rust/index.md");
        assert!(
            overview.contains(
                "[metadata](https://github.com/awesome-humans/rust/blob/main/AGENTS.md#metadata)"
            ),
            "{overview}"
        );
        assert!(
            overview.contains(
                "[scope](https://github.com/awesome-humans/rust/blob/main/README.md#scope)"
            ),
            "{overview}"
        );
        assert!(!overview.contains("Licence"), "{overview}");

        let research = staged(&fixture, "rust/research/unsafe.md");
        assert!(
            research.contains("<span class=\"ah-label\">Last reviewed</span> 2026-10-19</span>"),
            "{research}"
        );
        assert!(
            research.contains("{ .ah-meta }\n\n**Unresolved research.**"),
            "{research}"
        );
        assert!(
            research.contains("[the opinion](../opinions/rust.md)"),
            "{research}"
        );
        assert!(
            staged(&fixture, "rust/research/index.md")
                .contains("- [`unsafe` code in Rust: when and how](unsafe.md)\n")
        );

        let opinion = staged(&fixture, "rust/opinions/rust.md");
        assert!(opinion.contains("# Rust\n\n<span class=\"ah-meta__item\"><span class=\"ah-label\">Targets</span> `rust-1.99` `edition-2024`</span>"), "{opinion}");
        assert!(opinion.contains("[house:simon-mckenna](../house-opinions.md){ .ah-chip data-house=\"simon-mckenna\" }"), "{opinion}");
        assert!(
            opinion.contains(
                "[matklad](../awesome-humans/citable.md#matklad-citable-corroborate){ .ah-chip }"
            ),
            "{opinion}"
        );
        assert!(
            opinion.contains("[rust-docs](../awesome-humans/citable.md){ .ah-chip }"),
            "{opinion}"
        );
        assert!(
            opinion
                .contains("([matklad](../awesome-humans/citable.md#matklad-citable-corroborate))"),
            "{opinion}"
        );
        assert!(opinion.contains("/// house | simon-mckenna"), "{opinion}");

        let citable = staged(&fixture, "rust/awesome-humans/citable.md");
        assert!(
            citable.starts_with("# Citable: quotable in opinions and templates\n"),
            "{citable}"
        );
        assert!(
            citable.contains("| [`matklad`](#matklad-citable-corroborate) |"),
            "{citable}"
        );
        assert!(citable.contains("<details id=\"matklad-citable-corroborate\" markdown=\"1\">\n<summary><code>matklad</code>: Citable, <strong>Corroborate.</strong></summary>"), "{citable}");
        let overview = staged(&fixture, "rust/awesome-humans/index.md");
        assert!(overview.contains("## Admission criteria"), "{overview}");
        assert!(overview.contains("## Named references"), "{overview}");
        assert!(
            staged(&fixture, "rust/awesome-humans/decision-log.md").starts_with("# Decision log\n")
        );

        let config = staged(&fixture, "rust/templates/cargo/config.toml.md");
        assert!(
            config.contains(
                "```toml\n# awesome-humans/rust template | sources: rust-docs\n[build]\n```\n"
            ),
            "{config}"
        );
        assert!(
            config.contains(
                "/// house | simon-mckenna\n\n**House:** With this house profile, the file reads:"
            ),
            "{config}"
        );
        assert!(
            config.contains("[Raw](../../raw/none/templates/cargo/config.toml)"),
            "{config}"
        );
        assert_eq!(
            staged(
                &fixture,
                "rust/raw/simon-mckenna/templates/cargo/config.toml"
            ),
            "# awesome-humans/rust template | sources: house:simon-mckenna, rust-docs\n[build]\n# **House:** Deny warnings.\nrustflags = [\"-Dwarnings\"]\n"
        );
        let copy = staged(&fixture, "rust/raw/none/opinions/rust.md.txt");
        assert!(!copy.contains("House"), "{copy}");
        assert!(copy.contains("[matklad](https://awesome-humans.net/rust/raw/none/AWESOME-HUMANS.md.txt#matklad-citable-corroborate)"), "{copy}");
        assert!(
            copy.contains(
                "[the config](https://awesome-humans.net/rust/raw/none/templates/cargo/config.toml)"
            ),
            "{copy}"
        );
        let copy = staged(&fixture, "rust/raw/simon-mckenna/opinions/rust.md.txt");
        assert!(copy.contains("[the config](https://awesome-humans.net/rust/raw/simon-mckenna/templates/cargo/config.toml)"), "{copy}");

        let skill = staged(&fixture, "rust/skills/vet-source.md");
        assert!(skill.starts_with("# Vet a source\n\n[View on GitHub](https://github.com/awesome-humans/rust/blob/main/skills/vet-source/SKILL.md) · [Raw](../raw/none/skills/vet-source/SKILL.md.txt)"), "{skill}");
        assert!(
            skill.contains("[the roster](../awesome-humans/index.md)"),
            "{skill}"
        );

        let llms = staged(&fixture, "rust/llms.txt");
        assert!(llms.contains("- [Rust](https://awesome-humans.net/rust/raw/none/opinions/rust.md.txt): the language"), "{llms}");
        assert!(
            llms.contains(
                "- [Cargo.toml](https://awesome-humans.net/rust/raw/none/templates/Cargo.toml)"
            ),
            "{llms}"
        );
        assert!(llms.contains("- [vet-source](https://awesome-humans.net/rust/raw/none/skills/vet-source/SKILL.md.txt): Vet a source"), "{llms}");
        assert!(llms.contains("- [Awesome humans](https://awesome-humans.net/rust/raw/none/AWESOME-HUMANS.md.txt): "), "{llms}");
        assert!(llms.contains("- [House opinions](https://awesome-humans.net/rust/raw/none/HOUSE-OPINIONS.md.txt): "), "{llms}");
        assert!(!llms.contains("raw.githubusercontent.com"), "{llms}");
        assert_eq!(
            staged(&fixture, "rust/raw/none/templates/Cargo.toml"),
            "# awesome-humans/rust template | sources: rust-docs\n[workspace]\n"
        );
        let copy = staged(&fixture, "rust/raw/none/skills/vet-source/SKILL.md.txt");
        assert!(copy.starts_with("---\nname: vet-source\n---\n"), "{copy}");
        assert!(
            copy.contains(
                "[the roster](https://awesome-humans.net/rust/raw/none/AWESOME-HUMANS.md.txt)"
            ),
            "{copy}"
        );
        assert!(
            copy.contains(
                "[the rules](https://awesome-humans.net/rust/raw/none/AGENTS.md.txt#skills)"
            ),
            "{copy}"
        );
        assert!(
            copy.contains("[the topic](https://awesome-humans.net/rust/research/unsafe/)"),
            "{copy}"
        );
        assert!(
            copy.contains(
                "[the licence](https://raw.githubusercontent.com/awesome-humans/rust/main/LICENSE)"
            ),
            "{copy}"
        );
        assert!(
            staged(&fixture, "rust/raw/none/AWESOME-HUMANS.md.txt")
                .starts_with("# Awesome humans\n")
        );
        assert!(
            llms.contains(
                "- [Agent instructions](https://awesome-humans.net/rust/raw/none/AGENTS.md.txt): "
            ),
            "{llms}"
        );
        assert_eq!(
            staged(&fixture, "rust/raw/none/AGENTS.md.txt"),
            "# Agents\n\nSee [worktrees](https://awesome-humans.net/rust/raw/none/agents/worktrees.md.txt).\n"
        );
        assert_eq!(
            staged(&fixture, "rust/raw/none/agents/worktrees.md.txt"),
            "# Worktrees\n\nSee [the index](https://awesome-humans.net/rust/#scope) and [the skills](https://awesome-humans.net/rust/skills/). A bullet reads `[Title](opinions/<file>.md)`. Follow [Writing style](https://awesome-humans.net/raw/none/conventions/writing-style.md.txt#voice).\n"
        );
        // A file that isn't scanned for house opinions keeps the block it shows as an example.
        assert!(
            staged(&fixture, "rust/raw/none/HOUSE-OPINIONS.md.txt")
                .contains("/// house | simon-mckenna\n\n**House:** An example.\n\n///\n")
        );
        assert!(staged(&fixture, "rust/llms-simon-mckenna.txt").contains(
            "- [Cargo.toml](https://awesome-humans.net/rust/raw/none/templates/Cargo.toml)"
        ));
        assert!(
            llms.contains(
                "\n## House profiles\n\nFollow a profile only when the user chooses one, by reading its file instead of this one.\n\n- [simon-mckenna](https://awesome-humans.net/rust/llms-simon-mckenna.txt): these opinions and templates, with that profile's house opinions\n"
            ),
            "{llms}"
        );
        assert!(
            staged(&fixture, "rust/llms-simon-mckenna.txt")
                .contains("raw/simon-mckenna/opinions/rust.md.txt")
        );
        // Optional is the last section, after House profiles.
        assert!(
            llms.find("\n## House profiles\n") < llms.find("\n## Optional\n"),
            "{llms}"
        );
        assert!(
            !llms[llms.find("\n## Optional\n").expect("Optional")..].contains("\n## House"),
            "{llms}"
        );

        let nav: Value = serde_json::from_str(&staged(&fixture, "rust/nav.json")).expect("JSON");
        assert_eq!(
            nav["nav"],
            json!([
                {"Overview": "index.md"},
                {"Awesome humans": [
                    {"Overview": "awesome-humans/index.md"},
                    {"Citable": "awesome-humans/citable.md"},
                    {"Watch list": "awesome-humans/watch-list.md"},
                    {"Decision log": "awesome-humans/decision-log.md"}
                ]},
                {"Opinions": [{"Overview": "opinions/index.md"}, {"Rust": "opinions/rust.md"}]},
                {"Templates": [{"Overview": "templates/index.md"}, {"config.toml": "templates/cargo/config.toml.md"}, {"Cargo.toml": "templates/Cargo.toml.md"}]},
                {"Research": [{"Overview": "research/index.md"}, {"unsafe code": "research/unsafe.md"}]},
                {"Skills": [{"Overview": "skills/index.md"}, {"vet-source": "skills/vet-source.md"}]},
                {"House opinions": "house-opinions.md"}
            ])
        );

        let sources: Value =
            serde_json::from_str(&staged(&fixture, "rust/sources.json")).expect("JSON");
        assert_eq!(
            sources["sources"],
            json!([
                {"id": "matklad", "name": "Alex Kladov", "kind": "person", "url": "https://matklad.github.io/", "standing": "citable", "marking": "corroborate", "focus": "Tooling", "since": "2017"},
                {"id": "rust-docs", "name": "The Rust Programming Language", "kind": "publication", "url": "https://doc.rust-lang.org/", "standing": "citable", "focus": "Language", "since": "~2015"}
            ])
        );
    }

    #[test]
    fn an_invalid_profile_never_names_a_path() {
        let readme = "# Rust\n\n## Scope\n\n- **[Rust](opinions/rust.md):** the language\n\n## Repository layout\n";
        let opinion = OPINION
            .replace(
                "house:simon-mckenna, matklad, rust-docs",
                "house:../../escaped",
            )
            .replace("/// house | simon-mckenna", "/// house | ../../escaped");
        let fixture = Fixture::new()
            .file("README.md", readme)
            .file("HOUSE-OPINIONS.md", "# House opinions\n")
            .file("opinions/rust.md", &opinion);
        let errors = run(&fixture.repo(), &fixture.path().join("out"));
        assert!(
            errors.contains(&"opinions/rust.md:11: '../../escaped' isn't a profile id: use lowercase words joined by hyphens, other than none".to_string()),
            "{errors:?}"
        );
        let profiles: Vec<_> = fs::read_dir(fixture.path().join("out/rust/raw"))
            .expect("raw/")
            .filter_map(Result::ok)
            .map(|entry| entry.file_name())
            .collect();
        assert_eq!(profiles, ["none"]);
        assert!(!fixture.path().join("escaped").exists());
        assert!(!fixture.path().join("out/escaped").exists());
    }

    #[test]
    fn two_templates_that_share_a_raw_copy_are_an_error() {
        let readme = "# Rust\n\n## Repository layout\n";
        let fixture = Fixture::new()
            .file("README.md", readme)
            .file("HOUSE-OPINIONS.md", "# House opinions\n")
            .file("templates/.cargo/config.toml", "[build]\n")
            .file("templates/cargo/config.toml", "[build]\n");
        let errors = run(&fixture.repo(), &fixture.path().join("out"));
        assert_eq!(
            errors,
            [
                "templates/cargo/config.toml: its raw copy lands on rust/raw/none/templates/cargo/config.toml, as the copy of templates/.cargo/config.toml does; rename one of them"
            ]
        );
    }

    #[test]
    fn a_raw_copy_drops_each_leading_dot() {
        assert_eq!(
            raw_page("templates/.github/PULL_REQUEST_TEMPLATE.md", None),
            "rust/raw/none/templates/github/PULL_REQUEST_TEMPLATE.md.txt"
        );
        assert_eq!(
            raw_page("templates/.cargo/config.toml", Some("simon-mckenna")),
            "rust/raw/simon-mckenna/templates/cargo/config.toml"
        );
    }

    #[test]
    fn a_page_is_served_from_its_own_directory() {
        assert_eq!(
            site_url("rust/index.md"),
            "https://awesome-humans.net/rust/"
        );
        assert_eq!(
            site_url("rust/research/unsafe.md"),
            "https://awesome-humans.net/rust/research/unsafe/"
        );
        assert_eq!(
            site_url("rust/assets/diagrams/pipeline.svg"),
            "https://awesome-humans.net/rust/assets/diagrams/pipeline.svg"
        );
    }

    #[test]
    fn a_broken_link_is_an_error() {
        let readme = "# Rust\n\nSee [nothing](missing.md).\n\n## Repository layout\n";
        let fixture = Fixture::new()
            .file("README.md", readme)
            .file("HOUSE-OPINIONS.md", "# House opinions\n");
        assert_eq!(
            run(&fixture.repo(), &fixture.path().join("out")),
            ["README.md: link to missing.md resolves to missing.md, which doesn't exist"]
        );
    }

    #[test]
    fn a_research_topic_named_index_is_an_error() {
        let readme = "# Rust\n\n## Repository layout\n";
        let fixture = Fixture::new()
            .file("README.md", readme)
            .file("HOUSE-OPINIONS.md", "# House opinions\n")
            .file("research/index.md", "# Index\n");
        assert_eq!(
            run(&fixture.repo(), &fixture.path().join("out")),
            ["research/index.md: the research overview takes this page; rename the topic"]
        );
    }

    #[test]
    fn relative_paths_climb_and_descend() {
        assert_eq!(
            relative("rust/opinions", "rust/house-opinions.md"),
            "../house-opinions.md"
        );
        assert_eq!(relative("rust", "rust/index.md"), "index.md");
        assert_eq!(relative("", "AWESOME-HUMANS.md"), "AWESOME-HUMANS.md");
        assert_eq!(
            combine("skills/vet-source", "../../AWESOME-HUMANS.md"),
            "AWESOME-HUMANS.md"
        );
    }

    #[test]
    fn a_research_label_names_the_topic() {
        assert_eq!(nav_title("Unsafe: soundness and Miri"), "Unsafe");
        assert_eq!(nav_title("`unsafe` code in Rust"), "unsafe code");
        assert_eq!(
            nav_title("`std::sync::Mutex` in Rust: lock poisoning"),
            "std::sync::Mutex"
        );
    }

    #[test]
    fn slug_matches_github_anchors() {
        assert_eq!(
            slug("`matklad`: Citable, **Corroborate.**"),
            "matklad-citable-corroborate"
        );
        assert_eq!(
            slug("Citable: quotable in opinions and templates"),
            CITABLE_ANCHOR
        );
    }
}
