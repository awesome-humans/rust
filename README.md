# Awesome Humans of Rust

An agentic take on [awesome-rust](https://github.com/rust-unofficial/awesome-rust): distilled, opinionated guidance, thanks to awesome humans. Instead of links, it distils what proven Rust writers recommend into guidance your agent can follow, each opinion citing its sources. It documents current best practice in Rust code, and updates it as Rust evolves.

The writers are listed in [AWESOME-HUMANS.md](AWESOME-HUMANS.md): people and publications with at least two years of accurate, independent writing on Rust.

## Usage

- **Point your agent at it.** Tell your coding agent to follow the opinions when it writes code:

  ```text
  Follow the conventions at awesome-humans.net when writing Rust code
  ```

  The opinions cover conventions, project layout, language usage, and crate choices.

- **Check an existing project.** Once the Rust skill is published, ask your coding agent to run `review-project`:

  ```text
  Use the review project skill at awesome-humans.net on <repo>
  ```

  To narrow the review, say so in the request, such as "focus on testing".

- **Start a project from the templates.** Once they're published, copy the files in `templates/` into a new project, trimming what you don't need.

- **Read the opinions.** Once they're published, browse them at [awesome-humans.net](https://awesome-humans.net/rust/) or on GitHub. Each page gives its recommendation, with the rationale and sources behind it.

## Scope

`opinions/` holds one file per topic. None is published yet.

## Freshness policy

Opinions target the latest stable Rust release and the latest edition, never an older toolchain. Nightly-only features appear only in "Coming next" notes. The maintenance skills move the repository to each new release.

Every resource records when it was last reviewed against its sources, and opinions and templates also record when an agent last used them. [AGENTS.md: Metadata](AGENTS.md#metadata) defines the fields. The [repository scripts](#repository-scripts) enforce them.

## Awesome humans

Every opinion cites a vetted source: a person, such as Ralf Jung or Alex Kladov, or a publication, such as the Rust Blog or the Rust documentation. A source is admitted on its track record: at least two years of sustained writing, with depth, accuracy, and an independent view. Videos, talks, and podcasts are out of scope for now, because an opinion cites text a reader can check. [AWESOME-HUMANS.md](AWESOME-HUMANS.md) has the roster and the criteria.

This diagram shows how a source is admitted and what its standing allows. The admission criteria in AWESOME-HUMANS.md are canonical, and so is the `vet-source` skill once phase 4 of PLAN.md adds it.

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

## Repository scripts

[`xtask/`](xtask) holds the pull request checks as one binary crate, with one subcommand each, and the [`justfile`](justfile) runs them. CI calls the same recipes, so a check runs the same way everywhere.

| Recipe                       | Checks                                                                                                                                                                                                                                                                                                                                                                                    |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `just validate-metadata`     | Every resource under `opinions/`, `research/`, and `templates/` has `targets`, `last-reviewed`, and `sources`, plus `last-used` outside `research/`, with ISO 8601 dates and a sorted `sources` list                                                                                                                                                                                      |
| `just validate-sources`      | Every source ID in `opinions/` and `templates/` exists in the [AWESOME-HUMANS.md](AWESOME-HUMANS.md) roster and is citable. Each `house:<profile>` ID has a house block or region of that profile, and every house opinion sits inside one. The roster tables and notes sections are sorted by ID, no ID appears twice, every notes section matches a row, and the Citable count is right |
| `just validate-readme-index` | This README lists every opinion and skill, and lists nothing that doesn't exist. Scope is sorted by title, the layout tree by file name, and the skills table by skill name                                                                                                                                                                                                               |
| `just export-diagrams`       | Draws the SVG diagrams in `assets/diagrams/`. It doesn't check anything, but CI runs `just check-diagrams`, which redraws them and fails if the committed SVGs differ                                                                                                                                                                                                                     |
| `just build-site`            | Stages the `/rust/` pages of awesome-humans.net in `<out>/rust/`, `target/site/` by default, to the [awesome-humans/website staging contract](https://github.com/awesome-humans/website#staging-contract)                                                                                                                                                                                 |

Run them from the repository root, as CI does:

```sh
just validate-metadata
```

`just verify` runs everything CI checks locally, in one command: `just check`, `just clippy`, `just test`, the three validators, `export-diagrams`, and `build-site`. It redraws the diagrams in place, so commit any SVG it changes. `just` with no arguments lists every recipe.

Every check exits 0 when clean, 1 for findings, and 2 when it isn't run from the repository root. Until a later phase adds `opinions/`, `templates/`, or `skills/`, the checks read the missing one as empty. Once it exists, an empty directory or a missing README section is a finding.

Four modules hold the code that more than one subcommand uses:

- `repo.rs` lists the opinions, research topics, templates, and skills.
- `frontmatter.rs` parses the YAML frontmatter in `opinions/` and `research/` files.
- `comment_header.rs` parses the first-line comment on `templates/` files.
- `house.rs` finds the house blocks in `opinions/` and the house regions in `templates/`, and writes a file's copy for one house profile, or for none.

## Acknowledgement

Agents build and maintain this project, but it wouldn't exist without the people whose writing it draws on. Every opinion cites their work. Thank you to all of these awesome humans, and to everyone who supports them.

## Licence

See [LICENSE](LICENSE).
