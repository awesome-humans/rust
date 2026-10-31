# Plan: awesome-humans/rust

This plan sets up `awesome-humans/rust`: opinionated best practices for building with Rust, made the same way as [dotnet-awesome-humans](https://github.com/si618/dotnet-awesome-humans), which moves to `awesome-humans/dotnet`. It's published as the `/rust/` section of [awesome-humans.net](https://awesome-humans.net/) beside `/dotnet/`.

Written 2026-10-14. Delete this file in the pull request that completes phase 6.

## Decisions

These are settled. Each gives one line on the option that lost.

| Decision             | Choice                                                                                                                                                                                                                                                                                                                                                                                                                            |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Organisation         | A GitHub organisation, `awesome-humans`, holds one repository per ecosystem and one for the site: `website`, `dotnet`, and `rust`. The `<ecosystem>-awesome-humans` names lost because the organisation already says it. The site repository is `website`, not `site`, for clarity: `site` reads as a folder or a workflow. See [Phase 0](#phase-0-organisation)                                                                  |
| Local layout         | `~/Projects/github/awesome-humans/<repository>/`, each a bare clone in `.bare/` with one worktree per branch beside it, as in the .NET repository                                                                                                                                                                                                                                                                                 |
| House opinions       | `HOUSE-OPINIONS.md` starts with an empty Inbox and an empty Woven table. No .NET house opinion is carried over. Each one has to come back through the Inbox if it applies to Rust                                                                                                                                                                                                                                                 |
| Targets              | Latest stable Rust (1.99.1, released 2026-10-15) and the latest edition (2024). Nightly-only features appear only in "Coming next" notes                                                                                                                                                                                                                                                                                          |
| Repository checks    | A Rust `xtask` crate in the workspace, run through a `justfile`. This replaces the .NET repository's `dotnet run scripts/*.cs`                                                                                                                                                                                                                                                                                                    |
| Why not cargo script | `cargo -Zscript` single-file packages are still unstable ([Cargo Book: Unstable features](https://doc.rust-lang.org/cargo/reference/unstable.html#script), tracking issue [rust-lang/cargo#12207](https://github.com/rust-lang/cargo/issues/12207)). Running checks on nightly would break the repository's own "latest stable" rule. A `**Review by 2027-04-01.**` note in `AGENTS.md` records the switch for when it stabilises |
| Prose tooling        | Keep Prettier, markdownlint-cli2, and cspell (`en-GB`) through `npm`, behind `just` recipes. Both repositories then share one prose configuration. A Rust-only stack (dprint, rumdl, typos) lost because typos has no British-spelling dictionary                                                                                                                                                                                 |
| Link checking        | lychee, as now. It's already written in Rust                                                                                                                                                                                                                                                                                                                                                                                      |
| Site                 | `awesome-humans/website` builds every ecosystem's pages into awesome-humans.net. Each ecosystem repository stages its own pages. See [Phase 5](#phase-5-site-integration)                                                                                                                                                                                                                                                         |

## Repository layout (target)

```text
├── README.md                 ← overview, scope index, and maintenance table
├── AGENTS.md                 ← agent instructions, ported from the .NET repository
├── AWESOME-HUMANS.md         ← Rust roster and admission criteria
├── HOUSE-OPINIONS.md         ← empty inbox
├── PLAN.md                   ← this file, deleted when phase 6 lands
├── Cargo.toml                ← workspace: members = ["xtask"]
├── rust-toolchain.toml       ← channel = "stable", components = rustfmt, clippy
├── .cargo/config.toml        ← alias xtask = "run --package xtask --"
├── justfile                  ← every local and CI entry point
├── assets/diagrams/          ← generated SVGs
├── opinions/                 ← one topic per file
├── research/                 ← staging for research topics
├── skills/                   ← maintenance skills (Agent Skills specification)
├── templates/                ← copy-paste-ready Rust project files
└── xtask/                    ← the repository's own checks, in Rust
    └── src/
        ├── main.rs           ← clap subcommands
        ├── frontmatter.rs    ← YAML frontmatter parsing
        ├── comment_header.rs ← first-line template header parsing
        ├── opinions.rs       ← opinion file listing
        ├── validate_metadata.rs
        ├── validate_sources.rs
        ├── validate_readme_index.rs
        ├── export_diagrams.rs
        └── build_site.rs
```

## Phase 0: Organisation

The owner does these steps in the GitHub web UI, because a personal account can't create an organisation through the API. The name `awesome-humans` was free on 2026-10-14.

1. **Create the organisation** `awesome-humans`, and require two-factor authentication for members.
2. **Create `awesome-humans/.github`.** GitHub uses its community health files as the default for every repository that has none of its own. Move `.github/PULL_REQUEST_TEMPLATE.md` there, and keep the ecosystem-specific checklist lines in each repository's `AGENTS.md`. `AGENTS.md` itself, the admission criteria, and the house-opinion mechanism can't be shared this way.
3. **Transfer `si618/dotnet-awesome-humans`** to the organisation, and rename it `dotnet`. GitHub keeps its issues, pull requests, and stars, and redirects the old URLs. Then:
   - Install the Renovate app on the organisation, and give it the transferred repository.
   - Re-enable "Allow GitHub Actions to create and approve pull requests" in the organisation's and the repository's settings, which `dotnet-release-watch.yml` needs.
   - Re-create the Cloudflare secrets that the `Site` workflow reads. They move to `awesome-humans/website` in phase 5.
   - Replace every `si618/dotnet-awesome-humans` in the repository, such as `repo_url` and `repo_name` in `site/zensical.toml`, the `Repository` constant in `scripts/build-site.cs`, and the links in `README.md`. Do this in one `chore/move-to-organisation` pull request.
4. **Create `awesome-humans/rust`** empty, and push this repository's `main` to it once phase 1 has a first commit.
5. **Create `awesome-humans/website`** empty, for phase 5.
6. **Re-clone locally** as `~/Projects/github/awesome-humans/dotnet/`, with `.bare/` and a `main` worktree, and delete the old `~/Projects/github/dotnet-awesome-humans/` once no worktree there holds unpushed work.

## Phase 1: Scaffold

Branch `chore/scaffold`.

1. Copy `LICENSE`, `.editorconfig`, `.gitignore`, `.prettierignore`, `.markdownlint-cli2.jsonc`, `cspell.jsonc`, `lychee.toml`, and `package.json` from the .NET repository. Don't copy its pull request template: the organisation's, in `awesome-humans/.github`, applies to every repository without its own. Then make the copies Rust-specific:
   - Replace the .NET cspell dictionaries with the `rust` dictionary, and start `cspell.words.txt` empty.
   - Add `target/` to `.gitignore`, and drop `bin/`, `obj/`, `.vs/`, and `.dotnet/`.
   - Drop the `MD041` exception from `.markdownlint-cli2.jsonc` if it's still there. It existed only for the .NET repository's own template.
   - Put the Rust code-example check in `AGENTS.md`, not a template: examples pass `cargo check` and `cargo clippy -- -D warnings` on the declared toolchain.
2. Port `AGENTS.md` almost word for word. Writing style, metadata, worktrees, Conventional Commits, agent attribution, and orchestration don't change. Rust-specific edits:
   - Metadata `targets:` becomes `[rust-1.99, edition-2024]`.
   - Template headers use `#` for TOML and `//` for `.rs`. `rust-toolchain.toml` can carry a header. A JSON template, if one is ever added, is exempt.
   - The Scripts section describes `xtask` and `just`, not file-based apps.
   - Branch prefix `refresh/dotnet-<version>` becomes `refresh/rust-<version>`.
3. Port `README.md` with an empty Scope list and the heading "Awesome Humans of Rust", which the site also uses as the title of `/rust/`. Open it with "An agentic take on [awesome-rust](https://github.com/rust-unofficial/awesome-rust): distilled, opinionated guidance, thanks to awesome humans." Match `dotnet`'s Usage section: the two prompts in `text` blocks inside their list items, `Follow the conventions at awesome-humans.net when writing Rust code` and `Use the review-project skill at awesome-humans.net on <repository>`, then the templates and reading bullets. Add `HOUSE-OPINIONS.md` with an empty Inbox and Woven table.
4. Port `renovate.json`:
   - Turn on the `cargo` manager for `xtask/` and the root workspace.
   - Turn it off for `templates/**`, because those pins are content that `refresh-rust-versions` maintains.
   - Keep the npm and github-actions groups.

**Progress (2026-10-18):** `main` holds only `LICENSE`, pushed straight to it because the repository was empty. The rest merged as [rust#1](https://github.com/awesome-humans/rust/pull/1) on 2026-10-18. It differs from the steps above in five ways:

- **No `justfile` yet.** `just` wasn't installed locally, so Phase 2 added the `justfile` with the `xtask` recipes, in [rust#4](https://github.com/awesome-humans/rust/pull/4). `AGENTS.md` already described `just` and `xtask`, as step 2 asks.
- **Links only to files that exist.** `README.md` and `AGENTS.md` name `AWESOME-HUMANS.md`, `skills/`, `templates/`, and README.md's Maintenance via skills and Repository scripts sections in backticks, not as links, so `main` never has a broken link. Turn each one into a link in the pull request that adds its target: phase 3 for the roster, phase 2 for Repository scripts, and phase 4 for the skills.
- **`MD020` dropped too.** It existed only for headings ending in `C#` and `F#`. `MD041` was already gone.
- **`flagWords` kept.** cspell's default dictionaries accept `color`, `catalog`, `center`, and `analyze` under `en-GB` even without the .NET dictionaries, so the list still does its job.
- **No `.github/state/` entries** in `.prettierignore` and `cspell.jsonc`. Phase 4 adds them with `rust-release-watch.yml`.

The `protect-main` ruleset (ID `25144742`) is **active** since 2026-10-18, with `dotnet`'s rules apart from required status checks: it blocks deletion and force pushes, and requires a pull request that merges by squash. A required check that never reports would block every merge, so they waited for CI. Since Phase 2 merged, the ruleset requires the `xtask` and `format-and-lint` jobs, and since Phase 4 merged, the `validate` and `links` jobs too.

`verify-project` is now `review-project`, following [awesome-humans/dotnet#174](https://github.com/awesome-humans/dotnet/pull/174) and [awesome-humans/website#70](https://github.com/awesome-humans/website/pull/70), because "verify" reads as pass or fail while the skill writes a graded report.

**House profiles (done 2026-10-18):** [awesome-humans/website#72](https://github.com/awesome-humans/website/pull/72) and [awesome-humans/dotnet#175](https://github.com/awesome-humans/dotnet/pull/175) merged on 2026-10-18. They replace the bare `house` ID with `house:<profile>`, and keep each house opinion in a block beside the sourced text instead of rewriting it. [rust#3](https://github.com/awesome-humans/rust/pull/3) ported `dotnet`'s result here:

- **`HOUSE-OPINIONS.md`:** the Profiles table, the Profile column in Woven, and the new rule 4.
- **`AGENTS.md` and `README.md`:** `house:<profile>` in the metadata rules and the `sources` sorting example, and the house opinion wording.
- **Still to do:** `houses.toml` in `awesome-humans/website` registers `simon-mckenna` for `dotnet` only. Add `rust` to its `ecosystems` in the pull request that weaves the first Rust house opinion, as HOUSE-OPINIONS.md says.
- **Later phases** port the rest from `dotnet` instead of from this plan: phase 2's `validate-sources` and `build-site` parse the blocks and regions as `House.cs` does, phase 3's roster documents `house:<profile>`, and phase 4's skills take `dotnet`'s house-profile changes, `review-project`'s optional profile among them.

## Phase 2: Rust checks with `xtask` and `just`

Branch `feat/xtask`. Port the five .NET scripts into one binary crate, holding it to the repository's own opinions: `[lints]` with `clippy::pedantic` warned, `-D warnings` in CI, edition 2024, and `rust-version` set to the targeted release.

| .NET script                | `just` recipe                | `xtask` subcommand      | Crates                               |
| -------------------------- | ---------------------------- | ----------------------- | ------------------------------------ |
| `validate-metadata.cs`     | `just validate-metadata`     | `validate-metadata`     | `serde`, `serde_yaml_ng`, `jiff`     |
| `validate-sources.cs`      | `just validate-sources`      | `validate-sources`      | `pulldown-cmark`, `regex`            |
| `validate-readme-index.cs` | `just validate-readme-index` | `validate-readme-index` | `pulldown-cmark`                     |
| `export-diagrams.cs`       | `just export-diagrams`       | `export-diagrams`       | none: SVG is written as text         |
| `build-site.cs`            | `just build-site`            | `build-site`            | `walkdir`, `pulldown-cmark`, `serde` |
| `npm run check`            | `just check`                 | n/a                     | n/a                                  |
| `npm run verify`           | `just verify`                | n/a                     | n/a                                  |

Port `export-diagrams.cs` as it stands after [awesome-humans/dotnet#149](https://github.com/awesome-humans/dotnet/pull/149): the design system's tokens, IBM Plex first in the font stacks, no ecosystem accent, and no "hands off" column. Keep the Rust names in the labels, since the diagrams belong to the README.

- **Exit codes:** Keep the .NET scripts' contract: 0 clean, 1 findings, 2 not run from the repository root. CI and the skills read these codes.
- **Tests:** Each validator gets `#[cfg(test)]` cases built from the .NET repository's known failure inputs, such as a relative date, a duplicate roster ID, and an out-of-order Scope bullet. Port them, and `cargo nextest run -p xtask` runs them in `just verify`.
- **Crate choices:** These are provisional. Each one needs a roster source before the `rust` or `project-structure` opinion recommends it. Until then, they're repository plumbing, not opinions.
- **`justfile`:** It sets `set shell := ["bash", "-euo", "pipefail", "-c"]` and has a default recipe that lists the others. CI installs `just` from a pinned release (1.58.0, released 2026-08-03) through `taiki-e/install-action`, pinned to a commit SHA.

**Done (2026-10-18):** [rust#4](https://github.com/awesome-humans/rust/pull/4) merged. It ports all five scripts and their four helpers, with a `justfile` and a `Rust checks` workflow. `just verify` passes, and the staged `nav.json` and `sources.json` validate against the website's schemas. The SVGs match `dotnet`'s coordinate for coordinate, with only the Rust names changed. It differs from the steps above in seven ways:

- **Crates:** `clap`, which AGENTS.md already names, `jiff`, `regex`, `serde_json`, `serde_yaml_ng`, and `walkdir`, with `tempfile` for the tests. `pulldown-cmark` and `serde`'s derive aren't needed: the port keeps the .NET scripts' line and regex parsing, so the two produce the same findings and pages.
- **Missing directories read as empty.** `rust` has no `opinions/`, `templates/`, `skills/`, or `AWESOME-HUMANS.md` yet, so the checks recognise the root by `AGENTS.md` and `xtask/` instead of by `opinions/`. A directory that exists but is empty is still a finding, and so is a README section missing once its directory exists. `build-site` stages no pages or navigation for a missing part.
- **Tests come from the failures each check catches,** because `dotnet` has no test inputs to port: 59 cases, among them a relative date, a duplicate roster ID, a wrong Citable count, and out-of-order Scope bullets, tree entries, and skill rows.
- **CI pins:** `just` 1.58.0 as planned, because 1.59.0 came out on 2026-10-15, and `cargo-nextest` 0.9.146. Both install through `taiki-e/install-action` v2.87.22.
- **Everything runs through `just`, prose checks included.** Besides the planned recipes, `install`, `format-check`, `lint`, `spell`, `fmt-check`, and `check-diagrams` exist so every workflow calls only recipes. `just check` runs the four checks, and the npm scripts in `package.json` stay underneath, pinning the tool versions. AGENTS.md now names the recipes, not `npm run`.
- **`markdown.yml` arrived early,** with only its `format-and-lint` job, so the prose checks gate pull requests from now on. Phase 4 adds the `links` job and its schedule.
- **The diagrams aren't in the README yet.** Phase 3 adds source standing under Awesome humans, and phase 4 adds the pipeline and research lifecycle with the Maintenance via skills table, as `dotnet`'s README places them.

**After it merged:** the owner added the `xtask` and `format-and-lint` jobs as required status checks on the `protect-main` ruleset (ID `25144742`) on 2026-10-18. Phase 4's `validate` and `links` jobs followed on 2026-10-19.

## Phase 3: Initial roster for vetting

Branch `vet/initial-roster`. Every candidate below goes through `vet-source` before any opinion cites it. The standings given here are **provisional**, based on a feed check on 2026-10-14. `vet-source` decides the real standing.

### Official documentation and project publications

| Proposed ID          | Source                                                                                                                                                                                                                                                                                                                                                                     | Last checked post  | Provisional standing                                                                                         |
| -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------ | ------------------------------------------------------------------------------------------------------------ |
| `rust-blog`          | [Rust Blog](https://blog.rust-lang.org/) and [Inside Rust](https://blog.rust-lang.org/inside-rust/)                                                                                                                                                                                                                                                                        | 2026-10-13         | Citable: release announcements, the canonical "what shipped"                                                 |
| `rust-docs`          | [The Rust Programming Language](https://doc.rust-lang.org/book/), [Reference](https://doc.rust-lang.org/reference/), [Rustonomicon](https://doc.rust-lang.org/nomicon/), [Edition Guide](https://doc.rust-lang.org/edition-guide/), [Cargo Book](https://doc.rust-lang.org/cargo/), [Clippy](https://doc.rust-lang.org/clippy/), and [std](https://doc.rust-lang.org/std/) | Ships each release | Citable: the counterpart to `ms-learn`                                                                       |
| `api-guidelines`     | [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)                                                                                                                                                                                                                                                                                                         | Unchecked          | Citable if maintained. Check for staleness, since much of it predates the 2021 edition                       |
| `rust-patterns`      | [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)                                                                                                                                                                                                                                                                                                        | Unchecked          | Citable, **Corroborate.** It's community-written and shallow in places                                       |
| `perf-book`          | [Nicholas Nethercote: The Rust Performance Book](https://nnethercote.github.io/perf-book/) and [blog](https://nnethercote.github.io/)                                                                                                                                                                                                                                      | 2026-09-30         | Citable: compiler performance work with measurements                                                         |
| `effective-rust`     | [David Drysdale: Effective Rust](https://www.lurklurk.org/effective-rust/)                                                                                                                                                                                                                                                                                                 | Book, 2024         | Citable as a book. Check whether it has been updated for the 2024 edition                                    |
| `ms-rust-guidelines` | [Microsoft: Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/)                                                                                                                                                                                                                                                                                       | 2026-09-25         | Watch list: first published in 2025, so it falls short of two years. Promote in 2027                         |
| `anssi-rust`         | [ANSSI: Secure Rust Guidelines](https://github.com/ANSSI-FR/rust-guide)                                                                                                                                                                                                                                                                                                    | Unchecked          | Citable for `security.md`. It's a government publication with a checklist                                    |
| `tokio-blog`         | [Tokio blog](https://tokio.rs/blog) and [tutorial](https://tokio.rs/tokio/tutorial)                                                                                                                                                                                                                                                                                        | 2026-10-06         | Citable, with an independence note: a project's view of itself carries how it works, not whether to adopt it |

### People

| Proposed ID      | Source                                                                                               | Last checked post | Provisional standing                                                                                                        |
| ---------------- | ---------------------------------------------------------------------------------------------------- | ----------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `alice-ryhl`     | [Alice Ryhl](https://ryhl.io/)                                                                       | Unchecked         | Check for dormancy. Her writing may continue in the Tokio docs                                                              |
| `burntsushi`     | [Andrew Gallant](https://blog.burntsushi.net/)                                                       | 2026-06-03        | Citable: error handling, `regex`, `jiff`, and datetime                                                                      |
| `corrode`        | [Matthias Endler: corrode.dev](https://corrode.dev/) and [endler.dev](https://endler.dev/)           | 2026-10-02        | Citable, with a consultancy independence note                                                                               |
| `fasterthanlime` | [Amos Wenger: fasterthanli.me](https://fasterthanli.me/)                                             | 2025-12-31        | Citable, close to the dormancy line. Text only, because videos are out of scope                                             |
| `jon-gjengset`   | [Jon Gjengset: Rust for Rustaceans](https://rust-for-rustaceans.com/)                                | Book, 2021        | Watch list: most of the writing is the book, and the rest is video                                                          |
| `luca-palmieri`  | [Luca Palmieri](https://lpalmieri.com/) and [Mainmatter blog](https://mainmatter.com/blog/tag/rust/) | 2026-05-28        | Citable: web services, testing, and _Zero To Production_                                                                    |
| `mara-bos`       | [Mara Bos](https://blog.m-ou.se/) and [Rust Atomics and Locks](https://marabos.nl/atomics/)          | 2023-12-05        | Watch list: the blog is dormant. Admit through the book only if `vet-source` applies the "writing continues elsewhere" rule |
| `matklad`        | [Alex Kladov](https://matklad.github.io/)                                                            | 2026-10-07        | Citable: project structure, `xtask`, and testing                                                                            |
| `niko-matsakis`  | [Niko Matsakis: Baby Steps](https://smallcultfollowing.com/babysteps/)                               | 2026-10-10        | Citable: language design and direction                                                                                      |
| `predrag`        | [Predrag Gruevski](https://predr.ag/blog/)                                                           | 2025-10-31        | Citable for SemVer. He writes `cargo-semver-checks`, so add a note                                                          |
| `ralf-jung`      | [Ralf Jung](https://www.ralfj.de/blog/)                                                              | 2026-03-13        | Citable: `unsafe`, soundness, and Miri                                                                                      |
| `raph-levien`    | [Raph Levien](https://raphlinus.github.io/)                                                          | 2025-10-07        | Watch list if past one year when vetted                                                                                     |
| `sabrina-jewson` | [Sabrina Jewson](https://sabrinajewson.org/blog)                                                     | 2024-09-22        | Watch list: dormant                                                                                                         |
| `sunshowers`     | [Rain: sunshowers.io](https://sunshowers.io/)                                                        | 2026-06-16        | Citable: testing (`cargo-nextest`) and the build graph                                                                      |
| `without-boats`  | [without.boats](https://without.boats/)                                                              | 2024-10-25        | Watch list: dormant. Its async design writing is still a valuable reference                                                 |
| `yoshua-wuyts`   | [Yoshua Wuyts](https://blog.yoshuawuyts.com/)                                                        | 2026-08-15        | Citable, **Corroborate.** It's mostly language-design proposals                                                             |

### Vendors (independence notes required)

| Proposed ID       | Source                                                    | Last checked post | Provisional standing                    |
| ----------------- | --------------------------------------------------------- | ----------------- | --------------------------------------- |
| `ferrous-systems` | [Ferrous Systems blog](https://ferrous-systems.com/blog/) | 2026-07-28        | Citable, with a vendor note             |
| `tweede-golf`     | [Tweede golf blog](https://tweedegolf.nl/en/blog)         | 2024-09-30        | Watch list: dormant on the checked page |

### Discovery-only

| Proposed ID         | Source                                                          | Role                                     |
| ------------------- | --------------------------------------------------------------- | ---------------------------------------- |
| `awesome-rust`      | [awesome-rust](https://github.com/rust-unofficial/awesome-rust) | Curated crate and tool list              |
| `blessed-rs`        | [blessed.rs](https://blessed.rs/crates)                         | Curated "default crate" list             |
| `lib-rs`            | [lib.rs](https://lib.rs/)                                       | Crate index with ranking                 |
| `this-week-in-rust` | [This Week in Rust](https://this-week-in-rust.org/)             | Weekly newsletter, last issue 2026-10-07 |

### References, not sources

A reference corroborates a claim and never carries one, under the existing rules. These include a named crate's docs.rs and crates.io pages, and the repositories `AWESOME-HUMANS.md` names under Named references. An RFC links to its RFC Book page, and a RustSec advisory to its page on `rustsec.org`.

**Done 2026-10-23:** [rust#42](https://github.com/awesome-humans/rust/pull/42) adds a Named references section to `AWESOME-HUMANS.md`, naming `rust-lang/rfcs`, `rust-lang/rust`, `rust-lang/libs-team`, and `rustsec/advisory-db`, and the shared [admission criteria](https://github.com/awesome-humans/website/blob/main/pages/conventions/admission-criteria.md#reserved-ids-markings-and-references) hold the rules, since [awesome-humans/website#118](https://github.com/awesome-humans/website/pull/118).

### How to run the vetting

Follow `AGENTS.md: Orchestration and model economy`. Cheaper worker agents check each candidate in parallel for longevity, latest post, and a few sample posts for depth and accuracy. The strongest model is the editor: it decides each standing and writes `AWESOME-HUMANS.md` with its decision log. Use one pull request for the whole first pass.

- **Check every ecosystem's opt-outs first.** Before admitting anyone, read the `## Opted out` section of every other ecosystem's `AWESOME-HUMANS.md`, which is `dotnet`'s today. [website#69](https://github.com/awesome-humans/website/pull/69) settled that an opt-out from one ecosystem covers all of them, so a candidate listed there is declined here without vetting, and the decision log links the other roster's entry.
- **Start with an empty `## Opted out` section,** as dotnet#169 added to `dotnet`'s roster, so the site stages it and `sources.json` can export `opted-out` rows.
- **State the count above the Citable table,** as `dotnet`'s roster does: "The table lists N sources: N people and N publications." `validate-sources` counts the rows by Kind and fails when the sentence disagrees, so port that check in phase 2 with the rest of the script.

**Progress (2026-10-18):** The `vet/initial-roster` pull request adds `AWESOME-HUMANS.md` with 23 citable sources (12 people and 11 publications, four of them Discovery-only and one Corroborate) and 8 on the watch list. Four Sonnet workers gathered the evidence, and the editor decided each standing. `dotnet` lists no opted-out source, so none was declined on that ground. It differs from the tables above in six ways:

- **`perf-book` is now `nnethercote`,** a person, because the book and the blog are one person's work.
- **`tweede-golf` is citable,** not watch-listed: its blog has Rust posts monthly through 2026-10-14, so the 2024-09-30 check read an out-of-date page.
- **`burntsushi` is citable under the dormancy exception.** The blog's latest technical post is from 2023-07-05, but the Jiff comparison and design documents had new writing in 2026-01 and 2026-02.
- **`yoshua-wuyts` isn't marked Corroborate.** The marking is for thin depth, and the proposals are deep, so it gets the same usage limit as `niko-matsakis`: never cite a proposal as current Rust.
- **`effective-rust`, `api-guidelines`, and `luca-palmieri` are watch-listed** as dormant: no writing since 2024-04, 2024-03, and 2025-02.
- **`fasterthanlime` and `alice-ryhl` are citable, but close to the dormancy line,** with their latest posts on 2025-12-31 and 2025-11-05. Each note gives the date to re-evaluate.

`README.md` and `AGENTS.md` now link `AWESOME-HUMANS.md`, and the README shows the source standing diagram under Awesome humans. `vet-source` itself is phase 4, so the README names it in backticks until then.

## Phase 4: Skills

Branch `docs/port-skills`. Port all eight skills, change the ecosystem nouns, and keep the procedures.

| .NET skill                | Rust skill              | Changes beyond nouns                                                                                                                                  |
| ------------------------- | ----------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `refresh-dotnet-versions` | `refresh-rust-versions` | Reads `channel-rust-stable.toml`, the edition, the `rust-version` in templates, and crate pins in `templates/Cargo.toml` against crates.io            |
| `audit-freshness`         | `audit-freshness`       | Version-lag tolerance: two stable releases (12 weeks). Rust ships every six weeks, so bumping `targets:` in every file each release would be noise    |
| `review-project`          | `review-project`        | Runs `cargo metadata`, `cargo clippy`, and `cargo deny check` when a toolchain is present, and reads `Cargo.toml` and `rust-toolchain.toml` otherwise |
| `vet-source`              | `vet-source`            | Before admitting anyone, reads the `## Opted out` section of every other ecosystem's roster, and declines a source listed there without vetting       |
| The other four            | Same names              | Nouns only                                                                                                                                            |

Add `.github/workflows/rust-release-watch.yml`, which compares [channel-rust-stable.toml](https://static.rust-lang.org/dist/channel-rust-stable.toml) with `.github/state/rust-releases.json` daily and opens a trigger pull request. Port `validate-skills.yml` and `harvest-reminder.yml` as they are, and add `markdown.yml`'s `links` job and its weekly schedule beside the `format-and-lint` job that phase 2 added, with `just` recipes as the entry points.

Sort README.md's Maintenance via skills table by skill name, as [awesome-humans/dotnet#176](https://github.com/awesome-humans/dotnet/pull/176) does. Phase 2's `validate-readme-index` ports that PR's order check for the table, along with an out-of-order skill row as a test case.

**Done (2026-10-19):** [rust#11](https://github.com/awesome-humans/rust/pull/11) merged, and [rust#13](https://github.com/awesome-humans/rust/pull/13) closed two opt-out gaps that CodeRabbit found in the dotnet copies, which [awesome-humans/dotnet#181](https://github.com/awesome-humans/dotnet/pull/181) fixed there. #11 ports all eight skills from `dotnet` at `d7d7a39`, adds README.md's Maintenance via skills section with the two diagrams phase 2 already draws, and adds the four workflows. It differs from the steps above in six ways:

1. **`validate-skills.yml` runs only `just validate-skills`.** The metadata, sources, README, and diagram checks in `dotnet`'s workflow of the same name already run in the `Rust checks` workflow, so a copy would run them twice.
2. **The `links` job runs `just links`,** not `lycheeverse/lychee-action`, so the `justfile` stays every entry point. `taiki-e/install-action` installs lychee 0.24.2 through its cargo-binstall fallback, because lychee isn't in the action's own manifests.
3. **The release watch reads TOML** with the runner's Python and `tomllib`, so nothing is installed. The snapshot records the release, the manifest date, and the commit.
4. **`refresh-rust-versions` pins the root toolchain to the minor release,** `1.99`, as `rust-toolchain.toml` does today. How the templates pin it waits for their opinion in phase 6, so the skill defers to that opinion instead of choosing.
5. **`review-project` runs Clippy only on a trusted or sandboxed target,** because building runs the target's build scripts and procedural macros. It also reads a library's `rust-version` as an MSRV promise, and reports one below `targets:` only when the library documents no MSRV policy.
6. **`vet-source` checks opt-outs as a step of its own,** step 2, before any evidence is gathered. The later steps move down by one.

The table order check needed nothing new: phase 2 ported it. On 2026-10-19, the agent added the `validate` and `links` jobs to the `protect-main` ruleset's required checks, at the owner's request. A third-party host being down now blocks a merge until the `links` job is re-run.

## Phase 5: Site integration

The site now lives in the .NET repository, and `build-site.cs` hard-codes `Ecosystem = "dotnet"`. The `site/zensical.toml` comment already plans a move to a shared repository.

1. **Fill `awesome-humans/website`.** Move `site/pages/`, `site/zensical.toml`, `site/wrangler.jsonc`, `site/requirements.txt`, and the `Site` workflow there from the .NET repository. The workflow checks out each ecosystem repository at `main` and runs its staging step:
   - `dotnet run scripts/build-site.cs --out <dir>/dotnet`
   - `cargo xtask build-site --out <dir>/rust`

   Each step writes its pages and a `nav.json` fragment. The website repository merges the fragments into one `zensical.toml` and builds it with `--strict`. A push to `main` in an ecosystem repository triggers a rebuild through `repository_dispatch`. Send it with a token from a GitHub App installed on the organisation, not a personal access token, so the site doesn't depend on one person's account.

2. **Root `llms.txt`** links one `llms.txt` per ecosystem: `/dotnet/llms.txt` and `/rust/llms.txt`.
3. **Root page:** Add `**[Rust](rust/index.md)**` under Ecosystems. Name no ecosystem as next: a third is on hold until `dotnet` and `rust` have shaken out the multi-repository approach.
4. **Link check:** Remove the `awesome-humans.net/rust/` entry from `exclude` in `lychee.toml`, which phase 4 added because the section wasn't published yet.

### Branding

Each section takes its own brand colour, in a light-mode and a dark-mode variant, and the header carries both. The header and tabs stay the same colour in both themes, with white text on them. Contrast was measured on 2026-10-14 against white (`#ffffff`) and against Zensical's slate background (about `#1e2129`). WCAG AA needs 4.5:1.

| Use                                        | Colour                                     | Contrast                                   |
| ------------------------------------------ | ------------------------------------------ | ------------------------------------------ |
| .NET primary, and light-mode links         | `#512bd4` (existing)                       | 7.9:1 with white text                      |
| .NET dark-mode links and accent            | `#a08cf0` (existing)                       | 5.8:1 on slate                             |
| Rust primary, and light-mode links         | `#b7410e` (rust orange)                    | 5.6:1 with white text                      |
| Rust dark-mode links and accent            | `#ff7a3d` (light rust orange)              | 6.2:1 on slate                             |
| Header and tabs, every page and both modes | `linear-gradient(90deg, #512bd4, #b7410e)` | 5.6:1 or better with white text throughout |

- **Why these oranges:** Rust's official logo is monochrome, and rust-lang.org has no orange in its palette. Rust orange comes from the community Ferris artwork, whose `#f74c00` gets only 3.5:1 with white text, and so fails as a header. `#b7410e` is the same hue, deep enough for white text. `#ff7a3d` is its counterpart for links on the dark background, where `#b7410e` gets only 2.9:1.
- **Header gradient:** Set it on `.md-header` and `.md-tabs` in `extra.css`, from .NET purple on the left to rust orange on the right. Give the two elements the same `background-attachment: fixed` sizing, so the tabs carry on the header's gradient instead of restarting it. When a third ecosystem joins, add its colour as a third stop, and check the contrast of every stop.
- **Per-section colour:** Zensical can't set a palette per path. A small `ecosystem.js` sets `data-ecosystem` on `<html>` from the first path segment. `extra.css` then scopes `--md-primary-fg-color`, `--md-accent-fg-color`, and `--md-typeset-a-color` to `[data-ecosystem="dotnet"]` and `[data-ecosystem="rust"]`, each with a `[data-md-color-scheme="slate"]` override. The root page keeps the gradient header and neutral links.
- **Logo:** Use the [official Rust logo](https://github.com/rust-lang/rust-artwork) on the Rust section's index page and the root page's Ecosystems list. It's licensed CC-BY, so credit it in the site footer with a link to the licence, and record any recolouring. Under the [Rust Foundation: Rust trademark policy](https://rustfoundation.org/policy/rust-trademark-policy/), naming the language needs no permission, but the site mustn't look official or endorsed. Add a footer line: "Not affiliated with or endorsed by the Rust Project or the Rust Foundation." Never call the section "official".
- **Fonts:** Keep the system fonts. rust-lang.org uses Fira Sans and Alfa Slab One from Google Fonts, but `zensical.toml` turned off font loading to avoid a third-party request on every page view.

## Phase 6: First opinions and templates

Run `research-topic`, then `resolve-research`, for each topic. Write an opinion only from citable sources admitted in phase 3. In proposed order, with the sources expected to carry each:

| Opinion file             | Covers                                                                                                          | Expected sources                          |
| ------------------------ | --------------------------------------------------------------------------------------------------------------- | ----------------------------------------- |
| `rust.md`                | Edition 2024, the latest stable language features, and idioms                                                   | `rust-docs`, `rust-blog`, `niko-matsakis` |
| `project-structure.md`   | Workspaces, `[workspace.lints]`, `[workspace.dependencies]`, `rust-toolchain.toml`, `rust-version`, and `xtask` | `rust-docs`, `matklad`                    |
| `error-handling.md`      | Library and application errors, `thiserror` versus `anyhow`, and panics                                         | `burntsushi`, `rust-docs`                 |
| `ci.md`                  | `clippy -D warnings`, `cargo-deny`, MSRV checks, caching, and SHA-pinned actions                                | `rust-docs`, `corrode`, `sunshowers`      |
| `testing.md`             | `cargo-nextest`, snapshot and property tests, and doc tests                                                     | `sunshowers`, `matklad`, `luca-palmieri`  |
| `async.md`               | Tokio, cancellation, blocking, and structured concurrency                                                       | `tokio-blog`, `alice-ryhl`                |
| `unsafe.md`              | Soundness, `unsafe_op_in_unsafe_fn`, and Miri                                                                   | `ralf-jung`, `rust-docs`                  |
| `api-design.md`          | SemVer, `cargo-semver-checks`, and public API shape                                                             | `api-guidelines`, `predrag`               |
| `runtime-performance.md` | Profiling, allocation, and release profile settings                                                             | `perf-book`                               |
| `security.md`            | Supply chain (`cargo-deny`, `cargo-vet`, RustSec), and secure coding                                            | `anssi-rust`, `rust-docs`                 |

**Progress (2026-10-19):** [rust#16](https://github.com/awesome-humans/rust/pull/16) merged `research/rust.md`, the topic for `rust.md`. Four Sonnet workers read the release announcements for 1.85 to 1.99.1, the official documentation and Clippy, the roster authors, and the status of unstable features. It proposes promotion, with every recommendation carried by a citable source. Three findings change the steps here:

- **`rust.md` needs more sources than the table expects.** `rust-blog` and `rust-docs` carry the edition and the features, but the idioms rest on `burntsushi`, `corrode`, and `matklad`. `niko-matsakis` and `yoshua-wuyts` support only a "Coming next" note on closure captures, because their posts are proposals.
- **Two templates gain settings from it:** `templates/Cargo.toml` takes `edition = "2024"`, `resolver = "3"`, and `allow_attributes` with `allow_attributes_without_reason` in `[workspace.lints.clippy]`, and `templates/rustfmt.toml` takes `style_edition = "2024"`.
- **Rust 1.100, due 2026-11-12, stabilises the never type.** `refresh-rust-versions` picks it up, and the opinion's "Coming next" table loses that row.

CodeRabbit raised one finding, that the dates should read 2026-10-18, which was declined: the checks ran on 2026-10-19 in local time.

**Progress (2026-10-22):** rust#18 resolved the topic into `opinions/rust.md`, with `templates/Cargo.toml` and `templates/rustfmt.toml`. rust#23 and rust#24 merged `research/project-structure.md` and `research/ci.md` on 2026-10-21, and neither is resolved yet. rust#25 to rust#31 stage the other seven topics, from `error-handling` to `runtime-performance`, and wait for the owner's review. Next is `resolve-research` for `project-structure` and `ci`, then for each of the seven as it merges.

Templates, each with a first-line header comment:

- `Cargo.toml`, a workspace root with `[workspace.lints]` and `[profile.release]`
- `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml`, and `.cargo/config.toml`
- `justfile` and `.editorconfig`
- `projects/`, holding an example library crate, a binary crate, and an integration test

`just verify` builds the template workspace with `cargo check` and `cargo clippy`, so every template compiles against the declared targets. This is the Rust equivalent of the .NET repository's smoke test of the example projects.

**Re-evaluate `xtask/src/` once the first opinions and templates land.** Phase 2 wrote and tested every check against fixtures only, because the repository had no content for them to read. With real opinions, templates, a roster, and skills in place, review each module against what the content needs. Drop the allowances for missing directories that no longer apply, check that the port's regex parsing copes with the real files, and decide whether a Rust-native approach, such as `pulldown-cmark`, would now serve better than following the .NET scripts line by line.

The port also reads like the C# it came from, and it's longer than it needs to be. Tighten it into idiomatic Rust in the same pass:

- **Types instead of strings:** an enum or newtype for a repository path, a page, a profile, and a roster bucket, in place of `String` keys and `&'static str` tags.
- **Iterators and `?` instead of loops and early returns,** where they read better, and a crate error type in place of the hand-built finding strings for I/O failures.
- **Smaller functions:** split `build_site.rs`'s `stage` and `stage_roster`, and the long diagram functions, which carry `#[expect(clippy::too_many_lines)]` today, until they no longer need it.
- **Less shared mutable state:** `build_site.rs` threads one `&mut Site` through every step to collect errors. Return values instead, and collect the errors at the top.
- **Stage atomically:** `build-site` writes straight into `<out>/rust/`, so a failed run leaves partial output. Stage into a temporary directory and move it into place only when there are no errors, as CodeRabbit proposed on rust#4.

## Open questions

These need the owner's decision before the phase that depends on them:

1. ~~**Shared text drift:**~~ Settled 2026-10-18 by the owner: move the shared text, which is `AGENTS.md`'s writing style, the admission criteria, and the house-opinion mechanism, into `awesome-humans/website`, and link to it from each ecosystem repository instead of keeping copies. A fork of an ecosystem repository then either adopts the linked text or writes its own, which is fine. A check that compares copies lost, because it would still leave copies to keep in step. The website plan's Phase 9 has the steps.
2. ~~**Ecosystem repositories without the site:**~~ Done 2026-10-15, in the website plan's cutover: [awesome-humans/dotnet#139](https://github.com/awesome-humans/dotnet/pull/139) added `dotnet`'s `Site preview` and `Site dispatch` workflows, and [awesome-humans/dotnet#140](https://github.com/awesome-humans/dotnet/pull/140) removed the site and updated its `README.md`, `AGENTS.md`, and CI. Checked against `dotnet`'s `main` on 2026-10-19. For `rust`, `ci/site-workflows` adds the same two workflows, which `AGENTS.md` already described.
3. ~~**Phase 5 order:**~~ Settled 2026-10-19 by the owner: wire `rust` into the site now, and publish `/rust/` once the first opinions merge. `rust` stays `soon` in website's `ecosystems.toml` until then, but a `rust` pull request's preview builds it, so each one gets the strict build. Going live now lost, because `/rust/` has no opinions yet. Wiring everything at go-live lost, because the pull request that adds the first opinions would get no site check. The [website plan's Phase 6](https://github.com/awesome-humans/website/blob/a74ffae2609940044c05f40179d2830b4b27a84e/PLAN.md#phase-6-rust-joins), deleted in awesome-humans/website#160, tracked the steps.
4. ~~**Review instructions:**~~ Settled 2026-10-21 by the owner, revised 2026-10-31: a `REVIEW.md` at the root tunes the agent reviewer (currently CodeRabbit), or any other agent asked to review a pull request, as in every awesome-humans repository. It names no reviewer it depends on, because the reviewer might change. It's a full copy of `dotnet`'s, adapted to Rust's toolchain and `xtask`, so a reviewer never has to follow a link out of the repository. A pull request that changes one names the other. [awesome-humans/website#165](https://github.com/awesome-humans/website/pull/165) adds website's copy, and [awesome-humans/coderabbit#7](https://github.com/awesome-humans/coderabbit/pull/7) has CodeRabbit read each one.
