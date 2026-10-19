---
targets: [rust-1.99, edition-2024]
last-reviewed: 2026-10-19
sources: [burntsushi, corrode, matklad, nnethercote, rust-blog, rust-docs]
---

# Project structure: workspaces, shared manifest tables, toolchain, and `xtask`

Structure every multi-crate repository as one Cargo workspace whose root is a virtual manifest, and declare every shared setting once at the root:

- **Put the crates one level deep under `crates/`,** list them with `members = ["crates/*"]`, and name each directory after its crate. Crates you publish go under `libs/`, which adds `"libs/*"` to `members`.
- **Declare shared package fields in `[workspace.package]`, every external dependency's version in `[workspace.dependencies]`, and the lint levels in `[workspace.lints]`.** Each member inherits all three. `cargo new` inside the workspace writes the inheritance for you.
- **Pin the toolchain in `rust-toolchain.toml` to the targeted minor release,** and set `rust-version` from it for an application, or from a written support policy for a library.
- **Write project automation as an `xtask` crate,** run through a `.cargo/config.toml` alias, instead of shell scripts.
- **Commit `Cargo.lock`,** for libraries too.

Everything here is stable in Rust 1.99.1 (released 2026-10-15). Cargo's own lints, which would catch a member that forgets to inherit `[workspace.lints]`, are unstable on nightly as of 2026-10-19, so they're in [Coming next](#coming-next).

**No opinion file covers this.** `opinions/rust.md` sets `edition` and `resolver` in its Edition 2024 section, and `templates/Cargo.toml` already holds `[workspace]`, `[workspace.package]`, and `[workspace.lints.clippy]`. Neither says how to lay out the crates, which fields to share, or how to pin the toolchain. This is the second topic in [PLAN.md: Phase 6](../PLAN.md#phase-6-first-opinions-and-templates), whose target is `opinions/project-structure.md`.

## Workspace layout

**Make the root a virtual manifest, and keep every crate one level deep under `crates/`.** Two citable sources agree:

- The Cargo Book: "Keep all member packages in a flat directory (commonly `crates/`) and use a glob pattern for the `members` field, e.g. `members = ["crates/*"]`. This minimizes churn in maintaining the `members` list" ([The Cargo Book: Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-members-and-exclude-fields)). `rust-docs`.
- matklad: "Make the root of the workspace a virtual manifest", and nest everything else, the main crate included, "one-level deep under `crates/`". A main crate at the root "pollutes the root with `src/`" ([Alex Kladov: Large Rust Workspaces](https://matklad.github.io/2021/08/22/large-rust-workspaces.html), 2021-08-22). `matklad`.

matklad adds four rules that the Cargo Book doesn't state:

1. "The name of each directory is equal to the name of the crate", and "Don't succumb to the temptation to strip common prefix from folder names."
2. A flat layout beats a hierarchy, because Cargo's crate namespace is flat anyway, and moving or splitting a crate needs no new folder.
3. Keep `src/lib.rs` in a single-file crate: "even if crate is single file now, it might get expanded later."
4. Put crates you publish under a separate top-level `libs/`, so you're "very deliberate" about which crates carry a SemVer promise, and `libs/` can't depend on `crates/`. The glob then needs both directories, as `members = ["crates/*", "libs/*"]`, or Cargo refuses to build a `libs/` crate that no member depends on: "current package believes it's in a workspace when it's not" (editor's probe on Cargo 1.99.1, 2026-10-19).

He scopes the advice to projects of roughly 10 thousand to 1 million lines. Below that, a single package needs no workspace.

The watch-listed Microsoft guidelines (**watch-list**, `ms-rust-guidelines`) agree with the flat `crates/` folder and with never nesting crates, but allow subfolders such as `server/` and `client/` past one or two dozen crates ([Microsoft: Pragmatic Rust Guidelines, Project](https://microsoft.github.io/rust-guidelines/guidelines/project/index.html), version 2026.6). No citable source covers that size, so the opinion should keep matklad's single level.

### What the root holds

A virtual manifest has no package, so a few settings can live only there ([The Cargo Book: Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)):

- **`resolver = "3"`,** because there's no `package.edition` to imply it. `opinions/rust.md` already says this.
- **`[profile.*]`, `[patch]`, and `[replace]`.** Cargo ignores them in a member's manifest. The Rust Performance Book makes the same point for profiles ([The Rust Performance Book: Build Configuration](https://nnethercote.github.io/perf-book/build-configuration.html)). `nnethercote`. The profile settings themselves belong to `runtime-performance.md`.
- **One `Cargo.lock` and one `target/`,** shared by every member.

`default-members` lists the members that a bare `cargo build` at the root acts on. In a virtual workspace, it defaults to every member, which is what the opinion wants.

### Shape the crate graph for build time

**Keep the crate graph wide, not a chain:** a shared vocabulary crate, independent feature crates, and one leaf that ties them together. Cargo then compiles the feature crates in parallel ([Alex Kladov: Fast Rust Builds](https://matklad.github.io/2021/09/04/fast-rust-builds.html), 2021-09-04). `matklad`.

- **Split a crate that `cargo build --timings` shows serialising the build.** The Rust Performance Book points to `--timings` for exactly this ([The Rust Performance Book: Compile Times](https://nnethercote.github.io/perf-book/compile-times.html)). `nnethercote`. corrode recommends workspaces for the same reason, because only the crates that changed recompile ([Matthias Endler: Tips For Faster Rust Compile Times](https://corrode.dev/blog/tips-for-faster-rust-compile-times/), updated 2026-03-14). `corrode`.
- **Keep `serde` derives and other heavy procedural macros in leaf crates,** or behind an optional feature in shared ones. Both `matklad` and `corrode` give this rule in the posts above.
- **Make every binary count.** Each binary, example, and file in `tests/` links separately. matklad merges integration tests into one `tests/it/main.rs` ([Alex Kladov: Delete Cargo Integration Tests](https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html), 2021-02-27), and corrode agrees. That rule belongs to `testing.md`.

### Document the structure

**Add an `ARCHITECTURE.md` beside the README** once a project passes about 10 thousand lines: a bird's-eye view, then a code map that answers "where's the thing that does X?", then the invariants and boundaries ([Alex Kladov: ARCHITECTURE.md](https://matklad.github.io/2021/02/06/ARCHITECTURE.md.html), 2021-02-06). `matklad`. The post isn't specific to Rust, but its example is the architecture document of `rust-analyzer`.

## Shared manifest tables

**Declare every setting that two members share once at the root, and inherit it.** Workspace inheritance has been stable since Rust 1.64 ([Rust Blog: Announcing Rust 1.64.0](https://blog.rust-lang.org/2022/09/22/Rust-1.64.0/)), and `[workspace.lints]` since 1.74 ([Rust Blog: Announcing Rust 1.74.0](https://blog.rust-lang.org/2023/11/16/Rust-1.74.0/)). `rust-blog`.

Since Cargo 1.75, `cargo new` inside a workspace adds the new crate to `members` ([The Cargo Book: Changelog, Cargo 1.75](https://doc.rust-lang.org/cargo/CHANGELOG.html#cargo-175-2023-12-28)). The editor's probe on 2026-10-19, with Cargo 1.99.1, showed that it also writes `.workspace = true` for each key in `[workspace.package]`, and `[lints] workspace = true`. A member made by hand or copied from another repository gets neither.

### `[workspace.package]`

**Put `edition`, `rust-version`, `license`, `repository`, and `publish` in `[workspace.package]`, and inherit each with `<key>.workspace = true`.** The table accepts 16 keys: `authors`, `categories`, `description`, `documentation`, `edition`, `exclude`, `homepage`, `include`, `keywords`, `license`, `license-file`, `publish`, `readme`, `repository`, `rust-version`, and `version` ([The Cargo Book: Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-package-table)). `rust-docs`.

- **Leave `name`, `description`, and `version` to the member,** because they differ per crate.
- **Set `publish = false` at the root,** and override it with `publish = true` only in a crate under `libs/`. matklad gives internal crates `version = "0.0.0"` instead ([Large Rust Workspaces](https://matklad.github.io/2021/08/22/large-rust-workspaces.html)), and the post never mentions `publish`. `publish = false` makes Cargo refuse to publish, which is the stronger guard ([The Cargo Book: The manifest format](https://doc.rust-lang.org/cargo/reference/manifest.html#the-publish-field)), so recommend both.

### `[workspace.dependencies]`

**Declare each external dependency's version once, in `[workspace.dependencies]`, and inherit it in each member with `dep.workspace = true`.** A member may add only three keys beside `workspace`: `optional`, `features`, and `default-features`. Its `features` add to the workspace entry's ([The Cargo Book: Inheriting a dependency from a workspace](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#inheriting-a-dependency-from-a-workspace)). `rust-docs`.

**New in 1.99:** in an edition 2024 member, `default-features` overrides the workspace entry's value. Before 1.99, or in an earlier edition, `default-features = false` in a member "may be ignored or rejected" unless the workspace entry also sets it. The editor's probe confirmed both behaviours on Cargo 1.99.1. A member on edition 2024 that turned off a path dependency's default features built without them, and the same member on edition 2021 built with them.

So, on the targeted toolchain:

- **Declare only the version in `[workspace.dependencies]`,** with no features, and let each member ask for the features it uses. Feature unification still builds the union of every member's features into one copy ([The Cargo Book: Feature unification](https://doc.rust-lang.org/cargo/reference/features.html#feature-unification)), so a feature in the workspace entry turns on everywhere. The watch-listed Microsoft guidelines say the same (**watch-list**, `ms-rust-guidelines`).
- **Put path dependencies between members in `[workspace.dependencies]` too,** as `foo = { path = "crates/foo" }`. Members then name each other the same way they name an external crate. The Cargo Book's own example inherits a path dependency this way.

```toml
# Root Cargo.toml
[workspace.dependencies]
serde = "1.0.228"
foo = { path = "crates/foo" }

# crates/bar/Cargo.toml
[dependencies]
serde = { workspace = true, features = ["derive"] }
foo.workspace = true
```

This is a fragment: the root needs its `[workspace]` table, and the member its `[package]`.

### `[workspace.lints]`

**Set every lint level in `[workspace.lints]`, and put `[lints] workspace = true` in every member.** Inheritance is opt-in, and Cargo's own documentation warns that "many people mistakenly think that `workspace.lints` is implicitly inherited when it is not" ([The Cargo Book: Lints](https://doc.rust-lang.org/cargo/reference/lints.html#missing_lints_inheritance)). `rust-docs`.

- **A member can't override one inherited lint.** Cargo 1.99.1 rejects `[lints] workspace = true` beside a `[lints.clippy]` table: "cannot override `workspace.lints` in `lints`, either remove the overrides or `lints.workspace = true` and manually specify the lints" (editor's probe, 2026-10-19). Silence a lint where it fires with `#[expect(…, reason = "…")]`, as `opinions/rust.md` already says.
- **Give a lint group `priority = -1`** when you turn on a group and then change one of its lints, as in `pedantic = { level = "warn", priority = -1 }`. A lower priority is applied first, so the single lint wins ([The Cargo Book: The `[lints]` section](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section)). `rust-docs`.
- **Which lints to set is `rust.md`'s and `ci.md`'s business.** This opinion owns only where they're declared.

`[lints]` in a virtual manifest has been an error since Cargo 1.76, because the author almost certainly meant `[workspace.lints]` ([The Cargo Book: Changelog, Cargo 1.76](https://doc.rust-lang.org/cargo/CHANGELOG.html#cargo-176-2024-02-08)). `rust-docs`.

## Toolchain and `rust-version`

### `rust-toolchain.toml`

**Commit a `rust-toolchain.toml` that pins the targeted minor release, with Clippy and rustfmt:**

```toml
[toolchain]
channel = "1.99"
components = ["clippy", "rustfmt"]
profile = "minimal"
```

The file's format and precedence are rustup's: `channel` takes `stable`, a version such as `1.99` or `1.99.1`, or a dated nightly, and only `cargo +<toolchain>`, `RUSTUP_TOOLCHAIN`, and a directory override beat it ([The rustup book: Overrides](https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file)). The rustup book is a tool's own documentation, so it's a reference for the format, not a source for the recommendation.

Why pin, and not `channel = "stable"`:

- **A new stable release adds warn-by-default lints,** and CI denies warnings. For example, `io_other_error` arrived in 1.87 and `manual_is_multiple_of` in 1.90 (from the release announcements on `rust-blog`, as `opinions/rust.md` lists them). On a floating `stable`, a release day breaks the build of a commit that passed the day before.
- **corrode pins the compiler version in build images,** and uses the same one in every stage ([Matthias Endler: Hardening Rust Code For Production](https://corrode.dev/blog/hardening-rust/), 2026-07-23). `corrode`. The same post builds with `--locked`.
- **Pinning `1.99` rather than `1.99.1`** takes point releases without an edit. A point release carries fixes only. 1.99.1 fixed `wcslen`, FFI `bool` returns on x86-64, and `rust-lld` on macOS ([Rust Blog: Announcing Rust 1.99.1](https://blog.rust-lang.org/2026/10/15/Rust-1.99.1/)). `rust-blog`.

**Where the sources pull the other way:** corrode also says to "stick to the stable release of Rust if at all possible" and run `rustup update` regularly ([Matthias Endler: Long-term Rust Project Maintenance](https://corrode.dev/blog/long-term-rust-maintenance/), 2024-05-13), and its CI examples use the latest stable. That's an argument against nightly and against falling behind, not against pinning a stable release. A pin that a bot bumps on each release keeps both. Renovate, which this repository already runs, can bump `rust-toolchain.toml`, and the repository's own `refresh-rust-versions` skill does the same for its templates. No roster source argues for a floating `stable` in a toolchain file. **Unvetted** discussion, such as forum threads and CI templates, splits between the two, so this recommendation rests on the lint argument.

### `rust-version`

**Set `rust-version` in `[workspace.package]`, and inherit it in every member.** Cargo refuses to build a package on an older toolchain, `cargo add` picks the newest dependency version that supports it, and edition 2024's resolver 3 prefers dependency versions that do ([The Cargo Book: Rust version](https://doc.rust-lang.org/cargo/reference/rust-version.html), [Resolver: Rust version](https://doc.rust-lang.org/cargo/reference/resolver.html#rust-version)). Clippy reads it as its `msrv`, so it never suggests a newer API ([Clippy: Lint configuration](https://doc.rust-lang.org/clippy/lint_configuration.html#msrv)). The resolver has been MSRV-aware since 1.84 ([Rust Blog: Announcing Rust 1.84.0](https://blog.rust-lang.org/2025/01/09/Rust-1.84.0/)). `rust-docs` and `rust-blog`.

The value depends on what the workspace ships:

| Workspace ships           | `rust-version`                                      | Why                                                                                              |
| ------------------------- | --------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| An application or service | The pinned toolchain's minor release, as `"1.99"`   | Nobody else compiles it, so the toolchain file and `rust-version` say the same thing             |
| A published library       | The oldest release the written support policy keeps | Users compile it on their own toolchain, so it's a promise, and the Cargo Book asks for a policy |

For a library, the Cargo Book says to "choose a policy for what Rust versions to support and when that is changed", treats raising `rust-version` as "a minor incompatibility", and gives three example policies: the last N releases, every even release, or a calendar window ([The Cargo Book: Rust version](https://doc.rust-lang.org/cargo/reference/rust-version.html#support-expectations)). `rust-docs`. It doesn't pick one.

burntsushi's crates write a policy down. `regex` "can be increased in minor version updates", and patch releases keep it ([regex: README](https://github.com/rust-lang/regex#minimum-rust-version-policy)). Jiff "will never use a Rust version that requires a Rust compiler released in the previous 1 year", and raises it only in a minor release ([Jiff: README](https://github.com/BurntSushi/jiff#minimum-rust-version-policy)). `burntsushi`, writing in repository documentation under the dormancy exception. Tokio's six-month rolling window (**unvetted**, from its docs.rs page) and the Microsoft guidelines' "a few versions behind" (**watch-list**) are shorter.

**Recommend Jiff's form for a published library:** support every stable release from the last 12 months, raise `rust-version` only in a minor release, and say so in the README. It's the only written policy from a citable source, and a calendar window is one of the Cargo Book's three examples. Checking it in CI, with `cargo-hack` or `cargo-msrv` as the Cargo Book suggests, belongs to `ci.md`.

**This doesn't conflict with the repository's "never downgrade" rule.** `targets:` names the toolchain the opinions and templates are written for. A library's `rust-version` is a promise to its users, and `review-project` already reads it that way.

## `xtask`

**Write the project's automation in Rust, as a binary crate named `xtask` inside the workspace,** and alias it in `.cargo/config.toml`:

```toml
[alias]
xtask = "run --package xtask --"
```

`cargo xtask <task>` then works from any directory in the workspace, needs only Cargo and rustc, and runs the same on every platform ([matklad/cargo-xtask: README](https://github.com/matklad/cargo-xtask)). `matklad`. "To avoid both the bloat and proliferation of ad-hoc workflows, write all automation in Rust in a dedicated crate" ([Large Rust Workspaces](https://matklad.github.io/2021/08/22/large-rust-workspaces.html)). The README adds three rules:

- **Make `xtask` a workspace member,** so it shares the lockfile and the alias works from any directory. A separate `--manifest-path` workspace is allowed but discouraged.
- **Keep its compile time low:** few dependencies, and no heavy procedural macros.
- **Run Cargo through the `CARGO` environment variable,** so a nested build uses the same toolchain.

Give the crate `publish = false`. The alias form follows the Cargo Book's `[alias]` table ([The Cargo Book: Configuration, `[alias]`](https://doc.rust-lang.org/cargo/reference/config.html#alias)), which allows a string or an array, but can't redefine a built-in command.

matklad's later "Basic Things" generalises the rule: "a single, clearly documented command for building and testing the project", with glue code in the project's main language, not shell or Python ([Alex Kladov: Basic Things](https://matklad.github.io/2024/03/22/basic-things.html), 2024-03-22). `matklad`.

**`just` and `cargo-make` have no citable source.** No roster source compares them with `xtask`. This repository itself puts a `justfile` in front of `cargo xtask`, and PLAN.md lists a `justfile` among the planned templates. That's at odds with matklad's single-language rule unless the `justfile` holds only one-line recipes that call `cargo`. See [What this means for the repository](#what-this-means-for-the-repository).

## `Cargo.lock`

**Commit `Cargo.lock` in every workspace, libraries included.** The Rust project changed its guidance on 2023-08-29: `cargo new` no longer ignores the lockfile for a library, and the starting point is to commit it, because a lockfile makes `git bisect` and CI deterministic ([Rust Blog: Change in Guidance on Committing Lockfiles](https://blog.rust-lang.org/2023/08/29/committing-lockfiles/)). `rust-blog`. The Cargo FAQ keeps one caveat: the lockfile "does not affect the consumers of your package", so test against the latest dependencies as well ([The Cargo Book: FAQ](https://doc.rust-lang.org/cargo/faq.html#why-have-cargolock-in-version-control)). `rust-docs`. That CI job belongs to `ci.md`.

## Coming next

These are unstable on nightly as of 2026-10-19, so they're not yet an opinion:

- **Cargo's own lints and `[lints.cargo]`** ([The Cargo Book: Lints](https://doc.rust-lang.org/cargo/reference/lints.html), "can only be used on nightly toolchains"). They include `missing_lints_inheritance`, `unused_workspace_dependencies`, and `unused_workspace_package_fields`, each warn by default, which Cargo 1.95 added on nightly. They'd catch a member that forgets `[lints] workspace = true`. One worker reported that they stabilise in 1.100, due 2026-11-12, but the editor couldn't confirm it in the Cargo changelog, which has no 1.100 section yet. `refresh-rust-versions` should check when it moves the targets to 1.100.
- **Per-package feature unification** (`-Zfeature-unification`, [rust-lang/cargo#14774](https://github.com/rust-lang/cargo/issues/14774)), which would replace a workspace-hack crate.
- **Public and private dependencies** (`-Zpublic-dependency`, [rust-lang/rust#44663](https://github.com/rust-lang/rust/issues/44663)).
- **Path bases** (`-Zpath-bases`, [rust-lang/cargo#14355](https://github.com/rust-lang/cargo/issues/14355)).
- **Single-file packages** (`-Zscript`, [rust-lang/cargo#12207](https://github.com/rust-lang/cargo/issues/12207)), which `AGENTS.md` already tracks as a possible replacement for `xtask` subcommands.

`niko-matsakis` proposes "battery packs", crates that bundle recommended dependencies, features, and templates, with a `cargo-bp` prototype ([Niko Matsakis: Battery packs](https://smallcultfollowing.com/babysteps/blog/2026/07/15/battery-packs/), 2026-07-15). It's a proposal and changes nothing in Cargo, so it isn't cited in the frontmatter, and it belongs in no opinion yet.

## What this means for the repository

- **No opinion covers the topic.** Run `resolve-research` to create `opinions/project-structure.md`. Its first sentence is the opening list here. It needs these sections: workspace layout, the three shared tables, the toolchain file and `rust-version`, `xtask`, and `Cargo.lock`. List it under README.md's Scope and in the layout tree.
- **The templates need these changes:**
  - `templates/Cargo.toml`: `members = ["crates/*"]` with a comment to add `"libs/*"` for published crates, and `rust-version`, `license`, `repository`, and `publish = false` in `[workspace.package]`. Add an empty `[workspace.dependencies]` with a comment on declaring versions only.
  - `templates/rust-toolchain.toml` and `templates/.cargo/config.toml`, which PLAN.md plans: `channel = "1.99"` with `clippy` and `rustfmt`, and the `xtask` alias. This settles the deferred decision PLAN.md records for `refresh-rust-versions`: the templates pin the minor release.
- **The planned `justfile` template has no citable source.** `resolve-research` should either drop it from the template set, reduce it to recipes that only call `cargo` and `cargo xtask`, or record it as a house opinion through `weave-house-opinion`.
- **Hand off what other topics own:** integration-test layout and doctests to `testing.md`, `[profile.release]` to `runtime-performance.md`, MSRV checks in CI and `build.warnings = "deny"` to `ci.md`, and which lints to set to `rust.md` and `ci.md`.
- **Rust 1.100, due 2026-11-12,** may stabilise Cargo's lints. That's `refresh-rust-versions`, which would then add `missing_lints_inheritance` to the opinion.
- **No `vet-source` candidate.** The roster carried every recommendation. The Microsoft guidelines agree throughout, but they're already on the watch list until 2027-08-11.
