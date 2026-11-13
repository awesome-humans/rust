---
targets: [rust-1.99, edition-2024]
last-reviewed: 2026-10-20
sources: [anssi-rust, corrode, rust-blog, rust-docs, tweede-golf]
---

# Security: supply chain and secure coding

Commit `Cargo.lock`, build with `--locked`, and gate every pull request on `cargo deny check` with a strict `deny.toml`. In code, forbid `unsafe` unless a crate needs it, make overflow and panics explicit, and keep secrets out of `Debug`, logs, and panic messages:

- **Pin the graph:** commit `Cargo.lock` in every repository, binaries and libraries alike, and pass `--locked` in CI and to `cargo install`.
- **Check it on every pull request:** `cargo deny check` with `unsound = "all"`, `yanked = "deny"`, `wildcards = "deny"`, and `unknown-registry` and `unknown-git` set to `"deny"`. It reads the same RustSec database as `cargo-audit`, and adds bans, sources, and licences.
- **Keep the graph small:** treat each new dependency, and each new build script or proc macro, as code you run on every developer machine and CI runner.
- **Publish with trusted publishing:** a library released from GitHub Actions or GitLab.com CI uses crates.io's OIDC tokens, not a stored API token, and its owners turn on the setting that requires it.
- **Forbid `unsafe` by default:** set `unsafe_code = "forbid"` in `[workspace.lints.rust]`, inherit it with `[lints] workspace = true` in each member, and leave it out only in a crate that justifies `unsafe`.
- **Make failure explicit on untrusted input:** `checked_*` arithmetic, `get` over indexing, and no `unwrap` outside tests and invariants, with the restriction lints that enforce them turned on in security-critical crates.
- **Keep secrets out of output:** a hand-written `Debug`, no secret in a panic message, and no reliance on `Drop` alone to erase one.

Every tool and setting here works on stable Rust 1.99.1, and the editor probed the Cargo and Clippy settings on it on 2026-10-20. `cargo-vet` is the one recommendation that depends on the project: it suits software that must record a review of each dependency, as ANSSI's rules require, and costs a small project more than it returns. Two parts are unsupported until `resolve-research` settles them, because they rest on the cargo-deny book, a reference: the `deny.toml` values, and running `cargo-deny` without `cargo-audit`.

**No opinion file covers this.** `opinions/rust.md` covers the safe-Rust pitfalls that overlap (checked arithmetic, `as` casts, indexing, `Debug` on secrets), and nothing covers the supply chain. This is the `security.md` topic in [PLAN.md: Phase 6](../PLAN.md#phase-6-first-opinions-and-templates).

## Supply chain

### The threat is real and recent

On 2026-08-20, a compromised maintainer account published `arrayref` 0.3.10, `internment` 0.8.7, and `append-only-vec` 0.1.9, each with a new dependency on `proc-macro1`, whose build script downloaded a malicious payload. The versions were online for 86 to 107 minutes ([Rust Blog: Supply chain attack on arrayref](https://blog.rust-lang.org/2026/08/20/supply-chain-attack-on-arrayref), 2026-08-20). The RustSec advisory records that `arrayref` 0.3.10 was downloaded 2,285 times, "under 10% of `arrayref` download traffic, because most users had older versions locked" ([RUSTSEC-2026-0260](https://rustsec.org/advisories/RUSTSEC-2026-0260), a reference). The blog's only advice was to check local dependencies.

Look-alike crates are routine. The Rust Blog reported `faster_log` and `async_println` (2025-09-24), `evm-units` and `uniswap-utils` (2025-12-03), and `finch-rust` and `sha-rust` (2025-12-05). On 2026-02-13 the crates.io team stopped posting each one: "We will always publish a RustSec advisory when a crate is removed for containing malware", and a blog post only when a crate saw real use ([Rust Blog: An update to the malicious crate notification policy](https://blog.rust-lang.org/2026/02/13/crates.io-malicious-crate-update)). So the RustSec database, which `cargo-deny` reads, is now the one channel that reports every malicious crate.

Cargo itself had three advisories in 2026: CVE-2026-33056 (fixed in 1.94.1), and CVE-2026-5222 and CVE-2026-5223 (both fixed in 1.96.0). crates.io blocked uploads exploiting the first and found none published. The second affects only third-party sparse registries that host several registries under one domain, and crates.io forbids the symlinks the third needs ([Rust Blog: Security advisory for Cargo](https://blog.rust-lang.org/2026/03/21/cve-2026-33056), 2026-03-21, [Security Advisory for Cargo (CVE-2026-5222)](https://blog.rust-lang.org/2026/05/25/cve-2026-5222/), 2026-05-25, and [Security Advisory for Cargo (CVE-2026-5223)](https://blog.rust-lang.org/2026/05/25/cve-2026-5223/), 2026-05-25). All three are a reason to stay on the latest stable toolchain. `rust.md` targets Rust 1.99 but has no rule that says so, so the security opinion would have to state it.

### The lockfile and `--locked`

Commit `Cargo.lock` in every repository. ANSSI makes it a rule: "`Cargo.lock` files MUST be tracked by a version control system", because the lockfile records a checksum for each registry dependency and the build fails if one changes ([ANSSI: DENV-CARGO-LOCK](https://anssi-fr.github.io/rust-guide/devenv.html#DENV-CARGO-LOCK)). A git dependency is pinned to a commit instead, and a path dependency isn't pinned at all. The Cargo Book gives the same default: "When in doubt, check `Cargo.lock` into the version control system" ([The Cargo Book: Cargo.toml vs Cargo.lock](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html), read 2026-11-12). Its FAQ says the choice is "dependent on the needs of your package", notes that `cargo new` tracks the file by default, and warns that the lockfile doesn't reach consumers: "`cargo install` will select the latest dependencies unless `--locked` is passed in" ([The Cargo Book: FAQ](https://doc.rust-lang.org/cargo/faq.html#why-have-cargolock-in-version-control)). The `arrayref` numbers above show what a committed lockfile prevents. They come from the RustSec advisory, a reference, so they illustrate the rule and are unsupported as a claim of their own.

- **Build with `--locked` in CI,** so a pull request can't change the graph without changing `Cargo.lock` in the diff. `corrode` says the same for container builds ([Matthias Endler: Hardening Rust Code For Production](https://corrode.dev/blog/hardening-rust/), 2026-07-23).
- **Install tools with `cargo install --locked`.**
- **Update through pull requests,** with Renovate or Dependabot, so each update runs the checks below. The Cargo Book lists this among its options for testing newer dependencies ([The Cargo Book: Verifying Latest Dependencies](https://doc.rust-lang.org/cargo/guide/continuous-integration.html#verifying-latest-dependencies)). A pinned graph stays exposed to advisories filed after it was pinned, so updates still have to happen.

### `cargo-deny` over `cargo-audit`

Run `cargo deny check` in CI. `corrode` recommends `cargo-audit` and `cargo-deny`, "as part of CI" (2026-07-23, above). `tweede-golf` recommended `cargo audit` and `cargo vet`, then added that `cargo deny` "should really have been in the list of recommendations" ([Tweede golf: Dealing with Dependencies in Rust](https://tweedegolf.nl/en/blog/104/dealing-with-dependencies-in-rust/), 2023-10-30, with a later update).

ANSSI disagrees on the tool, not the intent: "The `cargo-audit` tool MUST be used to check for known vulnerabilities in dependencies" ([ANSSI: LIBS-AUDIT](https://anssi-fr.github.io/rust-guide/libraries.html#LIBS-AUDIT)), and it never mentions `cargo-deny`. Both tools read the RustSec Advisory Database. `cargo-deny`'s `advisories` check does the same job, and adds `bans`, `sources`, and `licenses` under one `deny.toml`.

**Unsupported: `cargo-deny` without `cargo-audit`.** Every roster source that covers the subject names `cargo-audit`: ANSSI requires it, `corrode` recommends it beside `cargo-deny`, and `tweede-golf` recommended it before adding `cargo-deny`. None prefers `cargo-deny` over it, and only `corrode` ties either tool to CI. That one check can replace the other is the editor's reading of the cargo-deny book, a reference. So `resolve-research` decides between `cargo-deny` alone, on that reading, and both tools, as the roster says. A project that must comply with ANSSI's rules exactly runs `cargo audit` either way. `anssi-rust`'s roster notes say to check a rule against `rust-docs` before an opinion rests on it, because the guide calls itself unstable.

Tool state, read from crates.io and GitHub on 2026-10-19 (references):

| Tool              | Version | Released   | Use here                                                                                                                                    |
| ----------------- | ------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `cargo-deny`      | 0.20.2  | 2026-07-09 | The CI gate. `EmbarkStudios/cargo-deny-action` v2.1.1 (2026-07-13) runs it                                                                  |
| `cargo-audit`     | 0.22.2  | 2026-06-05 | Proposed to lose to `cargo-deny`, which covers its advisories check: unsupported, as above. `cargo audit bin` still audits a shipped binary |
| `cargo-vet`       | 0.10.2  | 2026-01-13 | For software that must record a review of each dependency. See below                                                                        |
| `cargo-auditable` | 0.7.7   | 2026-10-02 | Embeds the dependency list in a binary, so `cargo audit bin` can check it after release                                                     |
| `cargo-outdated`  | 0.19.0  | 2026-04-14 | Lost to Renovate or Dependabot, which open the update instead of reporting it                                                               |
| `cargo-geiger`    | 0.14.0  | 2026-10-12 | Counts `unsafe` in the graph. `corrode` names it, but it gates nothing, so it belongs in `unsafe.md` if anywhere                            |

Advisory check, 2026-11-12: the RustSec Advisory Database has no advisory against `cargo-deny`, `cargo-audit`, `cargo-vet`, or `cargo-auditable` at any version ([RustSec Advisory Database](https://rustsec.org/)). The lookup covers each tool's own crate, not the crates it depends on.

#### `deny.toml` settings

**Unsupported.** No roster source gives a `deny.toml` value. Every setting in this section is the editor's reading of the cargo-deny book, a reference, so `resolve-research` needs a roster source or a house opinion before it promotes one.

The defaults are lenient on the checks that matter for the supply chain. From the cargo-deny book ([advisories](https://embarkstudios.github.io/cargo-deny/checks/advisories/cfg.html), [bans](https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html), [sources](https://embarkstudios.github.io/cargo-deny/checks/sources/cfg.html), references, read 2026-10-19, and the advisories, bans, and licences pages again 2026-11-12):

| Key                           | Default     | Proposed | Why                                                                                                               |
| ----------------------------- | ----------- | -------- | ----------------------------------------------------------------------------------------------------------------- |
| `[advisories] unsound`        | `workspace` | `all`    | The default checks direct dependencies only, so an unsound advisory on a transitive crate passes                  |
| `[advisories] unmaintained`   | `all`       | `all`    | Keep it. An unmaintained crate gets no fix for the next advisory                                                  |
| `[advisories] yanked`         | `warn`      | `deny`   | A yanked version in the lockfile is a release its author withdrew                                                 |
| `[bans] wildcards`            | `warn`      | `deny`   | A `*` requirement accepts any future release, including a malicious one                                           |
| `[bans] allow-wildcard-paths` | unset       | `true`   | With `wildcards = "deny"`, exempts path and git dependencies in private crates, and dev-dependencies in any crate |
| `[bans] multiple-versions`    | `warn`      | `warn`   | Duplicates cost build time and review effort, but denying them blocks too many routine updates                    |
| `[sources] unknown-registry`  | `warn`      | `deny`   | Only crates.io, unless a registry is listed in `allow-registry`                                                   |
| `[sources] unknown-git`       | `warn`      | `deny`   | A git dependency bypasses crates.io's checks. List each in `allow-git`                                            |
| `[licenses] allow`            | none        | a list   | Licence policy, not security. It's here because `cargo deny check` runs it too                                    |

Leave out `version = 2` under `[advisories]` and `[licenses]`: the book says the field "is (at the time of this writing) no longer used". Vulnerability and notice advisories always fail the check, because the keys that set their level were removed.

An ignored advisory goes in `[advisories] ignore` with its ID and a reason, the same discipline as `#[expect(…, reason = "…")]` in `rust.md`.

`[bans.build]` can deny crates that ship native executables (on by default), and `allow-build-scripts` can list the only crates allowed a build script. If that key is omitted, every crate may have one. An allow-list would have failed the check on the `arrayref` update, because `proc-macro1` wasn't on it, but only where `cargo deny check` runs before the build, as in CI. It doesn't protect a developer who runs `cargo build` first, and it doesn't sandbox a build script it allows. The documentation states the limit: "the compile time crate linting supplied by cargo-deny does **NOT** protect you from actively malicious code." The allow-list's cost is a review each time a dependency adds a build script, so `resolve-research` should decide whether the template sets it. By the RustSec advisory's count, the lockfile protected most users with no setting at all.

### Fewer dependencies, especially build-time ones

A build script or proc macro runs with the developer's privileges at build time, and the Cargo Book's build-script chapter has no warning about it ([The Cargo Book: Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html), checked 2026-10-19). `tweede-golf` cut `sudo-rs` from about 135 transitive dependencies to three, `libc`, `glob`, and `log`, because "each additional dependency is another team to trust and another codebase to validate", and replaced `clap` with its own argument parsing ([Tweede golf: Sudo-rs dependencies: when less is better](https://tweedegolf.nl/en/blog/119/sudo-rs-depencencies-when-less-is-better), 2024-03-07). That's a `setuid` program, an extreme case. The general rule the sources support: assess a new dependency's transitive graph, not just the crate, and prefer `std` where a release made the crate unnecessary, as `rust.md` already lists.

### Recording dependency review: `cargo-vet`

ANSSI requires review of every direct dependency: "Each direct third-party dependency MUST be properly validated, and each validation MUST be tracked" ([ANSSI: LIBS-VETTING-DIRECT](https://anssi-fr.github.io/rust-guide/libraries.html#LIBS-VETTING-DIRECT)), and recommends the same for transitive ones ([LIBS-VETTING-TRANSITIVE](https://anssi-fr.github.io/rust-guide/libraries.html#LIBS-VETTING-TRANSITIVE)). It names no tool. `cargo-vet` is the tool that records it: it matches dependencies against audits, imports audits from other organisations, and starts a project with an exemptions list "ratcheted down over time" ([cargo-vet book](https://mozilla.github.io/cargo-vet/), a reference). `tweede-golf` recommends it for a sense of "how many eyeballs have looked at" a dependency (2023-10-30, above).

No roster source makes it a default for every project, and the review costs time. Proposed split: `cargo-vet` for software ANSSI's scope covers, such as privileged, network-facing, or safety-critical code, and `cargo-deny` alone elsewhere. `resolve-research` should decide whether the opinion states that split or leaves `cargo-vet` to a single line.

### Publishing

crates.io's trusted publishing, shipped 2025-07-11, "eliminates the need for GitHub Actions secrets when publishing crates": a workflow with `id-token: write` exchanges an OIDC token for a short-lived publish token through `rust-lang/crates-io-auth-action@v1`. It launched for GitHub Actions only, and the first release still needs a manual publish ([Rust Blog: crates.io development update](https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07), 2025-07-11). On 2026-01-21 it added GitLab CI/CD, "currently only" on GitLab.com, and a crate setting that requires it: "Crate owners can now enforce Trusted Publishing for their crates", which disables "traditional API token-based publishing" ([Rust Blog: crates.io development update](https://blog.rust-lang.org/2026/01/21/crates-io-development-update/), 2026-01-21). Turn that setting on once the workflow publishes, so a stolen API token can't publish the crate. The `arrayref` attack used an author's credentials or machine, and the post on it doesn't say which, so it can't show what trusted publishing would have prevented.

## Secure coding

The roster scan here is narrower than for the supply chain. A worker covering the secure-coding sources stopped on a rate limit, and the editor read ANSSI and `corrode`'s hardening post in full, but not the rest of the roster on fuzzing, TLS, or cryptography crates. For the follow-up scan, see [What this means for the repository](#what-this-means-for-the-repository).

### `unsafe`

ANSSI: "`unsafe` blocks SHOULD be avoided, or MUST be justified" by FFI, memory-mapped hardware, or a measured performance limit, and otherwise "`#![forbid(unsafe_code)]` must appear in the crate root" ([ANSSI: LANG-UNSAFE](https://anssi-fr.github.io/rust-guide/unsafe/generalities.html#LANG-UNSAFE)). Since Rust 1.74, `[lints]` in `Cargo.toml` does the same for a whole workspace, which ANSSI doesn't mention. Set it in the workspace root:

```toml
[workspace.lints.rust]
unsafe_code = "forbid"
```

Cargo doesn't apply it to a member until the member opts in, in its own `Cargo.toml`:

```toml
[lints]
workspace = true
```

A crate that needs `unsafe` can't inherit and override at once. Cargo 1.99 rejects the manifest with "cannot override `workspace.lints` in `lints`" (editor's probe, 2026-10-20). So that crate drops `workspace = true` and lists its lints in its own `[lints.rust]` and `[lints.clippy]`, which makes the exception visible in its manifest. Soundness, Miri, `unsafe_op_in_unsafe_fn`, and ANSSI's FFI rules belong to `unsafe.md`.

The cost is that such a crate copies the workspace's other lints, and the copy can fall behind `[workspace.lints.clippy]`. Setting `unsafe_code = "deny"` in the workspace avoids that: the crate keeps `workspace = true` and marks each use with `#[expect(unsafe_code, reason = "…")]`, as `rust.md` requires for any suppression. Under `forbid`, rustc rejects that attribute with E0453 (editor's probe on Rust 1.99.1, 2026-11-12). `deny` lost because ANSSI's rule says `forbid`, and because under `deny` any module can opt out with one attribute and no change to a manifest. `resolve-research` should confirm that choice.

### Overflow, indexing, and panics

The release profile defaults to `overflow-checks = false` and `debug-assertions = false` ([The Cargo Book: Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html#overflow-checks)), and the Reference says integer operators panic on overflow only "when compiled in debug mode" ([The Rust Reference: Overflow](https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow)). ANSSI: "the usual operators MUST NOT be used directly" where an operation can overflow. Use `checked_`, `overflowing_`, `wrapping_`, or `saturating_` methods, or `Wrapping` and `Saturating` ([ANSSI: LANG-ARITH](https://anssi-fr.github.io/rust-guide/integer.html#LANG-ARITH)). It shows `overflow-checks = true` for release as an option, not a rule. `rust.md` already says the same, citing `corrode`.

ANSSI limits panics: "A Rust function MUST NOT panic UNLESS its usage conditions have been violated", and "Uses of `unwrap`, `expect` and `assert!` MUST be restricted" ([ANSSI: LANG-LIMIT-PANIC](https://anssi-fr.github.io/rust-guide/errors.html#LANG-LIMIT-PANIC), [LANG-LIMIT-PANIC-SRC](https://anssi-fr.github.io/rust-guide/errors.html#LANG-LIMIT-PANIC-SRC)). It names no lint. `corrode` does: turn on `indexing_slicing` and `arithmetic_side_effects` "to catch common panic sources", which "can be noisy in applications" but suit code where "panic freedom matters more than convenience" (2026-07-23, above).

The lints that turn these rules into checks, from [Clippy Lints](https://rust-lang.github.io/rust-clippy/stable/index.html), read 2026-10-19. All are allow-by-default:

| Lint                         | Group       | Added | Enforces                                 | Rule                                  |
| ---------------------------- | ----------- | ----- | ---------------------------------------- | ------------------------------------- |
| `arithmetic_side_effects`    | restriction | 1.64  | No plain `+`, `-`, `*` that can overflow | ANSSI `LANG-ARITH`, `corrode`         |
| `indexing_slicing`           | restriction | <1.29 | `get` over `[]`                          | ANSSI `LANG-ARRINDEXING`, `corrode`   |
| `unwrap_used`, `expect_used` | restriction | 1.45  | No `unwrap` or `expect`                  | ANSSI `LANG-LIMIT-PANIC-SRC`          |
| `panic`                      | restriction | 1.40  | No `panic!`                              | ANSSI `LANG-LIMIT-PANIC`              |
| `cast_possible_truncation`   | pedantic    | <1.29 | `TryFrom` over a narrowing `as`          | `rust.md`                             |
| `mem_forget`                 | restriction | <1.29 | No `mem::forget`                         | ANSSI `MEM-FORGET`, `MEM-FORGET-LINT` |

Turning all of these on in every crate would conflict with `rust.md`, which allows `unwrap` where a panic means a bug. Proposed: the opinion names them as the set for a crate that parses untrusted input or runs privileged, enabled in that crate's `[lints.clippy]`, with tests exempted through four `clippy.toml` keys: `allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-panic-in-tests`, and `allow-indexing-slicing-in-tests` ([Clippy Documentation: Lint Configuration](https://doc.rust-lang.org/clippy/lint_configuration.html), read 2026-11-12). `arithmetic_side_effects` has no such key and needs none.

The editor's probes on Clippy 1.99.1 (2026-10-20 and 2026-11-12) confirmed it. With the lints set to warn and the four keys set, `cargo clippy --all-targets` reported the `unwrap` and the `+` in library code, and nothing inside `#[cfg(test)]`. With only the first two keys, it also reported a `panic!` and a `v[0]` in a test. The `panic` lint doesn't cover `assert!`, which LANG-LIMIT-PANIC-SRC also restricts: the probe reported nothing for an `assert!` in library code, and no lint in the table does.

Bound everything that input controls. `corrode`: "Set explicit limits on everything", and bound recursion depth or rewrite it iteratively (2026-07-23, above).

### Secrets

- **`Debug` and logs:** write `Debug` by hand for a type that holds a secret (`rust.md`, from `corrode`). `corrode` adds: "avoid putting secrets or raw user data into panic messages in the first place", and sanitise in a panic hook (2026-07-23, above).
- **Erasure:** "Ensuring security operations at the end of some treatment (such as key erasure at the end of a cryptographic encryption) MUST NOT rely only on the `Drop` trait implementation", because a reference cycle, `mem::forget`, or an abort skips it ([ANSSI: LANG-DROP-SEC](https://anssi-fr.github.io/rust-guide/standard.html#LANG-DROP-SEC)). ANSSI doesn't say what to use instead, and names neither `zeroize` (1.9.1, 2026-10-06) nor `secrecy` (0.10.3, 2024-10-09). The scan found no roster source that recommends either, so the opinion can name the problem but not the crate.
- **`panic = "abort"`:** `corrode` shows it for release builds (2026-07-23, above). ANSSI warns that abort skips `Drop` ([ANSSI: errors](https://anssi-fr.github.io/rust-guide/errors.html)), which LANG-DROP-SEC already accounts for. Leave the panic strategy to `error-handling.md`.

### Toolchain and build environment

ANSSI also requires a stable toolchain ([DENV-STABLE](https://anssi-fr.github.io/rust-guide/devenv.html#DENV-STABLE)), leaving `debug-assertions` and `overflow-checks` at their defaults in `[profile.dev]` and `[profile.test]` ([DENV-CARGO-OPTS](https://anssi-fr.github.io/rust-guide/devenv.html#DENV-CARGO-OPTS)), and not overriding `RUSTC`, `RUSTC_WRAPPER`, or `RUSTFLAGS`, so flags stay in configuration files ([DENV-CARGO-ENVVARS](https://anssi-fr.github.io/rust-guide/devenv.html#DENV-CARGO-ENVVARS)). The first agrees with this repository's targets. The third is consistent with `rust.md`, which denies warnings through Cargo's `build.warnings`, stable since 1.97, and not through `cargo clippy -- -D warnings`. `rust.md` doesn't mention `RUSTFLAGS`, so the rule against overriding it belongs to `ci.md`.

## Where ANSSI is out of date

The guide's latest content change is 2026-04-07, and its environment chapter describes the move from Rust 2021 to Rust 2024 ([ANSSI: devenv](https://anssi-fr.github.io/rust-guide/devenv.html), read 2026-11-12), but some tool rules predate the current tools:

- **LIBS-OUTDATED** makes `cargo-outdated` a MUST. An update bot does the same job and opens the change, and the Cargo Book names Renovate and Dependabot.
- **LIBS-AUDIT** names `cargo-audit` and not `cargo-deny`, as above.
- **LANG-UNSAFE** and **MEM-FORGET-LINT** use crate-root attributes, `#![forbid(unsafe_code)]` and `#![deny(clippy::mem_forget)]`. The `[lints]` table has done this per workspace since 1.74.
- **The rustup warning** that rustup "does not yet validate signatures of downloads" wasn't checked against current rustup. The opinion shouldn't repeat it unchecked.
- **Fuzzing** is a commented-out TODO chapter, and async is out of scope.

## Watch-list and unvetted material

Kept apart from the roster findings above. None of it supported a claim.

- **Watch-list `ms-rust-guidelines`:** M-STATIC-VERIFICATION names `cargo-audit` for dependencies and Miri for `unsafe`, and M-UNSAFE says `unsafe` "needs reason, should be avoided" ([Microsoft: Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/)). It agrees with ANSSI and `corrode`.
- **Watch-list `effective-rust`:** not checked in this scan.
- **Unvetted:** StepSecurity, Wiz, and The Hacker News covered `arrayref`. The editor used only the Rust Blog post and the RustSec advisory, and the press reports disagree on whose account was compromised. They aren't `vet-source` candidates, because they're vendor incident write-ups, not Rust guidance.

## What this means for the repository

- **No opinion covers the topic.** Run `resolve-research` to create `opinions/security.md`, with a supply-chain section (lockfile, `cargo-deny`, fewer dependencies, `cargo-vet` split, trusted publishing) and a secure-coding section (`unsafe_code = "forbid"`, the restriction-lint set for untrusted-input crates, secrets). List it under README.md's Scope and in the layout tree.
- **`templates/deny.toml` is the main template work.** PLAN.md lists it among the phase 6 templates. It takes the settings in the `deny.toml` table once they're supported: they rest on the cargo-deny book alone, so `resolve-research` needs a roster source or a house opinion for them first. `corrode` and `tweede-golf` support running `cargo-deny`, so the first-line header can cite them. It can't cite `rust-docs`, which doesn't cover the cargo-deny book. The licence allow list needs a choice this topic doesn't make.
- **`cargo-deny` without `cargo-audit` is unsupported.** Every roster source names `cargo-audit`, so `resolve-research` chooses between `cargo-deny` alone and both tools.
- **`rust.md` has no rule to stay on the latest stable toolchain.** The Cargo advisories are the reason for one, and the security opinion would have to state it.
- **`templates/Cargo.toml` gains `[workspace.lints.rust] unsafe_code = "forbid"`,** if `resolve-research` agrees. Its header comment already tells each crate to inherit with `[lints] workspace = true`, and `unsafe.md` should say how a crate that needs `unsafe` lists its own lints instead.
- **Hand off to later topics:** running `cargo deny check` in CI, SHA-pinning `cargo-deny-action`, and `--locked` go to `ci.md`. Miri, `cargo-geiger`, and ANSSI's FFI rules go to `unsafe.md`. `panic = "abort"` goes to `error-handling.md`.
- **The secure-coding scan needs a second pass** before the opinion's secure-coding section is final: fuzzing (`cargo-fuzz`), TLS and cryptography crate choice, and deserialization limits weren't scanned across the roster. Run `research-topic` again on those, or have `resolve-research` scan them.
- **No `vet-source` candidate, and one roster note to correct.** ANSSI, `corrode`, `tweede-golf`, `rust-blog`, and `rust-docs` supported every claim that isn't marked unsupported. The marked ones rest on references: the RustSec advisory, and the cargo-deny book, which is a crate's own documentation that `rust-docs` doesn't cover. `anssi-rust`'s note says the guide "names no Rust version or edition", but its environment chapter names Rust 2021, Rust 2024, and version 1.29, and recommends no edition. `vet-source` should reword the note.
