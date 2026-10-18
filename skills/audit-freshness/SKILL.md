---
name: audit-freshness
description: Report every resource in this repository whose last-reviewed date has drifted past tolerance, whose targets lag the latest stable Rust release or edition, whose template version pins (rust-toolchain.toml, rust-version, and the crate versions in templates/Cargo.toml) lag the latest stable releases, or whose sources are no longer on the AWESOME-HUMANS.md roster. Use on a periodic cadence or when asked whether the repository is stale, current, or due for a refresh.
license: See repository LICENSE
compatibility: Requires git; internet access only needed to check latest release versions
metadata:
  repo: awesome-humans/rust
  change-flow: report-only
---

# Audit freshness

Report what's stale. This skill never edits content. Version drift goes to `refresh-rust-versions`, and content drift goes to `harvest-sources`.

## Tolerances

| Check                                                                          | Tolerance                                                                                               |
| ------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------- |
| `last-reviewed` on any `opinions/` file                                        | 180 days                                                                                                |
| `last-reviewed` after a new edition ships                                      | 60 days from the release                                                                                |
| `targets:` against the latest stable Rust release                              | Two stable releases, which is 12 weeks. For details, see [Version lag](#version-lag)                    |
| `targets:` against the latest edition                                          | Zero: any lag is a finding                                                                              |
| `last-reviewed` on any commented `templates/` header                           | 180 days                                                                                                |
| `channel` in `rust-toolchain.toml` and `rust-version` in `Cargo.toml`          | Checked for shape and for agreement with `targets:`. For details, see [Toolchain pins](#toolchain-pins) |
| Crate pins in `templates/Cargo.toml` against crates.io                         | One minor version, or any major version. Patch-only drift is informational                              |
| The `Versions verified against crates.io … on <date>` comment in the same file | 90 days, because the pins change faster than the prose                                                  |
| `research/` topics                                                             | 90 days, then reported for `resolve-research`. For details, see [Research](#research)                   |
| `sources:` IDs                                                                 | For the rules, see [Sources](#sources)                                                                  |
| `**Review by YYYY-MM-DD.**` markings in `opinions/` and `templates/`           | Zero past the date. Report each for drop-or-re-date triage                                              |
| Inbound references to each `opinions/` and `templates/` file                   | Informational, never a finding. For details, see [References](#references)                              |
| `templates/` still implement what `opinions/` require                          | Zero. For details, see [Templates](#templates)                                                          |
| `templates/` build, test, and pass Clippy without errors                       | Zero. For details, see [Templates](#templates)                                                          |
| Citation and roster URLs resolve                                               | Zero: a dead link is a content finding. Read the latest `Markdown` workflow run, not each URL           |

### Version lag

Rust ships a stable release every six weeks. Bumping `targets:` in every file each release would bury the content changes in noise, so a file can lag by up to two releases. With 1.101 the latest, `rust-1.99` is fresh and `rust-1.98` is a finding. A release that stabilises something an opinion covers is a finding sooner: report the opinion as soon as the release ships, whatever its lag.

### Toolchain pins

The root pins lag by the same two releases as `targets:`. Report these too:

- A patch-level pin in the root `rust-toolchain.toml` or `Cargo.toml`, such as `1.99.1`, where the minor release belongs (`refresh-rust-versions`, step 4a).
- A template pin that doesn't follow its opinion.
- A template's `channel` or `rust-version` whose minor release doesn't match the `targets:` in its header.

### Research

Every topic on disk is unresolved, so all of them count. A topic still on disk after promotion is a finding, because promotion ends in deletion.

### Sources

- In an opinion or template, every ID must resolve to a Citable row in `AWESOME-HUMANS.md`, or be a reserved `house:<profile>` ID.
- A `house:<profile>` ID is never a finding, in any directory.
- In an opinion or template, a watch-list ID or a `**Discovery-only.**` ID is a finding. So is a file whose citable sources are all marked `**Corroborate.**`. `just validate-sources` already fails the build on all three.
- `research/` topics can name unvetted sources if the text flags them, but never an opted-out one, because its owner asked not to be cited at all. `just validate-sources` doesn't check `research/`, so report an opted-out ID there.

### References

Count what refers to each opinion and template, by link or by file name, from the repository itself: `skills/`, `AGENTS.md`, `HOUSE-OPINIONS.md`, the other opinions, the template comments, and `research/` topics. Leave out the `README.md` Scope bullets and layout tree, because every opinion appears there.

Report each file with no inbound references as a pruning candidate. Nothing on the web or in another project shows up here, so a file with no references might still be read. Never report it as stale on that count alone.

### Templates

- **Templates match the opinions.** If an opinion names a lint, key, crate, or setting, and its template leaves it out, that's a finding. People copy `templates/`, so a gap ships the advice without the setting.
- **Templates build, test, and pass Clippy.** In `templates/`, run `cargo build`, `cargo test`, and `cargo clippy --all-targets -- -D warnings`. Lint and crate compatibility failures show up only this way.

## Steps

1. **Collect the metadata** from every file under `opinions/` and `research/`, and from the comment headers in `templates/`: `targets`, `last-reviewed`, and `sources`. Count the inbound references to each opinion and template, as [References](#references) describes.
2. **Find the latest released versions** of Rust and the edition, from the same sources as `refresh-rust-versions` step 1. Find the latest stable version of every crate in `templates/Cargo.toml`, with the lookup in `refresh-rust-versions` step 4b.
   - Read the release announcements between each file's target and the latest release, to find the stabilisations [Version lag](#version-lag) reports early.
   - If you're offline, skip the checks you can't run, and say so in the report. Report an unchecked pin as "not verified", never as fresh.
3. **Evaluate each resource** against the tolerances, including any `**Review by YYYY-MM-DD.**` marking in its body ([AGENTS.md: Writing style for opinions](../../AGENTS.md#writing-style-for-opinions)). Missing or malformed metadata is a high-severity finding.
4. **Check the roster.** Flag sources in `AWESOME-HUMANS.md` with no _published writing_ in over a year for `vet-source` re-evaluation. Repository activity doesn't count, because opinions don't cite it.
5. **Write the report,** most stale first:
   - For each resource: the path, the findings, the days or releases over tolerance, and the skill that fixes it.
   - Counts of fresh, stale, and malformed resources, and the oldest `last-reviewed` date.
6. **Recommend next actions,** usually `refresh-rust-versions`, `harvest-sources`, or `resolve-research` for an overdue topic. Don't run them unless asked.

## Edge cases

- **A repository with no opinions yet:** Report that there's nothing to check. Don't pass.
- **House content** (`house:<profile>` in `sources:`, and that profile's house blocks or regions; see HOUSE-OPINIONS.md): The reserved ID is never an unknown-source finding, and never suggest removing it. `just validate-sources` fails a build where the ID and its blocks don't appear together, or `**House:**` sits outside a block. Review a house block's age with its file, and never count it as stale sourced text.
- **Files that can't carry a comment header:** A JSON template, because JSON has no comment syntax ([AGENTS.md: Metadata](../../AGENTS.md#metadata)). None exists today. Missing metadata isn't a finding for one. Audit what it pins against the latest release instead. The root `rust-toolchain.toml` and `Cargo.toml` aren't templates, so they carry no header either, and step 4a's pins are their record.
- **A crate pin held back on purpose,** for a known-bad release or a tracked major migration: Report the drift as informational once a comment beside the pin gives the reason. Deliberate staleness must look different from silent staleness.
- **A resource that doesn't depend on a version,** such as naming conventions: It can declare `targets: [any]`. Version checks skip it, and review-age checks still apply.
- **Clock skew or future dates** in the metadata: Report them as malformed.
