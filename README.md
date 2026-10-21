# Awesome Humans of Rust

An agentic take on [awesome-rust](https://github.com/rust-unofficial/awesome-rust): distilled, opinionated guidance, thanks to awesome humans. Instead of links, it distils what proven Rust writers recommend into guidance your agent can follow, each opinion citing its sources. It documents current best practice in Rust code, and updates it as Rust evolves.

The writers are listed in [AWESOME-HUMANS.md](AWESOME-HUMANS.md): people and publications with at least two years of accurate, independent writing on Rust.

## Usage

- **Point your agent at it.** Tell your coding agent to follow the opinions when it writes code:

  ```text
  Follow the conventions at awesome-humans.net when writing Rust code
  ```

  The opinions cover conventions, project layout, language usage, and crate choices.

- **Check an existing project.** Ask your coding agent to run [`review-project`](skills/review-project/SKILL.md):

  ```text
  Use the review-project skill at awesome-humans.net on <repository>
  ```

  To narrow the review, say what to include or leave out, and optionally name a house profile:

  ```text
  Use the review-project skill at awesome-humans.net on <repository>, focus on logging and testing, skip CI, use house profile of simon-mckenna
  ```

- **Start a project from the templates.** Copy the files in [`templates/`](templates) into a new project, trimming what you don't need.

- **Read the opinions.** Browse them at [awesome-humans.net](https://awesome-humans.net/rust/) or on GitHub. Each page gives its recommendation, with the rationale and sources behind it.

## Scope

[`opinions/`](opinions) holds one file per topic:

- **[Rust](opinions/rust.md):** edition 2024, the stable features that replace older idioms, the lints that enforce them, and panics

## Freshness policy

Opinions target the latest stable Rust release and the latest edition, never an older toolchain. Nightly-only features appear only in "Coming next" notes. The maintenance skills move the repository to each new release.

Every resource records when it was last reviewed against its sources. [AGENTS.md: Metadata](AGENTS.md#metadata) defines the fields. The [repository scripts](#repository-scripts) enforce them.

## Awesome humans

Every opinion cites a vetted source: a person, such as Ralf Jung or Alex Kladov, or a publication, such as the Rust Blog or the Rust documentation. A source is admitted on its track record: at least two years of sustained writing, with depth, accuracy, and an independent view. Videos, talks, and podcasts are out of scope for now, because an opinion cites text a reader can check. [AWESOME-HUMANS.md](AWESOME-HUMANS.md) has the roster and the criteria.

This diagram shows how a source is admitted and what its standing allows. The admission criteria in AWESOME-HUMANS.md and the [`vet-source`](skills/vet-source/SKILL.md) skill are canonical over it.

![Source standing: vet-source sorts a candidate into citable, watch list or declined, and a citable source is unmarked, Corroborate or Discovery-only](assets/diagrams/source-standing.svg)

Standing can change. `vet-source` runs again when an admitted source goes dormant or drops in quality, and when a watch-listed source's blocker clears.

There's one citable tier. Two markings in a source's notes limit how it's cited:

- **Corroborate:** A source marked `**Corroborate.**` is never the only citation for a claim.
- **Discovery-only:** A source marked `**Discovery-only.**` is never cited. Cite the primary source it links to instead.

Any other limit, such as a concern about independence or an outdated back catalogue, goes in the source's notes as prose, and doesn't change its standing.

## House opinions

House opinions are preferences from experience rather than from a vetted source. Each belongs to a house profile, named for the person or organisation that owns it, such as the owner's, `simon-mckenna`. On awesome-humans.net, every page shows sourced community practice only, until you choose a profile in the header. Agents read `llms-<profile>.txt` instead of `llms.txt`. In this repository, each house opinion is a marked block beside the sourced text it changes, and [HOUSE-OPINIONS.md](HOUSE-OPINIONS.md) defines the format and records each one. Other contributors can propose sourced or experience-based opinions through the organisation's [opinion pull request template](https://github.com/awesome-humans/.github/blob/main/PULL_REQUEST_TEMPLATE/opinion.md).

## Repository layout

```text
├── README.md                 ← you are here
├── AGENTS.md                 ← instructions for AI agents working in this repo
├── AWESOME-HUMANS.md         ← the vetted sources: criteria, roster, notes, and decision log
├── HOUSE-OPINIONS.md         ← house profiles' opinions: intake and audit trail
├── Cargo.toml                ← the root Cargo workspace, which holds only xtask/
├── justfile                  ← every local and CI entry point: run just to list them
├── rust-toolchain.toml       ← the toolchain the checks build with
├── assets/diagrams/          ← generated SVG diagrams: rerun the command, never edit by hand
├── opinions/                 ← the opinions, one topic per file, code examples as needed
│   └── rust.md
├── research/                 ← saved research topics (staging: promote or discard)
├── skills/                   ← maintenance skills: see Maintenance via skills
├── templates/                ← copy-paste-ready files that encode the opinions
│   ├── Cargo.toml
│   └── rustfmt.toml
├── xtask/src/                ← this repository's own CI checks, one subcommand each
│   ├── build_site.rs
│   ├── comment_header.rs     ← shared module: template headers
│   ├── export_diagrams.rs
│   ├── frontmatter.rs        ← shared module: YAML frontmatter
│   ├── house.rs              ← shared module: house blocks and regions
│   ├── main.rs               ← the subcommands and their exit codes
│   ├── repo.rs               ← shared module: the repository's files
│   ├── validate_metadata.rs
│   ├── validate_readme_index.rs
│   └── validate_sources.rs
└── PLAN.md                   ← the plan for setting up this repository, deleted when it's done
```

## Maintenance via skills

Agent skills maintain the repository. They follow the [Agent Skills specification](https://agentskills.io/specification), so any compliant agent can run them. Each skill is a directory under [`skills/`](skills) with a `SKILL.md` file.

The skills split work by cost. Cheaper worker agents run the web searches and source sweeps in parallel. The strongest available model is the editor, and only the editor writes to the opinions and templates.

Every skill except the two report-only ones works on its own branch, and its changes land through a pull request that passes CI and the owner's review. Each `SKILL.md` is canonical over this diagram.

![Maintenance pipeline: each skill from what starts it, through CI and the owner's review, to the roster, research/, or the opinions and templates; audit-freshness and review-project only report](assets/diagrams/maintenance-pipeline.svg)

| Skill                                                            | Purpose                                                                                            |
| ---------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| [`audit-freshness`](skills/audit-freshness/SKILL.md)             | Report resources whose `last-reviewed` date has drifted past tolerance                             |
| [`harvest-sources`](skills/harvest-sources/SKILL.md)             | Sweep the awesome-humans sources for new posts, and add notable guidance to the opinions           |
| [`refresh-rust-versions`](skills/refresh-rust-versions/SKILL.md) | Detect new stable Rust releases and editions, and update all opinions and templates to target them |
| [`research-topic`](skills/research-topic/SKILL.md)               | Research a Rust topic from the opinions and vetted sources, and save it with citations             |
| [`resolve-research`](skills/resolve-research/SKILL.md)           | Resolve a saved research topic by weaving it into the opinions and templates, or discard it        |
| [`review-project`](skills/review-project/SKILL.md)               | Check an external project against the opinions and templates, and report deviations                |
| [`vet-source`](skills/vet-source/SKILL.md)                       | Evaluate a candidate source against the track-record criteria and admit or decline                 |
| [`weave-house-opinion`](skills/weave-house-opinion/SKILL.md)     | Weave a house profile's opinion into the opinions and templates, visibly marked as House           |

### Research lifecycle

Research is staged, never merged in place. `research-topic` saves the evidence, and `resolve-research` decides what becomes an opinion and deletes the file in the same pull request. A topic on disk is unresolved. The two `SKILL.md` files are canonical over this diagram.

![Research lifecycle: research-topic saves a topic under research/, and resolve-research promotes, discards or partly promotes it](assets/diagrams/research-lifecycle.svg)

A part that waits on an unvetted source stays on disk. It goes back through `resolve-research` if `vet-source` admits the source, and is discarded if not.

### Release watch automation

A scheduled GitHub Action, [`.github/workflows/rust-release-watch.yml`](.github/workflows/rust-release-watch.yml), checks the official [stable channel manifest](https://static.rust-lang.org/dist/channel-rust-stable.toml) daily against the snapshot in [`.github/state/rust-releases.json`](.github/state/rust-releases.json). When a new stable release appears, it opens a pull request that updates only the snapshot. That pull request is the cue to run [`refresh-rust-versions`](skills/refresh-rust-versions/SKILL.md).

A second, [`.github/workflows/harvest-reminder.yml`](.github/workflows/harvest-reminder.yml), opens a `Harvest due` issue on the 1st of each month, which is the cue to run [`harvest-sources`](skills/harvest-sources/SKILL.md).

## Repository scripts

[`xtask/`](xtask) holds the pull request checks as one binary crate, with one subcommand each, and the [`justfile`](justfile) runs them. CI calls the same recipes, so a check runs the same way everywhere.

| Recipe                       | Checks                                                                                                                                                                                                                                                                                                                                                                                    |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `just validate-skills`       | Every skill under `skills/` passes the Agent Skills specification's reference validator, `skills-ref`, pinned in `.github/requirements.txt`                                                                                                                                                                                                                                               |
| `just validate-metadata`     | Every resource under `opinions/`, `research/`, and `templates/` has `targets`, `last-reviewed`, and `sources`, with ISO 8601 dates and a sorted `sources` list, and no retired `last-used` field                                                                                                                                                                                          |
| `just validate-sources`      | Every source ID in `opinions/` and `templates/` exists in the [AWESOME-HUMANS.md](AWESOME-HUMANS.md) roster and is citable. Each `house:<profile>` ID has a house block or region of that profile, and every house opinion sits inside one. The roster tables and notes sections are sorted by ID, no ID appears twice, every notes section matches a row, and the Citable count is right |
| `just validate-readme-index` | This README lists every opinion and skill, and lists nothing that doesn't exist. Scope is sorted by title, the layout tree by file name, and the skills table by skill name                                                                                                                                                                                                               |
| `just export-diagrams`       | Draws the SVG diagrams in `assets/diagrams/`. It doesn't check anything, but CI runs `just check-diagrams`, which redraws them and fails if the committed SVGs differ                                                                                                                                                                                                                     |
| `just links`                 | Every link in the repository resolves, with the settings in `lychee.toml`. CI runs it on every pull request and weekly                                                                                                                                                                                                                                                                    |
| `just build-site`            | Stages the `/rust/` pages of awesome-humans.net in `<out>/rust/`, `target/site/` by default, to the [awesome-humans/website staging contract](https://github.com/awesome-humans/website#staging-contract)                                                                                                                                                                                 |

Run them from the repository root, as CI does:

```sh
just validate-metadata
```

`just verify` runs every offline check CI runs, in one command: `just check`, `just clippy`, `just test`, the skills check, the three validators, `check-diagrams`, and `build-site`. It redraws the diagrams, and fails until you commit any SVG it changes. The link check is left to CI, because lychee is a separate binary and depends on third-party hosts. Run `just links` to check links locally. `just` with no arguments lists every recipe.

Every check exits 0 when clean, 1 for findings, and 2 when it isn't run from the repository root. Until a later phase adds `opinions/` or `templates/`, the checks read the missing one as empty. Once it exists, an empty directory or a missing README section is a finding.

Four modules hold the code that more than one subcommand uses:

- `repo.rs` lists the opinions, research topics, templates, and skills.
- `frontmatter.rs` parses the YAML frontmatter in `opinions/` and `research/` files.
- `comment_header.rs` parses the first-line comment on `templates/` files.
- `house.rs` finds the house blocks in `opinions/` and the house regions in `templates/`, and writes a file's copy for one house profile, or for none.

## Acknowledgement

Agents build and maintain this project, so the [usual disclaimers](https://awesome-humans.net/credits/#disclaimer) apply, but it wouldn't exist without the people whose writing it draws on. Every opinion cites their work. Thank you to all of these awesome humans, and to everyone who supports them.

## Licence

See [LICENSE](LICENSE).
