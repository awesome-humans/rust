# Checks: formatting, linting, scripts, links

## Formatting and linting

- Markdown follows GFM and markdownlint, configured in `.markdownlint-cli2.jsonc`. The configuration uses the defaults minus one rule that conflicts with the prose style, and records why.
- Spelling follows cspell, configured in `cspell.jsonc` with the `en-GB` dictionary, the `rust` dictionary, and the repository's own word list in `cspell.words.txt`. The dictionaries cspell enables by default accept American spellings because API and crate names use them, such as `Color`, so the configuration flags them in Markdown prose. Editors with a cspell extension read the same configuration.
- Run `just format` to format, and `just check` to check formatting, lint, and spelling together, for prose and Rust. Each check also has its own recipe: `just format-check`, `just lint`, `just spell`, and `just fmt-check`. The prose recipes run the scripts in `package.json`, which use the versions pinned there. Don't run `npx prettier`, `npx markdownlint-cli2`, or `npx cspell` directly, because they resolve the latest version.
- `just lint` also audits the workflows for security problems with zizmor, configured in `.github/zizmor.yml`. It runs through `uvx` at the version `.github/requirements.txt` pins, so it needs uv. Without a GitHub token, zizmor skips its online audits and says so. CI passes `github.token` as `GH_TOKEN`.
- Run `just verify` before you open a pull request. It runs `just check` and every `xtask` check that CI runs, and fails until you commit any diagram it redraws. One of those checks is `just build-templates`, which builds, tests, lints, and format-checks a copy of `templates/` under `target/templates/`, so a change to a template is compiled before it's pushed. See [README.md: Repository scripts](../README.md#repository-scripts).
- Prettier formats Markdown and the JSON, JSONC, and YAML configuration, and reads `.editorconfig`, so the formatter and your editor agree. `rustfmt` formats Rust. `.prettierignore` records one exception: `package-lock.json` is machine-written, and gating it on formatting would fail a build that nobody can fix by editing. TOML isn't formatted, because Prettier has no built-in TOML parser.

## Scripts

- CI checks live in `xtask/`, one binary crate in the root Cargo workspace, with one `clap` subcommand per check. Run them from the repository root with `cargo xtask <name>`, which `.cargo/config.toml` aliases to `cargo run --package xtask --quiet --`. Code that two subcommands share goes in a module that both use, not in a copy.
- The `justfile` is every local and CI entry point. Each check has a recipe of the same name, such as `just validate-metadata`, and CI calls the recipes, never `cargo` directly. `just` with no arguments lists them.
- Subcommands use kebab-case names because you type them on a command line. Modules use snake_case, following Rust's naming convention, so `validate-metadata` lives in `validate_metadata.rs`.
- Add a check as a subcommand, not as an inline script in a workflow, and hold it to the repository's own opinions: `rust-toolchain.toml` pins the stable channel, the crate uses edition 2024 and sets `rust-version` to the targeted release, `[lints]` warns on `clippy::pedantic`, and CI denies warnings.
- Keep the exit codes: 0 when clean, 1 for findings, and 2 when not run from the repository root. CI and the skills read them.
- Test each check with `#[cfg(test)]` cases built from the failures it catches, such as a relative date or a duplicate roster ID, and run them with `just test`.
- When you add or rename a subcommand, update the table in [README.md: Repository scripts](../README.md#repository-scripts) and the layout tree.
- Renovate bumps the crates in `xtask/Cargo.toml` and the root workspace. `renovate.json` turns the cargo manager off under `templates/`, because those pins are content that `refresh-rust-versions` maintains.
- Single-file Cargo scripts (`cargo -Zscript`) are still unstable, tracked in [rust-lang/cargo#12207](https://github.com/rust-lang/cargo/issues/12207), and running checks on nightly would break the repository's own "latest stable" rule. When they stabilise, consider moving each check to a script, as the .NET repository's file-based apps do. **Review by 2027-04-01.**

## Links

The `Markdown` workflow's `links` job runs `just links`, which checks links with `lychee.toml`, on every pull request and weekly. A broken link **blocks the pull request**, because a dead citation is a content defect. If a third-party host is down, re-run the job after it recovers. Never merge past a failed link check.
