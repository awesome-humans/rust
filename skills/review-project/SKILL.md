---
name: review-project
description: Check an external Rust crate or workspace, given a local path or a repository URL and optionally a house profile, against the opinions and templates in awesome-humans/rust, and report deviations with severity and suggested fixes. Use when asked to verify, audit, review, or grade a Rust codebase against Awesome Humans of Rust conventions, or to check whether a project "does what good looks like".
license: See repository LICENSE
compatibility: Requires read access to the target project; git and internet access for URL targets; a Rust toolchain with cargo-deny helpful but optional
metadata:
  repo: awesome-humans/rust
  change-flow: report-only
---

# Review project

Compare a target Rust crate or workspace with this repository's opinions and templates, and report where it deviates. This skill **only reports** unless the user asks for fixes.

By default, it checks sourced community practice only. If the user names a house profile, such as `simon-mckenna`, it checks that profile's house opinions too ([HOUSE-OPINIONS.md](../../HOUSE-OPINIONS.md)).

"This repository" in this skill means [awesome-humans/rust](https://github.com/awesome-humans/rust). Each path in it, such as `opinions/` or `templates/Cargo.toml`, is relative to the repository's root. Without a checkout, read a file at `https://raw.githubusercontent.com/awesome-humans/rust/main/<path>`, and find the opinions and templates in [llms.txt](https://awesome-humans.net/rust/llms.txt).

## Steps

1. **Resolve the target,** from a local path or a repository URL:
   - **Local path:** Read it in place. Never write to it during the review.
   - **URL:** Read it remotely, with whatever repository browsing the host provides. Clone only when the user asks for a fix (step 9), when the host can't browse the tree well enough to sample files, or to run the tools in step 3. A shallow `git clone --depth 1` into a scratch location is a read cache, not a write to the target.
   - **Record the house profile,** or that there's none. Without one, skip every house block in `opinions/`, and diff against each template with its house regions removed. With one, keep that profile's blocks and regions, and skip the others.
   - **Record what you read:**
     - For a URL, the branch and commit SHA. Without the SHA, nobody can reproduce the review, and step 9 branches from it.
     - For a local git worktree, the SHA. If the tree has uncommitted changes, note that the findings describe the working tree, not the commit.
     - For a local path outside git, say so, and let the file paths stand as the record.
2. **Inventory the target:** `Cargo.toml` and whether it's a workspace, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/config.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml`, `.editorconfig`, the build entry points such as a `justfile` or an `xtask` crate, the CI workflows, and the mix of library and binary crates.
3. **If a Rust toolchain is present, run the tools,** and read the files alone otherwise:
   - `cargo metadata --format-version 1 --no-deps` lists the workspace members, their `edition` and `rust-version`, and their features.
   - `CARGO_BUILD_WARNINGS=deny cargo clippy --workspace --all-targets --locked`, with the lint configuration the target sets. On a toolchain older than 1.97, which has no `build.warnings`, pass `-- -D warnings` instead, so a warning still fails the run. If the target commits no `Cargo.lock`, `--locked` fails, and dropping it makes Cargo write one into the target. Run Clippy in a scratch copy without `--locked` instead, and say the versions were resolved on the day.
   - `cargo deny check`, if the target has a `deny.toml`. A missing one is a gap for step 4.
   - Set `CARGO_TARGET_DIR` to a scratch directory, so the build output stays out of the target.
   - **Clippy runs the target's build scripts and procedural macros,** which can run any code. Run it only on a local target the user owns, or a URL target in a sandbox. Otherwise skip it, and say so in the report.
   - Without a toolchain, read `Cargo.toml` and `rust-toolchain.toml` for the same facts, and mark the Clippy and cargo-deny checks as not run.
4. **Compare structure and tooling** with `templates/`:
   - Required files that are missing, such as `rust-toolchain.toml`, `deny.toml`, or a `[workspace.lints]` table.
   - Files that differ. Diff each against its template, and classify each difference as a _violation_ (it contradicts an opinion) or a _local choice_ (no opinion covers it).
5. **Compare versions:** `edition`, `rust-version`, and the toolchain `channel` against the opinions' `targets:`.
   - An older edition, or a toolchain pin behind `targets:`, is a finding under the freshness policy, even if the project has reasons.
   - A library's `rust-version` is its minimum supported Rust version, a promise to its users, not the toolchain it builds with. Report one below `targets:` only if the target documents no MSRV policy.
6. **Compare code-level opinions** for the areas the target uses, such as error handling, async, `unsafe`, or testing. Sample representative files, and say what you sampled.
7. **Write the report:**
   - Group findings by severity:
     - **Violation:** Contradicts an opinion. Cite the opinion file.
     - **Drift:** Older versions.
     - **Gap:** Missing scaffolding.
     - **Observation:** A local choice worth a look.
   - For each finding, give the path in the target, what the opinion says, and a one-line fix, ideally "copy `templates/<file>` and trim".
   - If the violated opinion is a house block of the chosen profile, say so, and grade the finding one level lower. The target can reasonably follow the sourced text instead.
   - End with a short verdict that reads on its own.

8. **After you present the report, ask in one question whether to save it, and where.** Never save without asking, and never assume a location.
   - Offer `review-<target-name>-<YYYY-MM-DD>.md` in the directory where the user ran the skill, or any path they name. `<target-name>` is the target repository or workspace, so two same-day reviews don't collide.
   - Write one Markdown file with the report's content. Head it with the target's name and the SHA or uncommitted-changes note from step 1.
   - If the user declines, write nothing.

   **Never save the report inside this repository.** It describes someone else's code, not a resource this repository maintains. You can save it into the _target_ if the user asks, but that's a write step 1 ruled out, so it needs the same explicit approval as a fix.

9. **Only if the user asks for fixes:** Apply them in the _target_ on a working branch, never its default branch, named by the host's convention. Do gaps and drift first, because they're mechanical. Make opinionated rewrites only with explicit approval.
   - Clone a URL target now, and branch from **the SHA recorded in step 1**.
   - A local path outside git has no branch. Hand back a patch, or ask the user to put the target under git first.
   - A depth-1 clone has no earlier commit to branch from. Clone without `--depth 1`, or run `git fetch origin <full SHA>`. An abbreviated SHA is rejected.
   - If you must branch from a default branch that has moved, recheck the files behind each finding, and say which changed. A fix written against moved code produces a confidently wrong diff.
   - Run `cargo clippy` and `cargo test` on the fix before you hand it back, under the same sandbox rule as step 3.
   - In commit messages, pull requests, and comments in the target, name this repository `awesome-humans/rust`.

## Edge cases

- **The URL is unreachable, private without credentials, or not a git repository:** Say so and stop. Never review from README text, a crates.io listing, or memory.
- **You can't push to the target:** Make the branch in the clone, and hand back a diff or patch. Never push to a repository the user doesn't control.
- **The target pins an older toolchain for a stated reason,** such as a platform constraint in its README: Report the drift as acknowledged, not actionable.
- **`no_std` or embedded crates:** Review against the opinions that apply without `std`. Don't report a `std`-only convention, such as a crate that needs an allocator, as a violation.
- **The user narrows the review,** such as to testing, one opinion, or one crate in a workspace: Review only that scope, and open the report with what you left out, so nobody reads it as a full review.
- **No opinion covers something the target does:** Report it as out of scope, and note it as a possible opinion gap for `harvest-sources`.
- **The target uses different spelling:** Never a finding. The spelling rule in AGENTS.md covers prose written here, not anyone else's identifiers, comments, or documentation.
- **The templates and opinions disagree:** That's a bug in this repository. Report it against _this_ repository, and review the target against the opinion text, which takes precedence.
