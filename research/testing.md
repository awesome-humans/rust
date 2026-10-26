---
targets: [rust-1.99, edition-2024]
last-reviewed: 2026-10-19
sources: [corrode, luca-palmieri, matklad, rust-blog, rust-docs, sunshowers]
---

# Testing: test runners, test layout, snapshot and property tests, and doctests

Run tests with `cargo nextest run` and doctests with `cargo test --doc`, keep the test suite to one integration-test binary per crate, and let Clippy enforce the test hygiene:

- **Run tests with cargo-nextest, and doctests with `cargo test --doc`.** Nextest can't run doctests on stable Rust, so CI needs both commands in a package with a library target. Configure a `ci` profile in `.config/nextest.toml` with `fail-fast = false` and JUnit output.
- **Put unit tests in a `#[cfg(test)] mod tests` at the end of the file they test, and integration tests in one binary,** `tests/it/main.rs`, with a module per area. Each file directly under `tests/` is a separate binary that has to be linked.
- **Keep `main.rs` thin over `lib.rs`,** so integration tests can reach the logic through `use`.
- **Write a property test, with `proptest`, when you can state what makes an output correct, and a snapshot test when you can't.**
- **Keep doctests on for a published library.** Edition 2024 merges compatible doctests into one binary, which removes most of their old compile cost.
- **Warn on the two pedantic test lints:** set `should_panic_without_expect = "warn"` and `ignore_without_reason = "warn"` in `[workspace.lints.clippy]`, unless the workspace already warns on `clippy::pedantic`. `items_after_test_module` warns by default and needs no key.

Every feature recommended here is stable in Rust 1.99.1, and cargo-nextest 0.9.148 was the latest release when the sweep ran (crates.io, read 2026-10-19). 0.9.149 has shipped since. Nextest's setup scripts are experimental, and stabilising libtest's JSON output is a 2026 project goal that's open for help. Both are in [Coming next](#coming-next).

**No opinion file covers this.** `opinions/` holds only `rust.md`, which touches testing in three places: the panic boundary (`unwrap` in tests is fine), step 4 of its edition migration ("Run `cargo build` and `cargo test`"), and the `standalone_crate` row of its edition 2024 table, under Combined doctests. This topic is the fifth in [PLAN.md: Phase 6](../PLAN.md#phase-6-first-opinions-and-templates), whose target is `opinions/testing.md`.

## How the sweep ran

Four worker agents were assigned the sweep. The `sunshowers` worker finished. The other three hit a rate limit, so the editor read `matklad`, `rust-docs`, `rust-blog`, `corrode`, `luca-palmieri`, and the crate references directly. Missing as a result: no testing posts were sought from `burntsushi`, `fasterthanlime`, `ferrous-systems`, `nnethercote`, `tokio-blog`, or `tweede-golf`. No recommendation below depends on them, but a harvest might add to it.

## Test runner: cargo-nextest

**Run `cargo nextest run` in place of `cargo test`, and run `cargo test --doc` beside it.** `corrode`, which has no stake in it, recommends adopting it, and `sunshowers` explains its design:

- `corrode` says "Use Cargo Nextest Instead of `cargo test`", which "can be up to 60% faster than `cargo test` thanks to its parallel execution model" ([Matthias Endler: Tips For Faster Rust Compile Times](https://corrode.dev/blog/tips-for-faster-rust-compile-times/), published 2024-01-12, updated 2026-03-14). Its table of 1.37× to 3.38× speed-ups across nine projects is copied from nextest's own benchmarks page, so the measurement is the vendor's, and the recommendation is `corrode`'s.
- `sunshowers` explains the design: "One test segfaulting does not take down a bunch of other tests", and the process-per-test model "forms a universal protocol that everyone in the ecosystem can rely on" ([Rain: Why nextest is process-per-test](https://sunshowers.io/posts/nextest-process-per-test/), 2025-01-07). Rain writes nextest, and the roster note forbids citing them alone on adopting it. Here `corrode` carries the adoption, and `sunshowers` carries how it works.

How nextest differs from `cargo test`, from its documentation (a reference):

| Behaviour           | `cargo test`                                                  | `cargo nextest run`                                                                                                                                         |
| ------------------- | ------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Scheduling          | One test binary at a time, so a slow test holds up its binary | Each test in its own process, scheduled across all binaries ([nextest: How nextest works](https://nexte.st/docs/design/how-it-works/))                      |
| Fail fast           | Stops at the first failing binary, unless `--no-fail-fast`    | "By default, nextest stops running tests after the first failure", and lets running tests finish ([nextest: Running tests](https://nexte.st/docs/running/)) |
| No tests matched    | Passes                                                        | Fails by default since 0.9.85, set by `--no-tests` (nextest: Running tests, above)                                                                          |
| Doctests            | Runs them                                                     | "Doctests are currently not supported because of limitations in stable Rust"                                                                                |
| Flaky tests         | No retries                                                    | `--retries 2` or a profile's `retries`, and a test that passes on retry is reported `FLAKY` ([nextest: Retries](https://nexte.st/docs/features/retries/))   |
| Slow and hung tests | No timeout                                                    | `SLOW` after 60 seconds by default, and `slow-timeout = { period = "60s", terminate-after = 5 }` kills a hung test                                          |
| CI output           | Text only on stable                                           | JUnit XML from `[profile.ci.junit]`, archives with `cargo nextest archive`, and sharding with `--partition hash:1/3`                                        |

The editor checked the fail-fast default, `--partition`, and the doctest gap against cargo-nextest 0.9.148 on Rust 1.99.1 on 2026-10-19, and the defaults for `--no-tests` and `--flaky-result` against nextest's documentation and cargo-nextest 0.9.149 on 2026-10-26.

**The cost of a process per test.** Nextest's own page says it's "quite slow on Windows", and can be slow on macOS where anti-malware scanning interferes. It also makes in-process sharing, such as a cache or an in-memory server reused across tests, "harder to achieve", and "Adapting existing tests to process-per-test may require a significant engineering effort" ([nextest: Why process-per-test?](https://nexte.st/docs/design/why-process-per-test/)). An opinion should name these as the reasons to stay on `cargo test`, not as reasons against nextest in general.

**A CI profile.** An opinion could ship this as a template. Each key is from nextest's configuration pages:

```toml
# .config/nextest.toml
[profile.ci]
fail-fast = false
slow-timeout = { period = "60s", terminate-after = 5 }

[profile.ci.junit]
path = "junit.xml"
```

CI then runs `cargo nextest run --profile ci` and `cargo test --doc`. Run the second only for a package with a library target: on a binary-only package, `cargo test --doc` fails with "no library targets found" (editor's probe on 1.99.1, 2026-10-26). `ci.md` owns the workflow around them.

### Flaky tests and retries

The sources don't settle retries. Nextest offers them, and "By default, flaky tests are treated as ultimately successful". Since 0.9.131, `--flaky-result fail` or `NEXTEST_FLAKY_RESULT=fail` fails the run instead ([nextest: Retries](https://nexte.st/docs/features/retries/)). `matklad` doesn't discuss retries, but warns that "the temptation to just re-run the test suite is enormous". He says to keep running the full suite on `main` behind a merge queue, where "every failure on main is a flake, by definition", and to "Maintain an easily accessible list of recent main failures" ([Alex Kladov: Catch Flakes On Main](https://matklad.github.io/2026/05/14/catch-flakes-on-main.html), 2026-05-14; the post is language-agnostic).

The editor's reading: leave retries off by default, so a flake fails the run and lands on that list. If a project turns retries on in the `ci` profile, track each `FLAKY` result, which the JUnit output records as `<flakyFailure>`, instead of letting it pass unseen. That's a synthesis of two sources, and an opinion needs to say it's one.

## Test layout

**Unit tests:** the Book's convention is "to create a module named `tests` in each file to contain the test functions and to annotate the module with `cfg(test)`" ([The Rust Programming Language: Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)). Clippy's `items_after_test_module` (style, 1.71) keeps that module last in the file.

**Integration tests: one binary per crate.** Three sources agree, for the same reason. Each file directly under `tests/` becomes its own crate, and "`rustc` needs to repeatedly re-link the library crate with each of the integration tests":

- `matklad` recommends "only one integration test crate with several modules", as `tests/it/main.rs`, and measured Cargo's own test suite compiling three times faster, with five times less on disk, after the change ([Alex Kladov: Delete Cargo Integration Tests](https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html), 2021-02-27).
- `corrode` says "Combine All Integration Tests Into A Single Binary", because many system linkers are single-threaded (2026-03-14, above).
- `luca-palmieri`, watch-list, uses `tests/api/main.rs` and notes that "the linking phase is instead entirely sequential!" ([Luca Palmieri: Skeleton And Principles For A Maintainable Test Suite](https://www.lpalmieri.com/posts/skeleton-and-principles-for-a-maintainable-test-suite/), 2021-02). It agrees, and carries nothing on its own.

`matklad`'s second reason, that Cargo runs test binaries one after another, doesn't apply under nextest, which schedules tests across binaries. The link-time reason still applies, so the recommendation stands either way.

The Book's `tests/common/mod.rs` convention for shared helpers becomes a plain module, `tests/it/common.rs`, in the single binary.

```text
tests/
└── it/
    ├── main.rs      // mod common; mod parse; mod serve;
    ├── common.rs
    ├── parse.rs
    └── serve.rs
```

**Binary crates:** a crate with only `src/main.rs` can't be tested from `tests/`, so keep a "straightforward _src/main.rs_ file that calls logic that lives in the _src/lib.rs_ file" (the Book, above).

**Test through the public boundary.** `matklad` says "test features, not code", and routes tests through one `check` helper per feature, so that a refactor changes the helper, not every test ([Alex Kladov: How to Test](https://matklad.github.io/2021/05/31/how-to-test.html), 2021-05-31). He replaces the unit-versus-integration split with purity and extent: "Ruthlessly optimize purity", meaning no IO and no timing, and let tests "have their natural extent" instead of mocking your own code to shrink it ([Alex Kladov: Unit and Integration Tests](https://matklad.github.io/2022/07/04/unit-and-integration-tests.html), 2022-07-04). `luca-palmieri` agrees for a web service: "All of our integration tests are black-box", with a fresh database per test so tests don't interfere (above).

### Where the roster disagrees: `[lib] test = false`

`matklad` also passes on a further step: set `[lib] test = false`, make the APIs you unit test public, and test everything from the one binary, because otherwise "the library is recompiled twice" (2021-02-27, above). The Book and `corrode` keep unit tests beside the code. The weight is with the Book's layout: `test = false` changes a library's public API to suit the build, and `matklad` frames it as a large-project measure. An opinion could mention it as the next step for a workspace whose test build time has been measured.

## Snapshot and property tests

**Prefer a property test, and fall back to a snapshot test.** `matklad`: "If I need to test something, I write a property/fuzz/randomized test. If I can't fuzz, I write a snapshot test" ([Alex Kladov: Underusing Snapshot Testing](https://matklad.github.io/2025/04/15/underusing-snapshot-testing.html), 2025-04-15). He names no property-testing crate, so the choice of `proptest` rests on `sunshowers` (below). That post's examples are Zig, but the rule is language-independent, and his 2021 Rust post makes the same case for expect tests, where "a mismatch between expected and actual causes the gold value to be updated in-place" (How to Test, above).

### Snapshot tests

`matklad` recommends inline snapshots that the tool rewrites in the source, and lists "insta, k9, expect-test" without choosing one (How to Test, above). Both crates in the table below update inline snapshots in place, so his preference doesn't separate them. No roster source compares them, so the crate facts are references only:

| Crate         | Version, crates.io 2026-10-19 | Inline snapshots | Update                                  | On CI                                                                                                                          |
| ------------- | ----------------------------- | ---------------- | --------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `insta`       | 1.49.0, 2026-10-03            | `@"…"` literal   | `cargo insta review`, or `INSTA_UPDATE` | `INSTA_UPDATE=auto` acts as `no`: "does not update snapshot files at all" ([insta: Advanced](https://insta.rs/docs/advanced/)) |
| `expect-test` | 1.5.1, 2024-12-21             | `expect![[…]]`   | `UPDATE_EXPECT=1`                       | Fails on a mismatch                                                                                                            |

`insta` also stores snapshots in files and supports redactions for timestamps and IDs, and it's the one released in the last year. `expect-test` is smaller, and it's the crate `matklad`'s examples resemble most: `expect![[…]]` and a single update variable. The editor leans to `insta` for its file snapshots and redactions, but that rests on references and the editor's reading, so the opinion needs a roster source for the choice. Its author, Armin Ronacher, is a `vet-source` candidate (see the closing section).

### Property tests

**Use `proptest`.** `sunshowers` uses it, and explains that "most modern property-based testing frameworks, like **proptest** in Rust, try and take care of shrinking for you". They warn that `prop_flat_map` is slow, because "shrinking has to be started again, from scratch, each time", so prefer `prop_map` where it fits ([Rain: Demystifying monads in Rust through property-based testing](https://sunshowers.io/posts/monads-through-pbt/), 2025-02-19). The proptest book gives the trade-off against `quickcheck`: proptest shrinks through explicit `Strategy` values and respects constraints, but generating complex values "can be up to an order of magnitude slower" ([Proptest: Differences between QuickCheck and Proptest](https://proptest-rs.github.io/proptest/proptest/vs-quickcheck.html)).

`matklad` uses `arbtest`, his own crate, in a Rust post on concurrent data structures ([Alex Kladov: Properly Testing Concurrent Data Structures](https://matklad.github.io/2024/07/05/properly-testing-concurrent-data-structures.html), 2024-07-05). It's one post, the crate's latest release is from 2024-12-18, and he wrote it, so it doesn't outweigh `sunshowers`. The same post points to `loom` for real concurrency testing: "if you need that, go read loom's docs".

| Crate        | Version, crates.io 2026-10-19 | Role                                                      |
| ------------ | ----------------------------- | --------------------------------------------------------- |
| `proptest`   | 1.12.0, 2026-10-10            | Property tests with integrated shrinking                  |
| `quickcheck` | 1.1.0, 2026-02-10             | Lost: shrinks by type alone and can't express constraints |
| `cargo-fuzz` | 0.13.2, 2026-06-09            | Coverage-guided fuzzing on nightly, through libFuzzer     |
| `loom`       | 0.7.2, 2024-04-23             | Explores every thread interleaving of concurrent code     |

`corrode` lists `cargo-fuzz` for fuzzing, and `cargo-llvm-cov` as "a good default for new projects" for coverage, which "works with `cargo test` and `cargo nextest`" ([Matthias Endler: Hardening Rust Code For Production](https://corrode.dev/blog/hardening-rust/), 2026-07-23). The same post adds `cargo test --release` in CI for "critical integration tests, arithmetic-heavy code, unsafe or FFI-heavy code", because overflow checks and optimisations differ between profiles. Miri belongs in `unsafe.md`.

## Doctests

**Keep doctests on for a published library, and make each one compile.** From the rustdoc book ([The rustdoc book: Documentation tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)):

- "Starting in the 2024 edition, compatible doctests are merged as one before being run", because "the slowest part of doctests is to compile them". Each still runs "in their own process".
- Hide setup lines with `#`, and end a doctest that uses `?` with `Ok::<(), E>(())`.
- Use `no_run` for code that needs a network or other environment, `compile_fail` for code the compiler must reject, and `should_panic` for code that must panic. `ignore` "is almost never what you want".

A doctest isn't merged if it uses `compile_fail` or an `edition` tag, uses a global attribute such as `global_allocator`, sets a crate-wide attribute, or defines a macro that uses `$crate`. Mark one that depends on line numbers or `Location` with `standalone_crate` ([Rust Edition Guide: Rustdoc combined tests](https://doc.rust-lang.org/edition-guide/rust-2024/rustdoc-doctests.html)). On 1.99.1, `cargo test --doc` reports "merged doctests compilation" as its own step (editor's probe, 2026-10-19).

### Where the roster disagrees: `doctest = false`

`matklad` says to "avoid doc tests in internal libraries for big projects", with `[lib] doctest = false`, because each doctest was linked as its own binary (2021-02-27, above). Edition 2024 removed that cost for merged doctests, so his reason now applies only to the doctests that can't be merged. The editor's reading: on edition 2024, set `doctest = false` only on an unpublished crate whose doc comments carry no examples, where it saves the doctest build step and loses nothing. That needs a measurement or a second source before an opinion rests on it.

## Assertions and test hygiene

**Use `assert_matches!` for a pattern.** It's stable since 1.96 ([Rust Blog: Announcing Rust 1.96.0](https://blog.rust-lang.org/2026/05/28/Rust-1.96.0/)), and it prints the value on failure. It isn't in the prelude: import it with `use std::assert_matches;` (editor's probe on 1.99.1, 2026-10-19).

```rust
/// Parses a TCP port.
///
/// # Errors
///
/// Returns an error if `s` isn't a number that fits in a `u16`.
pub fn parse_port(s: &str) -> Result<u16, std::num::ParseIntError> {
    s.trim().parse()
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::parse_port;

    #[test]
    fn trims_whitespace() {
        assert_eq!(parse_port(" 8080\n"), Ok(8080));
    }

    #[test]
    fn rejects_out_of_range() {
        assert_matches!(parse_port("65536"), Err(_));
    }

    #[test]
    #[ignore = "binds a real socket; run with --run-ignored all"]
    fn binds() {}
}
```

This passes `cargo clippy --all-targets -- -D warnings -W clippy::pedantic` and `cargo nextest run --run-ignored all` on 1.99.1 (editor's probe, 2026-10-19).

**Lints.** Groups and versions are from the `#[clippy::version]` attributes in rust-clippy's source, read 2026-10-19:

| Lint                          | Group       | Added | Enforces                                                     |
| ----------------------------- | ----------- | ----- | ------------------------------------------------------------ |
| `items_after_test_module`     | style       | 1.71  | The `tests` module comes last in its file                    |
| `should_panic_without_expect` | pedantic    | 1.74  | `#[should_panic(expected = "…")]`, so the right panic passes |
| `ignore_without_reason`       | pedantic    | 1.88  | `#[ignore = "reason"]`                                       |
| `tests_outside_test_module`   | restriction | 1.70  | Every `#[test]` inside a `#[cfg(test)]` module               |
| `redundant_test_prefix`       | restriction | 1.88  | No `test_` prefix on a test function's name                  |

The pedantic pair comes with `clippy::pedantic`. `templates/Cargo.toml` doesn't warn on that group, and turns on pedantic lints one at a time, as it does `manual_let_else`, so the opinion should name the pair as keys. The two restriction lints match the Book's layout, but `tests_outside_test_module` also fires on a `#[test]` in `tests/it/` unless it's wrapped in a `#[cfg(test)]` module too (editor's probe on 1.99.1, 2026-10-26), so an opinion should leave it off.

**`unwrap` in tests.** `opinions/rust.md` already says `unwrap` is fine in tests. A workspace that turns on `clippy::unwrap_used` or `expect_used` needs `allow-unwrap-in-tests = true` and `allow-expect-in-tests = true` in `clippy.toml`, because both default to `false` (`cargo clippy --explain unwrap_used` on 1.99.1).

## Changes in recent releases

Testing changes from [RELEASES.md](https://github.com/rust-lang/rust/blob/stable/RELEASES.md), the release notes the `rust-blog` announcements link to:

| Release | Change                                                                                                    |
| ------- | --------------------------------------------------------------------------------------------------------- |
| 1.85    | Edition 2024 merges compatible doctests                                                                   |
| 1.86    | Fixes doctest merging, and deprecates libtest's `--logfile`                                               |
| 1.88    | `--no-capture` replaces `--nocapture`, and doctests take `ignore-<target>`                                |
| 1.89    | `cargo test --doc --target <other>` runs cross-compiled doctests through the target's `runner`            |
| 1.91    | An invalid `#[should_panic]` attribute is an error                                                        |
| 1.93    | `#[test]` on an item where it means nothing, such as a trait method, is an error instead of being ignored |
| 1.96    | `assert_matches!` and `debug_assert_matches!`                                                             |
| 1.99    | `doc(cfg())` no longer filters doctests                                                                   |

The 1.89 change can break CI that cross-compiles: doctests that never ran now do ([Rust Blog: Announcing Rust 1.89.0](https://blog.rust-lang.org/2025/08/07/Rust-1.89.0/)).

## Coming next

Not stable as of 2026-10-19, not yet an opinion.

| Feature               | Status, 2026-10-19                                                                                                                                                                                                                 |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| libtest JSON output   | Unstable on nightly. "Finish the libtest json output experiment" is a 2026 project goal for the cargo, libs, and testing-devex teams, marked Help Wanted ([Rust Project Goals: 2026](https://goals.rust-lang.org/2026/goals.html)) |
| nextest setup scripts | Experimental in cargo-nextest, enabled with `experimental = ["setup-scripts"]` in `.config/nextest.toml` ([nextest: Setup scripts](https://nexte.st/docs/configuration/setup-scripts/))                                            |
| Doctests in nextest   | Blocked on stable Rust, by nextest's own account. Nothing scheduled                                                                                                                                                                |

Stable libtest JSON output would give `cargo test` the structured results that nextest gives today. If it lands, the runner recommendation needs a fresh look.

## Watch-list and unvetted material

Kept apart from the roster findings above.

- **Watch-list `luca-palmieri`:** agrees on the single integration binary and black-box tests, as quoted above. It carried no claim alone.
- **Unvetted:** no source outside the roster carried a claim. Searches for modern Rust testing guidance returned generated list articles and tutorials, which were discarded.
- **References:** the nextest, insta, and proptest documentation, the crates.io version data, and rust-clippy's source were used for facts about those tools, never for whether to adopt them.

## What this means for the repository

- **No opinion covers the topic.** Run `resolve-research` to create `opinions/testing.md`. Its first sentence is the opening list here. It needs sections for the runner and its CI profile, the test layout, snapshot and property tests, doctests, and the lints. List it under README.md's Scope and in the layout tree.
- **The expected sources need one swap.** PLAN.md expects `sunshowers`, `matklad`, and `luca-palmieri`. `luca-palmieri` is watch-listed and can't be cited, and `sunshowers` can't carry nextest adoption alone. `corrode` fills both gaps.
- **Avoid repeating `rust.md`.** `opinions/testing.md` should link the `standalone_crate` row in `rust.md` instead of repeating it. Step 4 of the migration can keep `cargo test`, because it also runs the doctests that nextest can't. Leave `rust.md` unchanged.
- **Templates:** add `templates/.config/nextest.toml` with the `ci` profile above, and add `should_panic_without_expect` and `ignore_without_reason` to `[workspace.lints.clippy]` in `templates/Cargo.toml`. Don't add a `templates/clippy.toml` for `allow-unwrap-in-tests`: no template turns on `unwrap_used` or `expect_used`, so the keys would do nothing. The single integration binary fits the planned example crates under `templates/projects/`.
- **Leave the CI workflow, Miri, and async tests to their own topics.** `ci.md` owns the commands' place in a workflow, `unsafe.md` owns Miri, and `async.md` owns `#[tokio::test]` and paused time.
- **`vet-source` candidate:** Armin Ronacher ([Armin Ronacher's Thoughts and Writings](https://lucumr.pocoo.org/)), the author of `insta`, if the opinion names a snapshot crate. He'd carry a conflict on adopting `insta`, like `sunshowers` on nextest.
- **A harvest could fill the gap the rate limit left:** `burntsushi`, `tokio-blog`, and the others listed in [How the sweep ran](#how-the-sweep-ran) weren't searched for testing posts.
