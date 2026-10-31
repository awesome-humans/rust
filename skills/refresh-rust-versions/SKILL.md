---
name: refresh-rust-versions
description: Detect new stable Rust releases and editions, and update all opinions and templates in this repository to target them, including the crate pins in templates/Cargo.toml. Use when a new stable Rust release or edition has shipped, when a pinned crate has a newer stable release, or when asked to check whether the repository targets the latest released versions.
license: See repository LICENSE
compatibility: Requires git and internet access
metadata:
  repo: awesome-humans/rust
  change-flow: branch-pr
---

# Refresh Rust versions

Move every resource to the **latest stable** Rust release and the latest edition, and every crate pin in `templates/Cargo.toml` to its **latest stable** version.

"This repository" in this skill means [awesome-humans/rust](https://github.com/awesome-humans/rust). Run the skill in a checkout, where each path in it, such as `opinions/` or `templates/Cargo.toml`, is relative to the repository's root.

## Trigger

The `.github/workflows/rust-release-watch.yml` workflow checks the official stable channel manifest daily. When the stable release changes, it opens a **trigger pull request** labelled `rust-release` on the branch `automation/rust-release-watch`.

- The trigger pull request only updates the snapshot in `.github/state/rust-releases.json`.
- Run this skill on that branch to make the update, and merge both together.
- You can also run the skill by itself. The snapshot answers "Which stable release does this repository know about?" offline.

## Orchestration

As [AGENTS.md: Orchestration and model economy](../../AGENTS.md#orchestration-and-model-economy) describes, delegate the web lookups to worker agents: the version checks in step 1, the crates.io lookups in step 4b, and the release-notes reading in step 5. Only the editor decides what changes, and edits `opinions/` and `templates/`.

## Steps

1. **Find the latest released versions.**
   - Rust: `pkg.rust.version` in [channel-rust-stable.toml](https://static.rust-lang.org/dist/channel-rust-stable.toml), and the release announcement on the [Rust Blog](https://blog.rust-lang.org/).
   - The edition: the [Edition Guide](https://doc.rust-lang.org/edition-guide/).
   - Use stable releases only. A beta or nightly feature can appear in a "Coming next" note, never as the target.
2. **Compare with the current targets:** the `targets:` metadata in `opinions/`, the version pins in `templates/`, and the root `rust-toolchain.toml` and `Cargo.toml`. The template pins are `channel` in `rust-toolchain.toml`, and `edition` and `rust-version` in each `Cargo.toml`.
   - `targets:` names the minor release, such as `rust-1.99`, so a point release such as 1.99.1 changes no target.
   - If everything matches, the toolchain part has nothing to do. Say so, and **still run step 4b**, because crate pins drift on their own schedule.
   - Report "up to date" and stop only when step 4b finds nothing too.
3. **Start a worktree** on the branch `refresh/rust-<version>`, following the rules in [AGENTS.md](../../AGENTS.md).
4. **Update the template files** under `templates/`: the `edition` and `rust-version` in every `Cargo.toml`, and anything else pinned to a version. Then update the two kinds of pin that need more than a version string:
   - **4a. The toolchain pins.** Move `channel` in the root `rust-toolchain.toml`, and `rust-version` in the root `Cargo.toml`, to the new minor release, such as `1.100`. Pin the template copies the way their opinion says.
     - The root pins name the minor release, never a patch such as `1.99.1`. rustup resolves `1.99` to the newest 1.99 point release, so a point release arrives without an edit.
     - The root pins move with every release, because [AGENTS.md: Scripts](../../AGENTS.md#scripts) holds `xtask/` to the repository's own opinions. Run `just clippy` and `just test` after the move. A new release adds Clippy lints, and CI denies warnings.
     - Renovate never changes the template pins: `renovate.json` turns the cargo manager off under `templates/`. Only this skill changes them.
   - **4b. The crate pins in `templates/Cargo.toml`.** Look up every dependency on crates.io, with the API (`https://crates.io/api/v1/crates/<name>`, sending a `User-Agent` header) or `cargo info <name>`.
     - Take the **latest stable** version: `max_stable_version` in the API's answer. Never take a prerelease, or a version whose `rust-version` is newer than `targets:`.
     - Before you take a new major version, read its changelog. Before 1.0, a minor bump such as 0.12 to 0.13 is a major version, as Cargo's SemVer compatibility rules treat it. A major version that changes an idiom an opinion teaches is a content change: handle it in step 5.
     - Set the `Versions verified against crates.io … on <date>` comment to the run date **whether or not any version changed**. It's the file's only freshness record.

5. **Update each opinion** under `opinions/`:
   - Read the release notes from the first-party sources, `rust-blog` and `rust-docs`: the release announcement, the [release notes](https://doc.rust-lang.org/releases.html), and the Edition Guide. Read feature deep dives from the rest of the citable roster, such as `nnethercote` on compile times or `predrag` on SemVer.
   - Add a newly stabilised feature where it changes an opinion, for example an API that replaces a crate or an old idiom.
   - When a feature needs a new opinion, create a stub with a `TODO` and a source link, and add its `README.md` Scope bullet and Repository layout entry, which CI enforces.
   - Update `targets:` and `last-reviewed:` on every file you change **or verify as unchanged**.
   - **Never remove or weaken a house block or region** ([awesome-humans/website: House opinions](https://github.com/awesome-humans/website/blob/main/pages/conventions/house-opinions.md#how-this-works)). If a release makes one obsolete, note inside the block that the release supersedes it, and keep the marking. A template must still build with its house regions removed.
6. **If a Rust toolchain is available, verify the code samples** against the new targets. Assemble the scratch workspace that [audit-freshness: Templates](../audit-freshness/SKILL.md#templates) describes, and run **`cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check`**.
   - Clippy is the check a new release most often fails. A sample that still compiles can trip a lint the release added or moved into a default group.
   - Treat "running 0 tests" for a crate that has tests as a failure. It means a test target stopped being built, such as a `[[test]]` entry left behind by a rename.
   - Without a toolchain, flag the samples as unverified in the pull request.
7. **If the source roster changed, record it** in the `AWESOME-HUMANS.md` decision log. Otherwise, summarise the run in the pull request.
8. **Open a pull request** to the default branch, filling in the [`version-refresh.md`](https://github.com/awesome-humans/.github/blob/main/PULL_REQUEST_TEMPLATE/version-refresh.md) template, as [AGENTS.md](../../AGENTS.md) describes. It lists:
   - The versions before and after: the Rust release, the edition, and each crate pin that changed.
   - The opinions that changed, and the ones verified as unchanged.
   - Whether the samples built, tested, and passed Clippy.
   - Anything left as a TODO.

   A person reviews it before it changes any opinion.

## Edge cases

- **Release timing:** A stable release ships every six weeks, and a new edition every three years, most recently 2024. A point release, such as 1.99.1, fixes a regression and changes no target. A run between releases that only changes `templates/Cargo.toml` is normal.
- **A new edition:** Moving `edition` is a content change, not a version bump. Run `cargo fix --edition` on the templates, read the Edition Guide's migration notes for each opinion, and say in the pull request what the edition changed.
- **An unmaintained crate:** If a crate's latest stable release is much older than the current `targets:`, the crate is probably unmaintained. Leave the pin, and raise it for `vet-source` or opinion review.
- **Guidance lags the compiler:** If a feature is stable but the vetted sources haven't covered it, update the target, and keep the old idiom with a note. Don't write unsourced guidance.
- **No long-term support track:** Rust has none. Always target the latest stable release.
