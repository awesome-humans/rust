---
targets: [rust-1.99, edition-2024]
last-reviewed: 2026-10-19
sources: [anssi-rust, corrode, matklad, rust-blog, rust-docs, sunshowers]
---

# CI: warnings, toolchains, MSRV, caching, and workflow hardening

Run CI on the toolchain that `rust-toolchain.toml` pins, and deny warnings with Cargo's `build.warnings`, not `RUSTFLAGS`:

- **Pin the toolchain, and let a bot open the bump.** Install it with `rustup toolchain install` and no arguments, so CI never floats to a release the repository didn't choose.
- **Deny warnings with `CARGO_BUILD_WARNINGS=deny`.** It's stable since 1.97, covers `cargo check`, `cargo clippy`, and `cargo doc`, and doesn't recompile anything when it changes. `RUSTFLAGS=-Dwarnings` lost: adding it recompiles everything it applies to, and `anssi-rust` forbids overriding `RUSTFLAGS`.
- **Commit `Cargo.lock`, and pass `--locked` to every build, lint, and test.**
- **Run four checks:** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features --locked`, the tests, and `cargo hack check --rust-version` for a crate whose `rust-version` trails the pinned toolchain.
- **Cache dependencies with `Swatinem/rust-cache`, and turn debug info off in test builds.** From Rust 1.99, Cargo turns incremental compilation off on its own when `CI` is set.
- **Gate merges on a merge queue,** so every commit on `main` has passed CI.

Every recommendation above is stable on Rust 1.99.1 and rests on a citable source. Hardening the workflow itself (SHA-pinned actions, a read-only `GITHUB_TOKEN`, and `persist-credentials: false`) has no citable source yet, so it's a blocker, not part of the recommendation. For details, see [Workflow hardening](#workflow-hardening).

**No opinion file covers this.** `opinions/rust.md` already says to deny warnings with `cargo clippy -- -D warnings`, and leaves the rest to `ci.md` ([PLAN.md: Phase 6](../PLAN.md#phase-6-first-opinions-and-templates)).

## Toolchain

The Cargo Book's CI guide warns that "CI can fail due to new toolchain versions because there are limited compatibility guarantees around warnings", and says to "Consider pinning the toolchain version with an automated job that creates a PR to upgrade the toolchain on new releases" ([The Cargo Book: Continuous Integration](https://doc.rust-lang.org/cargo/guide/continuous-integration.html#checking-for-warnings), read 2026-10-19). `anssi-rust` requires a stable toolchain: "Development of a secure application MUST be done using a fully stable toolchain" (DENV-STABLE, [ANSSI: Secure Rust Guidelines](https://anssi-fr.github.io/rust-guide/devenv.html)).

- **Install with `rustup toolchain install` and no arguments.** Since rustup 1.28.0, "rustup will no longer automatically install the active toolchain if it is not installed", and the no-argument form installs the one `rust-toolchain.toml` names ([Rust Blog: Announcing Rustup 1.28.0](https://blog.rust-lang.org/2025/03/02/Rustup-1.28.0/), 2025-03-02). GitHub's `ubuntu-latest` image ships rustup 1.29.1 ([actions/runner-images: Ubuntu 24.04](https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2404-Readme.md), read 2026-10-19).
- **Don't use a third-party toolchain action.** `dtolnay/rust-toolchain` selects the toolchain by the Git ref you use it at, so a SHA pin has to sit on `master` and name the toolchain in an input ([dtolnay/rust-toolchain: README](https://github.com/dtolnay/rust-toolchain)). `actions-rust-lang/setup-rust-toolchain` reads the toolchain file, but also turns on caching and `build-warnings: deny` by default ([actions-rust-lang/setup-rust-toolchain: README](https://github.com/actions-rust-lang/setup-rust-toolchain)). Both are references, and neither adds anything that one `rustup` line doesn't. This is the editor's reading: no roster source compares them.

The Cargo Book's starter workflow tests stable, beta, and nightly in a matrix. That contradicts its own advice to pin, and a failure on nightly fails the job. Its later section is the one an opinion should follow. The Clippy book suggests an optional nightly Clippy job only to report false positives to Clippy ([Clippy: Continuous Integration](https://doc.rust-lang.org/clippy/continuous_integration/index.html)).

## Warnings

`build.warnings = "deny"` "can be used to fail a CI job if warnings are present" ([The Cargo Book: Continuous Integration](https://doc.rust-lang.org/cargo/guide/continuous-integration.html#checking-for-warnings)). It's stable since Rust 1.97.0 (2026-07-09), and its environment variable is `CARGO_BUILD_WARNINGS`. It affects only lints in local packages: "Use `--keep-going` to see the lint warnings for all dependent crates" ([The Cargo Book: Configuration](https://doc.rust-lang.org/cargo/reference/config.html#buildwarnings)).

The editor probed it on 1.99.1 on 2026-10-19:

| Probe                                                        | Result                                                         |
| ------------------------------------------------------------ | -------------------------------------------------------------- |
| `CARGO_BUILD_WARNINGS=deny cargo check` with dead code       | Fails: "warnings are denied by `build.warnings` configuration" |
| `CARGO_BUILD_WARNINGS=deny cargo clippy` with `ptr_arg`      | Fails                                                          |
| `CARGO_BUILD_WARNINGS=deny cargo doc` with a broken doc link | Fails with exit code 101                                       |
| Switching `CARGO_BUILD_WARNINGS` between `warn` and `deny`   | No recompilation                                               |
| Adding `RUSTFLAGS=-Dwarnings` to a built crate               | Recompiles the crate                                           |

So one environment variable replaces both `RUSTFLAGS=-Dwarnings` and `RUSTDOCFLAGS=-Dwarnings`.

### Where the sources disagree

- **The Clippy book still says `RUSTFLAGS`.** It recommends `-Dwarnings`, and, "To enforce errors on warnings on all `cargo` commands not just `cargo clippy`, you can set the env var `RUSTFLAGS="-Dwarnings"`" ([Clippy: Continuous Integration](https://doc.rust-lang.org/clippy/continuous_integration/index.html), read 2026-10-19). `matklad` gave the same advice in 2021, against `#![deny(warnings)]` in source ([Alex Kladov: Fast Rust Builds](https://matklad.github.io/2021/09/04/fast-rust-builds.html), 2021-09-04). Both predate `build.warnings`.
- **`anssi-rust` forbids it:** "The environment variables RUSTC, RUSTC_WRAPPER and RUSTFLAGS MUST NOT be overridden when using Cargo to build the project" (DENV-CARGO-ENVVARS).

The weight is with `build.warnings`. The Cargo Book is newer than the Clippy book's page on this, it satisfies `anssi-rust`, and the probe shows it keeps the cache. `opinions/rust.md`'s `cargo clippy -- -D warnings` still works, but it covers only Clippy and only the crates it lints, so `ci.md` should replace it.

## Lockfile and dependencies

- **Commit `Cargo.lock`.** "Cargo.lock files MUST be tracked by a version control system" (DENV-CARGO-LOCK, `anssi-rust`). The Cargo Book lists not committing it as an option that "comes at the cost of determinism".
- **Pass `--locked`.** It stops Cargo from rewriting `Cargo.lock`, which "is particularly useful for CI operations since you save the time to update dependencies" ([Matthias Endler: Tips For Faster Rust CI Builds](https://corrode.dev/blog/tips-for-faster-ci-builds/), 2025-01-28, updated 2025-08-16). A lockfile that's out of date then fails CI, instead of passing on versions nobody reviewed.
- **Update dependencies through bot pull requests.** The Cargo Book lists four ways to test the latest dependencies, and says Dependabot or Renovate "Only uses the resources necessary". `corrode` gives Renovate "a bit of an edge". A scheduled job lost: the guide warns that hosted CI "may disable scheduled jobs for repositories that haven't been touched in a while".

For a library, the guide's latest-dependencies job sets `CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS: allow`, so resolver 3 doesn't hold dependencies back to the crate's `rust-version`, and runs `cargo update` before testing with `continue-on-error: true`. A bot pull request does the same work, so the job isn't needed beside one.

## Checks

These are the jobs an opinion can name. Each is from `rust-docs` unless noted.

| Check  | Command                                                                       | Source                                                                                                                                                                                                                                                                                          |
| ------ | ----------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Format | `cargo fmt --all --check`                                                     | DENV-FORMAT says rustfmt "SHOULD be used" (`anssi-rust`). The command is from [rustfmt: README](https://github.com/rust-lang/rustfmt#checking-style-on-a-ci-server), a reference                                                                                                                |
| Lint   | `cargo clippy --workspace --all-targets --all-features --locked --keep-going` | [The Cargo Book: Continuous Integration](https://doc.rust-lang.org/cargo/guide/continuous-integration.html#checking-for-warnings) and [Clippy: GitHub Actions](https://doc.rust-lang.org/clippy/continuous_integration/github_actions.html). DENV-LINTER says a linter "MUST be used regularly" |
| Test   | `cargo test --workspace --all-targets --locked`, then `cargo test --doc`      | Covered by `testing.md`. `--all-targets` skips doctests, so they run on their own                                                                                                                                                                                                               |
| MSRV   | `cargo hack check --rust-version --workspace --all-targets --ignore-private`  | [The Cargo Book: Continuous Integration](https://doc.rust-lang.org/cargo/guide/continuous-integration.html#verifying-rust-version)                                                                                                                                                              |

**Run Clippy on the same toolchain as the build.** "We recommend to use Clippy from the same toolchain, that you use for compiling your crate" ([Clippy: Continuous Integration](https://doc.rust-lang.org/clippy/continuous_integration/index.html)). Pinning gives this without extra configuration.

**Documentation is the open check.** `CARGO_BUILD_WARNINGS=deny cargo doc --no-deps` fails on a broken intra-doc link, as the probe shows. No citable source says to build documentation in CI: only watch-list `effective-rust` and the CI of `ripgrep` do (see [Watch-list and unvetted material](#watch-list-and-unvetted-material)). An opinion can include it once `api-design.md` or `testing.md` finds a citable source.

### MSRV

**Run the MSRV job only when `rust-version` is lower than the pinned toolchain.** The Cargo Book's job uses `cargo hack check --rust-version`. Its reasons: one platform is enough, `cargo check` is enough "as most issues contributors will run into are API availability and not behavior", and `--ignore-private` skips unpublished packages ([The Cargo Book: Continuous Integration](https://doc.rust-lang.org/cargo/guide/continuous-integration.html#verifying-rust-version)). Install `cargo-hack` with `taiki-e/install-action`, which verifies SHA-256 checksums by default ([taiki-e/install-action: README](https://github.com/taiki-e/install-action)).

Clippy reads `rust-version` from `Cargo.toml`. With `rust-version = "1.85"`, the editor's probe of `Vec::pop_if` on 1.99.1 warned under `clippy::incompatible_msrv`: "current MSRV (Minimum Supported Rust Version) is `1.85.0` but this item is stable since `1.86.0`". That catches most API-availability failures before the MSRV job runs. The Clippy book documents `msrv` in `clippy.toml` but not the fallback to `rust-version`, which only the probe shows.

How far `rust-version` trails is a policy question, not CI. Two references set the range: Tokio keeps "a rolling MSRV … of at least 6 months" ([tokio-rs/tokio: README](https://github.com/tokio-rs/tokio#supported-rust-versions)), and `ripgrep` "tracks the latest stable release" ([BurntSushi/ripgrep: README](https://github.com/BurntSushi/ripgrep#building)). `api-design.md` should own the policy, because raising `rust-version` is a SemVer question ([The Cargo Book: SemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)). An application whose `rust-version` matches the pinned toolchain needs no MSRV job.

RFC 3537, which added the MSRV-aware resolver, agrees on the scope: maintainers "will need to verify at least their MSRV and latest", and "only need to do a compilation check for MSRV" ([RFC 3537: MSRV-aware Resolver](https://rust-lang.github.io/rfcs/3537-msrv-resolver.html)). An accepted RFC is a reference on this roster, so it corroborates the Cargo Book and carries nothing alone.

## Caching and build speed

**Cache dependencies with `Swatinem/rust-cache`, after installing the toolchain.** `corrode` calls it "easily my most important recommendation on this list" ([Matthias Endler: Tips For Faster Rust CI Builds](https://corrode.dev/blog/tips-for-faster-ci-builds/)). `matklad`'s rule is to cache what changes rarely, the dependencies, and not the workspace's own crates ([Alex Kladov: Fast Rust Builds](https://matklad.github.io/2021/09/04/fast-rust-builds.html)). The action does exactly that. Its key includes the rustc version, `Cargo.lock`, and `rust-toolchain.toml`, so a toolchain bump or a dependency change starts a fresh cache ([Swatinem/rust-cache: README](https://github.com/Swatinem/rust-cache), v2.9.2, 2026-08-06).

**Turn debug info off in test builds with `CARGO_PROFILE_TEST_DEBUG: 0`.** Both `corrode` and `matklad` say debug info makes `target/` larger and harms caching. Keep `[profile.dev]` alone, so local debugging is unchanged.

**Drop `CARGO_INCREMENTAL: 0`.** Both posts set it, but since Rust 1.99, "In environments with `CI` set, the default if not set is `CARGO_BUILD_INCREMENTAL=false`" ([The Cargo Book: Configuration](https://doc.rust-lang.org/cargo/reference/config.html#buildincremental)). GitHub Actions sets `CI`, and the editor's probe on 1.99.1 confirmed that `CI=true cargo build -v` passes no `-C incremental` to rustc. The 1.99.0 announcement doesn't mention it, so `rust-blog` can't date it. `rust-lang/rust`'s `RELEASES.md` does.

`sccache` lost: `corrode` calls its benefit "rather negligible" outside shared build servers ([Matthias Endler: Tips for Faster Rust Compile Times](https://corrode.dev/blog/tips-for-faster-rust-compile-times/), updated 2026-03-14). GitHub still gives each repository 10 GB of cache at no cost, and can bill for more since 2025-11-20 ([GitHub Changelog: GitHub Actions cache size can now exceed 10 GB per repository](https://github.blog/changelog/2025-11-20-github-actions-cache-size-can-now-exceed-10-gb-per-repository/)).

**Run tests with `cargo-nextest`, with a `ci` profile.** `corrode` measured "around 40%" in CI, below the "3x" nextest claims, which is the independent measure `sunshowers`' conflict note requires. The nextest documentation sets `fail-fast = false` in `[profile.ci]`, because "CI test runs should not be cancelled as soon as the first test failure is seen" ([nextest: Configuration](https://nexte.st/docs/configuration/)). Adopting nextest is `testing.md`'s decision. `ci.md` should only link it.

## Merging

**Gate merges on a merge queue, so every commit on `main` passes CI.** `matklad` recommends "setting up a merge robot which guarantees that every state of the main branch passes CI", and says such a bot "adds pressure to keep the test suite fast and non-flaky" ([Alex Kladov: Fast Rust Builds](https://matklad.github.io/2021/09/04/fast-rust-builds.html), and [How to Test](https://matklad.github.io/2021/05/31/how-to-test.html), 2021-05-31). He names bors. GitHub's merge queue is the built-in form today, and a branch ruleset that requires the CI jobs is the minimum. The post predates the merge queue, so the opinion should name the guarantee and treat the tool as one way to get it.

## Supply-chain checks

**Check dependencies against the RustSec advisory database on every pull request, without blocking on new advisories.** "The cargo-audit tool MUST be used to check for known vulnerabilities in dependencies" (LIBS-AUDIT, `anssi-rust`). The `cargo-deny-action` README runs advisories in their own matrix entry with `continue-on-error: ${{ matrix.checks == 'advisories' }}`, to "Prevent sudden announcement of a new advisory from failing ci" ([EmbarkStudios/cargo-deny-action: README](https://github.com/EmbarkStudios/cargo-deny-action), v2.1.1). An advisory published overnight then can't fail an unrelated pull request, but still shows.

Choosing between `cargo-audit` and `cargo-deny`, and their configuration, is `security.md`'s question. `anssi-rust` names only `cargo-audit` and `cargo-outdated`, and no citable source weighs them. `ci.md` should name the job and link `security.md`.

## Workflow hardening

**Blocker: no citable source recommends SHA-pinned actions, a read-only `GITHUB_TOKEN`, or `persist-credentials: false`.** The sweep found these only in GitHub's documentation, in tools, and in other projects' CI, none of which can carry a claim:

- **GitHub:** "Pin references to actions to a commit SHA in your workflow files", because an attacker "would need to generate a SHA-1 collision" to change what runs. Set the `GITHUB_TOKEN` to "read access only for repository contents", and avoid `pull_request_target` ([GitHub Docs: Secure use reference](https://docs.github.com/en/actions/reference/security/secure-use), read 2026-10-19). Organisation and repository admins can require SHA pins since 2025-08-15 ([GitHub Changelog: Actions policy now supports blocking and SHA pinning actions](https://github.blog/changelog/2025-08-15-github-actions-policy-now-supports-blocking-and-sha-pinning-actions/)).
- **The incident:** `tj-actions/changed-files` had its tags repointed to a commit that printed CI secrets to logs around 2025-03-14 (CVE-2025-30066), and CISA links it to the `reviewdog/action-setup` compromise on 2025-03-11 (CVE-2025-30154) ([CISA: Supply Chain Compromise of Third-Party GitHub Action](https://www.cisa.gov/news-events/alerts/2025/03/18/supply-chain-compromise-third-party-github-action-cve-2025-30066)). A SHA pin wasn't affected.
- **Tools:** `zizmor` audits workflows for `unpinned-uses`, `excessive-permissions`, `artipacked` (credentials that `actions/checkout` persists), and `template-injection` ([zizmor: Audit Rules](https://docs.zizmor.sh/audits/), v1.31.1, 2026-10-15). Renovate's `helpers:pinGitHubActionDigests` preset pins and updates digests ([Renovate: Helper presets](https://docs.renovatebot.com/presets-helpers/)).
- **Practice:** `rust-lang/cargo` and `jonhoo/rust-ci-conf` pin every action by SHA with a version comment, and both set `permissions: contents: read`. The Cargo Book's own examples and `ripgrep` use tags. This repository's own workflows pin by SHA, as `PLAN.md` decided.

The CISA alert is a reference for what happened, and GitHub's documentation is a reference for how the platform works. Neither decides that a Rust project should pin. Two ways to unblock it, for `resolve-research` to choose between:

1. **`vet-source` a security source on GitHub Actions.** William Woodruff, who leads `zizmor` and writes on Actions security, is the strongest candidate the sweep found. GitHub's own documentation is a vendor's view of itself, so it's a weaker candidate.
2. **Record it as a house opinion** for `house:simon-mckenna` with `weave-house-opinion`, since the repository already follows it.

### Coming next

**Review by 2027-04-14.** GitHub put workflow dependency lockfiles in public preview on 2026-10-14, for GitHub Enterprise Cloud only ([GitHub Changelog: GitHub Actions workflow lockfiles in public preview](https://github.blog/changelog/2026-10-14-github-actions-workflow-lockfiles-in-public-preview/)). The `gh actions-lock` extension writes `.github/workflows/actions.lock`, which pins every action, transitive ones included, and GitHub says that once a repository adopts it, "SHA pinning within workflow files is no longer necessary" ([GitHub Docs: Workflow dependency lockfiles](https://docs.github.com/en/actions/concepts/security/workflow-dependency-lockfiles)). It's a preview, and only GitHub Enterprise Cloud has it, so it can't replace SHA pins yet. Recheck it when it reaches general availability.

**Miri runs only on nightly.** Its README installs `nightly --component miri`, then runs `cargo miri setup` and `cargo miri test` ([rust-lang/miri: README](https://github.com/rust-lang/miri#running-miri-on-ci)). A Miri job is the one CI job that can't follow the pinned stable toolchain, and `unsafe.md` should own it.

## Watch-list and unvetted material

Kept apart from the roster findings above. None of it carried a claim.

- **Watch-list `effective-rust`:** Item 32 lists build, test, Clippy with `-Dwarnings`, `cargo doc`, `cargo-deny`, `cargo fmt --check`, an MSRV build, and a build without `Cargo.lock` for libraries. It says to "pin external scripts to versions or hashes" in public projects ([David Drysdale: Effective Rust, Item 32](https://www.lurklurk.org/effective-rust/ci.html)). It agrees with the checks table, and predates `build.warnings`.
- **Watch-list `ms-rust-guidelines`:** M-STATIC-VERIFICATION names Clippy, rustfmt, `cargo-audit`, `cargo-hack`, `cargo-udeps`, and Miri "as part of check-in gates" ([Microsoft: Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/)). It doesn't say how to wire them into CI.
- **Watch-list `luca-palmieri`:** the _Zero To Production_ repository runs `cargo clippy -- -D warnings`, `cargo fmt --check`, and `cargo llvm-cov`, with actions pinned by tag.
- **Unvetted `jonhoo/rust-ci-conf`** (Jon Gjengset, watch-list for his book, last commit 2026-08-02): five workflows, with Clippy on stable and beta, `cargo hack --feature-powerset check`, an MSRV job, `-Zminimal-versions` on nightly, a daily `cargo update` job on beta, and sanitizers, Miri, and `loom` in a nightly safety workflow. It's the most complete public template, and pins every action by SHA. Its feature-powerset check has no citable source here, and belongs with `api-design.md`.

## What this means for the repository

- **No opinion covers the topic.** Run `resolve-research` to create `opinions/ci.md`, with sections for the toolchain, warnings, lockfile and dependency updates, the checks table, MSRV, caching, merging, and advisory checks. List it under README.md's Scope and in the layout tree.
- **`opinions/rust.md` needs one line changed.** Its Lints section says to deny warnings with `cargo clippy -- -D warnings`. Point it at `ci.md`'s `CARGO_BUILD_WARNINGS=deny` instead. That's part of the same `resolve-research` pass.
- **The repository's own CI doesn't follow the topic yet.** `.github/workflows/xtask.yml` and the `justfile`'s `clippy` recipe use `-D warnings`. That's a `fix` or `ci` change, outside the research skills, and it waits for the opinion.
- **No template change is needed for the plan's list.** Keep `build.warnings` out of `templates/.cargo/config.toml`, so local builds stay lax and only CI denies. `PLAN.md` lists no workflow template, so `resolve-research` can propose one for `templates/.github/workflows/`.
- **Workflow hardening is blocked.** Either `vet-source` a source on GitHub Actions security, starting with William Woodruff, or `weave-house-opinion` it.
- **Leave nextest, MSRV policy, feature combinations, and `cargo-deny` configuration to their topics:** `testing.md`, `api-design.md`, and `security.md`.
