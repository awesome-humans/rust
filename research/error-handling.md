---
targets: [rust-1.99, edition-2024]
last-reviewed: 2026-10-19
sources:
  [
    burntsushi,
    corrode,
    matklad,
    nnethercote,
    rust-blog,
    rust-docs,
    sunshowers,
    yoshua-wuyts,
  ]
---

# Error handling: library and application errors, `thiserror` and `anyhow`, and panics

Choose an error's shape by how its caller uses it, and let the library-or-application split follow from that:

- **Give a caller that branches on the failure a concrete error type.** In a library, make it an opaque struct that boxes its representation, with a `kind()` method that returns a `#[non_exhaustive]` fieldless enum. Derive `std::error::Error` with `thiserror` 2.
- **Report a failure that nobody branches on with `anyhow::Error`.** That's most of an application: `fn main() -> anyhow::Result<()>`, with `.context()` or `.with_context()` at each layer that knows what it was doing.
- **Write `Display` as a short lowercase phrase with no trailing punctuation. Return the cause from `source()` and leave it out of the message, so it never appears in both.**
- **Panic only on a bug.** Prefer `expect("… should …")` to `unwrap()`, but keep `unwrap()` where a message would only repeat the code. Don't turn on `clippy::unwrap_used` across a codebase.
- **Document both:** a `# Errors` section on every public function that returns `Result`, and `# Panics` on one that can panic. Turn on `clippy::missing_errors_doc` and `clippy::missing_panics_doc` to enforce it.

Everything here is stable in Rust 1.99.1 (released 2026-10-15), with `thiserror` 2.0.21 (2026-09-23) and `anyhow` 1.0.104 (2026-07-18). Their `rust-version` is 1.77 and 1.68. The never type reaches stable in 1.100. The rest of [Coming next](#coming-next) is unstable on nightly as of 2026-10-19.

**Partial coverage exists.** `opinions/rust.md` has a Panics section, which fixes the `Result`-or-panic boundary and says "For error types, see `error-handling.md`, which isn't written yet". No opinion covers error types, `thiserror`, `anyhow`, `Display` and `source()`, or the documentation lints. This is the third topic in [PLAN.md: Phase 6](../PLAN.md#phase-6-first-opinions-and-templates), whose target is `opinions/error-handling.md`.

## Ask how the error will be used

`matklad` puts the design question first: "When designing an Error type for use with `Result<T, E>`, the main question to ask is 'how the error will be used?'" Either the consumer inspects the error, "so its internal structure needs to be exposed to a reasonable degree", or it propagates the error to a person, which "is typically handled by using a boxed trait object" ([Alex Kladov: Study of std::io::Error](https://matklad.github.io/2020/10/15/study-of-std-io-error.html), 2020-10-15).

`burntsushi` gives the same split as library and application. "If you're writing a library, defining your own error type should be strongly preferred so that you don't remove choices from the caller unnecessarily" ([Andrew Gallant: Error Handling in Rust](https://burntsushi.net/rust-error-handling/), 2015-05-14, updated 2020-01-03 to recommend `anyhow`). His 2022 post adds: "anyhow is great for application oriented code, but if one is building a library intended for others to use, I'd suggest writing out concrete error types and providing an appropriate `std::fmt::Display` impl" ([Andrew Gallant: Using unwrap() in Rust is Okay](https://burntsushi.net/unwrap/), 2022-08-08). The Book agrees on the library side: "it's best to return an error if you can so that the user of the library can decide what they want to do in that case" ([The Rust Programming Language: To panic! or Not to panic!](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html), the Book shipped with Rust 1.99.1, which targets Rust 1.97 or later).

The two framings agree, because a library's caller is the one most likely to branch. `matklad`'s question also settles the cases the split misses: an application module whose caller retries on one failure needs a concrete type, and a library function whose only failure is "report this" can return a boxed error. The crate READMEs, which are references, frame it the same way: thiserror "most often applies to library-like code", and anyhow suits code that doesn't "care what error type your functions return" ([dtolnay/thiserror: Comparison to anyhow](https://github.com/dtolnay/thiserror#comparison-to-anyhow), [dtolnay/anyhow: Comparison to thiserror](https://github.com/dtolnay/anyhow#comparison-to-thiserror)).

## Library errors

**Make the public type an opaque struct, and expose a `#[non_exhaustive]` fieldless kind enum for callers that branch.** This is `std::io::Error`'s design. `matklad` notes that its internal enum "is a well-hidden implementation detail", while "publicly exposing `ErrorKind` is much less scary. A `#[non_exhaustive]` `Copy` fieldless enum's design space is a point — there are no plausible alternatives or compatibility hazards" (2020-10-15, above).

**Don't make the public type an enum whose variants hold other crates' errors.** `matklad` names two costs: "exposing errors from underlying libraries makes them a part of your public API. Major semver bump in your dependency would require you to make a new major version as well", and "it sets all the implementation details in stone" (2020-10-15, above). `burntsushi`'s own crates show the same lesson. The `regex` crate's public `Error` enum carries the comment "We really should have made the Error type in the regex crate completely opaque. Rookie mistake." ([rust-lang/regex: src/error.rs](https://github.com/rust-lang/regex/blob/master/src/error.rs)). His later `jiff` has one opaque `Error` with predicates such as `is_range()`, chosen "after attempting to provide finer grained error types. But finer grained error types proved difficult in the face of composition" ([BurntSushi/jiff: crates/jiff/src/error/mod.rs](https://github.com/BurntSushi/jiff/blob/master/crates/jiff/src/error/mod.rs), changed 2026-07-19).

**Box the representation, so the `Err` side stays one pointer.** `matklad` warns that "On-the-stack size of errors is important: you pay for it even if there are no errors!" (2020-10-15), and "just a single large error on however rare code path leads to worse code everywhere" ([Alex Kladov: Error ABI](https://matklad.github.io/2025/11/09/error-ABI.html), 2025-11-09). That post covers languages in general, so under his roster notes it corroborates and doesn't carry the rule. `nnethercote` advises: "If an enum has an outsized variant, consider boxing one or more fields", which is "more likely to be a net performance win if the `A::Z` variant is relatively rare" ([The Rust Performance Book: Type Sizes](https://nnethercote.github.io/perf-book/type-sizes.html)). Clippy's `result_large_err` (perf, warn) fires at an `Err` of 128 bytes, set by `large-error-threshold` ([Clippy Lints](https://rust-lang.github.io/rust-clippy/stable/index.html#result_large_err)). In the editor's probe below, `Result<Vec<u64>, LoadError>` is 24 bytes, the same as `Vec<u64>`.

**Derive with `thiserror`.** It writes the `Display`, `Error`, and `From` implementations a hand-written type needs. Its README says it "deliberately does not appear in your public API", so "switching from handwritten impls to thiserror or vice versa is not a breaking change" ([dtolnay/thiserror: Details](https://github.com/dtolnay/thiserror#details)). Version 2 builds without `std` when you set `default-features = false` ([thiserror 2.0.0 release notes](https://github.com/dtolnay/thiserror/releases/tag/2.0.0), 2024-11-06), on the `core::error::Error` that Rust 1.81 stabilised ([Rust Blog: Announcing Rust 1.81.0](https://blog.rust-lang.org/2024/09/05/Rust-1.81.0/)).

`burntsushi` dissents mildly: "The thiserror crate removes some of the boiler plate involved in doing that, but I'd skip it to avoid the procedural macro dependencies if one isn't already using procedural macro dependencies for something else" (2022-08-08, above). That's the only roster source on the question, and it's a build-cost argument, not a design one. An opinion could keep it as the one exception: hand-write the implementations in a crate with no other proc-macro dependency.

This compiles with `-D warnings -W clippy::pedantic` on 1.99.1, with `thiserror` 2.0.21 (editor's probe, 2026-10-19):

```rust
use std::{fs, io, num::ParseIntError, path::Path};

/// The error [`load`] returns.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct LoadError(Box<Repr>);

/// What went wrong, for a caller that branches on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LoadErrorKind {
    /// The file couldn't be read.
    Read,
    /// A line wasn't a number.
    Parse,
}

#[derive(Debug, thiserror::Error)]
enum Repr {
    #[error("failed to read the file")]
    Read { source: io::Error },
    #[error("line {line} is not a number")]
    Parse { line: usize, source: ParseIntError },
}

impl LoadError {
    /// Returns what went wrong.
    #[must_use]
    pub fn kind(&self) -> LoadErrorKind {
        match *self.0 {
            Repr::Read { .. } => LoadErrorKind::Read,
            Repr::Parse { .. } => LoadErrorKind::Parse,
        }
    }
}

impl From<Repr> for LoadError {
    fn from(repr: Repr) -> Self {
        Self(Box::new(repr))
    }
}

/// Reads one number per line.
///
/// # Errors
///
/// Returns [`LoadErrorKind::Read`] if the file can't be read, and
/// [`LoadErrorKind::Parse`] if a line isn't a number.
pub fn load(path: &Path) -> Result<Vec<u64>, LoadError> {
    let text = fs::read_to_string(path).map_err(|source| Repr::Read { source })?;
    text.lines()
        .enumerate()
        .map(|(i, line)| {
            line.trim().parse().map_err(|source| {
                Repr::Parse {
                    line: i + 1,
                    source,
                }
                .into()
            })
        })
        .collect()
}
```

`#[error(transparent)]` forwards both `Display` and `source()` to `Repr`, and a field named `source` becomes the `source()` without an attribute. The caller passed the path, so the error leaves it out, and the caller's context adds it once.

## `Display` and `source()`

**Write the message as a concise lowercase phrase with no trailing punctuation.** The `std` documentation says that "Error messages are typically concise lowercase sentences without trailing punctuation", as in `"invalid digit found in string"` ([std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html)).

**Return the wrapped error from `source()`, and leave it out of the message.** The `std` documentation says that "the underlying error should be either returned by the outer error's `Error::source()`, or rendered by the outer error's `Display` implementation, but not both" (same page). A reporter such as `anyhow` walks the chain, so an error that does both prints its cause twice. With the probe above, `anyhow` prints each layer once, on both failure paths:

```text
Error: loading nope.txt

Caused by:
    0: failed to read the file
    1: No such file or directory (os error 2)
```

```text
Error: loading n.txt

Caused by:
    0: line 2 is not a number
    1: invalid digit found in string
```

No lint checks this, so it's a review rule. Clippy's `map_err_ignore` (restriction) catches the opposite mistake, `map_err(|_| …)`, which drops the cause entirely.

## Application errors

**Return `anyhow::Result<()>` from `main`, and add context where a layer knows what it was doing.** `burntsushi` puts it simply: "Just cargo add anyhow and then use `fn main() -> anyhow::Result<()>`. That's it", and attaching context "is one of the most important parts" of errors as values, which "the anyhow crate makes … effortless" (2022-08-08, above). Use `with_context(|| …)` when the message formats anything, so it's built only on failure, `bail!` for an early `Err`, and `.context()` on an `Option` to turn `None` into an error. The probe's binary:

```rust
use std::path::PathBuf;

use anyhow::{Context, Result, bail};

fn main() -> Result<()> {
    let Some(path) = std::env::args_os().nth(1).map(PathBuf::from) else {
        bail!("usage: app <file>");
    };
    let numbers = probe::load(&path).with_context(|| format!("loading {}", path.display()))?;
    let total = numbers
        .iter()
        .try_fold(0_u64, |sum, &n| sum.checked_add(n))
        .context("the total overflows u64")?;
    println!("{total}");
    Ok(())
}
```

`std` prints an `Err` from `main` to stderr as `Error: {err:?}` and exits with `EXIT_FAILURE` ([std::process::Termination](https://doc.rust-lang.org/std/process/trait.Termination.html), stable since 1.61). `anyhow`'s `Debug` prints the chain, as shown above, and a backtrace when `RUST_BACKTRACE=1` or `RUST_LIB_BACKTRACE=1` is set ([std::backtrace: Environment Variables](https://doc.rust-lang.org/std/backtrace/index.html#environment-variables), stable since 1.65).

`matklad` notes that the ecosystem never settled: "Rust doesn't really have super strong conventions about the errors, but it started with mostly enums, and then failure and anyhow shone spotlight on the universal error type", and threading precise types through a large codebase "ultimately leads to 'a bag of everything' error types with many 'dead' variants" ([Alex Kladov: The Second Great Error Model Convergence](https://matklad.github.io/2025/12/29/second-error-model-convergence.html), 2025-12-29). The post compares languages in general, so under his roster notes it can't carry a Rust detail. It's context for `burntsushi`'s `anyhow` advice above, not a ruling between crates.

## Panics

`opinions/rust.md` already fixes the boundary: return `Result` for every failure a caller can cause, and panic only on a bug. The sources add four rules it doesn't have.

**Prefer `expect` with a "should" message, and keep `unwrap` where a message adds nothing.** The `std` documentation says: "We recommend that `expect` messages are used to describe the reason you _expect_ the `Result` should be `Ok`", with the hint to "focus on the word 'should'", as in "env variable `IMPORTANT_PATH` should be set by `wrapper_script.sh`" ([Result::expect: Recommended Message Style](https://doc.rust-lang.org/std/result/enum.Result.html#recommended-message-style)). The Book does the same: `.expect("Hardcoded IP address should be valid")` (Book, above). `burntsushi` agrees: "Prefer `expect()` to `unwrap()`, since it gives more descriptive messages when a panic does occur. But use `unwrap()` when `expect()` would lead to noise" (2022-08-08).

**Don't turn on `clippy::unwrap_used` codebase-wide.** `burntsushi` argues against linting `unwrap()`, because it invites `expect("")` and messages that repeat the code: "I just happen to think the juice isn't worth the squeeze" (2022-08-08). `unwrap_used` and `expect_used` are restriction lints, allowed by default. Clippy's own description says "for a lot of quick-and-dirty code, unwrap is a good choice, which is why this lint is Allow by default" ([Clippy Lints: unwrap_used](https://rust-lang.github.io/rust-clippy/stable/index.html#unwrap_used)). `corrode` is a little stricter, saying `unwrap` "shouldn't be the first intuition for dealing with unexpected situations", but agrees it's "okay to use `unwrap()` when you can prove that a failure is impossible" ([Matthias Endler: Don't Unwrap Options: There Are Better Ways](https://corrode.dev/blog/rust-option-handling-best-practices/), 2024-08-29, updated 2025-05-20). The sources agree once read as: no ban, and `ok_or_else` or `?` wherever the `None` or `Err` can happen. Where panic freedom matters more than convenience, `corrode` suggests `indexing_slicing` and `arithmetic_side_effects` instead ([Matthias Endler: Hardening Rust Code For Production](https://corrode.dev/blog/hardening-rust/), 2026-07-23).

**Document the contract.** The Book: "Contracts for a function, especially when a violation will cause a panic, should be explained in the API documentation for the function" (above). Clippy's `missing_errors_doc` and `missing_panics_doc` (both pedantic, allowed by default) report a public function without a `# Errors` or `# Panics` section, as the editor's probe on 1.99.1 confirmed on 2026-10-19. `templates/Cargo.toml` doesn't turn on `clippy::pedantic`, so the two need naming in `[workspace.lints.clippy]`.

**Choose the panic strategy in the binary, never in a library.** `sunshowers` writes that "in typical production-grade Rust, expected errors are `Result`s while unexpected errors (and only unexpected errors) are panics", and "reusable Rust libraries _cannot_ require `panic = 'abort'`" ([Rain: In defense of lock poisoning in Rust](https://sunshowers.io/posts/on-poisoning/), 2025-12-02). The same post defends `.lock().unwrap()`: poisoning is how a panic in a critical section is detected. `corrode` says that "Panic behavior is part of your system's failure model", so "Be explicit about whether a failure may take down a single task, a single thread, or the entire process", and "If you maintain a library, you have less control over where your code runs and what a panic can take down" (2026-07-23, above). An opinion could say: a binary sets `panic` in `[profile.release]` on purpose, and a library stays correct under unwinding.

## Lints

Groups and defaults are from [Clippy Lints](https://rust-lang.github.io/rust-clippy/stable/index.html) for Rust 1.99, and the warn-by-default ones were confirmed by the editor's probe.

| Lint                 | Group       | Default | What it catches                                           |
| -------------------- | ----------- | ------- | --------------------------------------------------------- |
| `result_large_err`   | perf        | warn    | an `Err` of 128 bytes or more                             |
| `result_unit_err`    | style       | warn    | a public function that returns `Result<_, ()>`            |
| `manual_ok_err`      | complexity  | warn    | a `match` that rewrites `.ok()` or `.err()` (new in 1.86) |
| `io_other_error`     | style       | warn    | `io::Error::new(ErrorKind::Other, …)` (new in 1.87)       |
| `missing_errors_doc` | pedantic    | allow   | a public `Result` function with no `# Errors`             |
| `missing_panics_doc` | pedantic    | allow   | a public function that can panic, with no `# Panics`      |
| `map_err_ignore`     | restriction | allow   | `map_err(\|_\| …)`, which drops the cause                 |
| `unwrap_used`        | restriction | allow   | `unwrap()` on `Option` or `Result`                        |
| `expect_used`        | restriction | allow   | `expect()` on `Option` or `Result`                        |

Turn on `missing_errors_doc` and `missing_panics_doc`. Leave `unwrap_used` and `expect_used` off, for the reasons in [Panics](#panics). `map_err_ignore` is a candidate, but no roster source argues for it.

## Where the roster disagrees

- **`thiserror` or hand-written implementations in a library.** `burntsushi` would skip `thiserror` where it's the only proc macro. No other roster source addresses it. The weight is with `thiserror`, because its own documentation makes the choice reversible without a breaking change, and `burntsushi` frames his view as build cost. An opinion could keep his case as the single exception.
- **Universal or precise error types.** `matklad` describes the trade-off without picking a side, and `burntsushi`'s `jiff` moved from fine-grained types to one opaque type. Nothing on the roster argues for one error type per function. The opaque-struct-with-kind shape takes both into account: one public type, with as much precision as callers need through `kind()`.

## Coming next

Not stable as of 2026-10-19, so not yet an opinion. Statuses are from the tracking issues, which are references.

| Feature                                | Status, 2026-10-19                                                                                                                                                                                                                                                                                                                                  |
| -------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Never type `!`                         | On beta, and stable in 1.100, due 2026-11-12. Makes `Infallible` an alias of `!` ([rust-lang/rust#155499](https://github.com/rust-lang/rust/pull/155499)). `opinions/rust.md` already carries the note                                                                                                                                              |
| `try` blocks                           | Unstable, with no stabilisation PR ([rust-lang/rust#154391](https://github.com/rust-lang/rust/issues/154391))                                                                                                                                                                                                                                       |
| `Try` trait v2                         | Unstable, labelled for design concerns ([rust-lang/rust#84277](https://github.com/rust-lang/rust/issues/84277))                                                                                                                                                                                                                                     |
| `Error::provide` and `Request`         | Unstable. On 2026-02-26 the libraries API team said it prefers language-level trait-object casting, RFC 3885, and the stabilisation proposal was cancelled ([rust-lang/rust#99301](https://github.com/rust-lang/rust/issues/99301)). The `std` page already says to "avoid delegating implementations of `provide` to source error implementations" |
| `std::error::Report`, `Error::sources` | Unstable, with no stabilisation PR ([rust-lang/rust#90172](https://github.com/rust-lang/rust/issues/90172), [rust-lang/rust#58520](https://github.com/rust-lang/rust/issues/58520)). Until they land, `anyhow` does the reporting                                                                                                                   |

`yoshua-wuyts` proposes `throws` syntax and anonymous error enums that could give "about 80%" of what `anyhow::Error` provides without boxing ([Yoshua Wuyts: Syntactic Musings on the Fallibility Effect](https://blog.yoshuawuyts.com/syntactic-musings-on-the-fallibility-effect/), 2025-12-17). It's a proposal, and his roster notes forbid citing one as current Rust.

## Watch-list and unvetted material

Kept apart from the roster findings above. None of it carried a claim.

- **Watch-list `luca-palmieri`:** frames the choice as "reason about intent", and calls "`anyhow` is for applications, `thiserror` is for libraries" the wrong framing ([Luca Palmieri: Error Handling In Rust - A Deep Dive](https://www.lpalmieri.com/posts/error-handling-rust/), 2021). It agrees with `matklad`'s "how the error will be used" question.
- **Watch-list `ms-rust-guidelines`:** M-ERRORS-CANONICAL-STRUCTS asks for a situation-specific struct with a private kind and `is_xxx()` methods, M-APP-ERROR allows `anyhow` or `eyre` in applications, and M-PANIC-ON-BUG says detected bugs panic ([Microsoft: Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/), version 2026.6). They agree with every recommendation here.
- **Watch-list `sabrina-jewson`:** one `#[non_exhaustive]` error struct per operation, with a `kind` field, lowercase `Display`, and the cause in `source()` ([Sabrina Jewson: Modular Errors in Rust](https://sabrinajewson.org/blog/errors), 2023-04-08). It's finer-grained than the recommendation here, and `jiff`'s experience above argues against that granularity.
- **Watch-list `effective-rust`:** Item 4 recommends `thiserror` for libraries and `anyhow` for applications, and Item 18 says don't panic ([David Drysdale: Effective Rust](https://www.effective-rust.com/errors.html)). It agrees.
- **Unvetted:** Tyler Neely argues that a crate's error type should "only hold errors that should cause the overall system to halt", and returns expected failures as a nested `Result` ([Tyler Neely: Error Handling in a Correctness-Critical Rust Project](https://sled.rs/errors), 2020-04-08). It's the strongest source outside the roster, and it's older than the window.
- **Unvetted:** GreptimeDB moved from `thiserror` and `anyhow` to `snafu` for a many-crate workspace, recording a virtual stack of locations instead of a backtrace ([Ruihang Xia: Error Handling for Large Rust Projects](https://greptime.com/blogs/2024-05-07-error-rust), 2024-05-07). A vendor's engineering blog. Dmitrii Aleksandrov argues for `thiserror` enums even in applications ([Why Use Structured Errors in Rust Applications?](https://home.expurple.me/posts/why-use-structured-errors-in-rust-applications/), 2025-05-28). The only 2025 to 2026 critique of the `anyhow` default found, and its author's track record is unconfirmed.
- **Crates not recommended:** `snafu` 0.9.2, `eyre` 0.6.14, `miette` 7.6.0, and `error-stack` 0.8.0 are maintained. `color-eyre`'s standalone repository is archived, and the crate now lives in `eyre-rs/eyre`. No roster source recommends any of them over `thiserror` and `anyhow`, and the 2025 to 2026 sweep found nothing that changes that.

## What this means for the repository

- **No opinion covers error types.** Run `resolve-research` to create `opinions/error-handling.md`. Its first sentence is the opening list here. It needs these sections: how the error will be used, library errors with the probe example, `Display` and `source()`, application errors, the four panic rules, and the lints table. List it under README.md's Scope and in the layout tree.
- **`opinions/rust.md` keeps its Panics section and changes one line.** It replaces "For error types, see `error-handling.md`, which isn't written yet" with a link, and the new file links back for the `Result`-or-panic boundary instead of restating it.
- **`templates/Cargo.toml` gains two lints.** Set `missing_errors_doc = "warn"` and `missing_panics_doc = "warn"` in `[workspace.lints.clippy]`. Pinning `thiserror = "2.0.21"` and `anyhow = "1.0.104"` in `[workspace.dependencies]` is a choice for `project-structure.md` to make, and `refresh-rust-versions` would then maintain the pins.
- **The 1.100 release** brings the never type on 2026-11-12. `refresh-rust-versions` handles it, as `opinions/rust.md` already notes.
- **One weak `vet-source` candidate:** Tyler Neely (sled.rs), if a sustained recent Rust record turns up. The roster carried every recommendation without him, so it isn't a blocker.
