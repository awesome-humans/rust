---
name: follow-opinions
description: Write or change Rust code in a project so that it follows the opinions and templates in awesome-humans/rust. Reads the opinions that cover the change before writing, starts a new manifest or configuration file from the templates, and says afterwards which opinions it applied and where the project's own decisions took precedence. Use when writing, scaffolding, or changing Rust code, a Cargo manifest, lint configuration, or CI in a project that follows the Awesome Humans of Rust opinions, or when asked to "follow the opinions" or to write something "the awesome-humans way".
license: See repository LICENSE
compatibility: Requires write access to the project; a Rust toolchain to build, lint, and test what it writes
metadata:
  repo: awesome-humans/rust
  change-flow: target-only
---

# Follow the opinions

Apply this repository's opinions and templates while you write or change code in a Rust crate or workspace. The opinions are the reference, and the project is where you write. **Never write to this repository** from this skill.

This skill changes only what the task asks for. To find where a project's existing code deviates, run `review-project`.

By default, it follows sourced community practice only. If the user or the project names a house profile, such as `simon-mckenna`, it follows that profile's house opinions too. For the profiles, see [HOUSE-OPINIONS.md](../../HOUSE-OPINIONS.md).

"This repository" in this skill means [awesome-humans/rust](https://github.com/awesome-humans/rust). Each path in it, such as `opinions/` or `templates/Cargo.toml`, is relative to the repository's root. Without a checkout, read an opinion, a template, a skill, the roster, or an agent instruction file from its copy at `https://awesome-humans.net/rust/raw/none/<path>`, with `.txt` added to a Markdown file's name and the leading dot dropped from a file or directory name. [llms.txt](https://awesome-humans.net/rust/llms.txt) links the opinions, templates, and skills. Any other file is at `https://raw.githubusercontent.com/awesome-humans/rust/main/<path>`.

## Steps

1. **Work out what the task touches,** before you read an opinion:
   - Library code, binary code, or both, and whether the crate is `no_std`.
   - `Cargo.toml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml`, or `.cargo/config.toml`.
   - Tests, and CI workflows.
   - The areas: error handling, async code, `unsafe`, or dependencies.
2. **Read the project's own decisions,** in its `AGENTS.md`, `README.md`, and contributing guide:
   - **Record the house profile the user or the project names,** or that neither names one. Without one, skip every house block in `opinions/`, and leave the house regions out of anything you copy from `templates/`. With one, keep that profile's blocks and regions, and skip the others. Without a checkout, read a profile's copy of each file from the links in `https://awesome-humans.net/rust/llms-<profile>.txt`, because the copies under `raw/none/` have no house opinions.
   - **Record each decision that departs from an opinion,** such as a lint it allows or an MSRV policy. A recorded decision takes precedence: follow it, and don't change it back.
3. **Read each opinion that covers the task, in full,** from `opinions/`:
   - [rust.md](../../opinions/rust.md), for every task.
   - Every other opinion whose area step 1 found.

   Read them each time. An opinion changes with each release, so don't write from what you remember of one. Don't read an opinion the task doesn't touch.

4. **Start a new file from its template.** When the task adds a file that `templates/` holds, such as `Cargo.toml` or `rustfmt.toml`, copy the template and trim it. In a file that already exists, change only what the task needs. Don't replace it with the template.
5. **Write the code as the opinions recommend:**
   - **Keep the versions the project targets.** Don't raise an `edition`, a `rust-version`, a toolchain `channel`, or a dependency version unless the task asks. A new crate in an empty repository targets the opinions' `targets:`.
   - **Where an opinion gives a recommendation and an exception,** apply the exception only when its stated condition holds.
   - **Where the code around your change follows another convention,** write the new code as the opinion recommends, unless that leaves one file with two styles. Don't rewrite the code around it.
   - **Don't use a feature that an opinion marks as nightly-only or "Coming next".**
   - **Where no opinion covers something,** follow the project's existing convention. Don't present your own preference as an opinion.
6. **Build, lint, and test what you wrote,** with the commands the project documents. Without any, run `cargo clippy --workspace --all-targets` and `cargo test`, and fix every warning your change adds. Add `--locked` to both when the project commits a `Cargo.lock`: without the file, `--locked` fails before Clippy runs.
7. **Report, after the work:**
   - The opinions you read in full, and the templates you copied.
   - Each place where a decision of the project's took precedence over an opinion.
   - Anything you noticed that deviates from an opinion and didn't change, in one line each. For a full comparison, suggest `review-project`.
   - Anything the task needed that no opinion covers, as a possible opinion gap. The user can raise it in an [issue](https://github.com/awesome-humans/rust/issues).

## Edge cases

- **The project doesn't follow the opinions yet:** Apply them to the code you write, and don't convert the rest as a side effect. Say so, and suggest `review-project`.
- **The user's instruction conflicts with an opinion:** Follow the user. Say once which opinion recommends otherwise, and why.
- **A house block conflicts with the sourced text beside it,** and the project follows that profile: Follow the house block.
- **`no_std` or embedded crates:** Follow the opinions that apply without `std`. Don't apply a `std`-only convention, such as one that needs an allocator.
- **A library's `rust-version`:** It's the crate's minimum supported Rust version, a promise to its users. Don't use a language or library feature newer than it, even when the toolchain allows one.
- **The templates and opinions disagree:** That's a bug in this repository. Follow the opinion text, which takes precedence, and report it against _this_ repository.
- **The project spells differently:** Match the project's spelling in its identifiers, comments, and documentation. The spelling rule in AGENTS.md covers prose written here.
