---
targets: [rust-1.99, edition-2024]
last-reviewed: 2026-10-19
sources: [burntsushi, corrode, matklad, predrag, rust-blog, rust-docs]
---

# API design: SemVer, compatibility checks, and public API shape

Design a library's public API so that you can add to it in a minor release, and check every release for accidental breakage before you publish:

- **Treat the Cargo Book's SemVer chapter as the rule book.** Removing or renaming a public item, adding a public field to a struct with no private ones, adding an enum variant, adding a non-defaulted trait item, and removing a Cargo feature are all major changes.
- **Mark a public struct or enum `#[non_exhaustive]` when you introduce it,** unless the set is closed. Adding the attribute later is itself a major change.
- **Seal a public trait that downstream code doesn't need to implement,** so you can add items to it later.
- **Prefer concrete types, newtypes, and enums to `bool` and to generics** in public signatures, and add `#[must_use]` where ignoring the return value is a bug.
- **Document every public item, with `# Errors`, `# Panics`, and `# Safety` sections,** and turn on the lints that enforce it.
- **Treat a `rust-version` bump as a minor change,** under a written MSRV policy.
- **Run a SemVer checker before `cargo publish`.** `cargo-semver-checks` is the only one, and the Rust standard library now runs it, but its case rests on its own author's writing.

Everything recommended here is stable in Rust 1.99.1 (released 2026-10-15). The two language features that would make sealing and dependency hygiene first-class, `impl` restrictions and public dependencies, are unstable on nightly as of 2026-10-19 ([Coming next](#coming-next)).

**No opinion file covers this.** `opinions/rust.md` covers argument-position `impl Trait` and pushing conditions to the caller, and says "For public API shape, see `api-design.md`, which isn't written yet." This topic is the `api-design.md` row in [PLAN.md: Phase 6](../PLAN.md#phase-6-first-opinions-and-templates).

**The plan's expected sources don't both hold.** `api-guidelines` is on the watch list as dormant, so it can't carry a claim here. `predrag` is citable, but its roster note says never to cite it alone on adopting `cargo-semver-checks`. `rust-docs` carries the SemVer rules instead, and `corrode`, `burntsushi`, and `matklad` carry the API shape.

## What counts as a breaking change

The Cargo Book classes each change as major, minor, or possibly breaking ([The Cargo Book: SemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)). These are the rows that catch people out, with the chapter's anchor for each:

| Change                                                             | Class             | Anchor                                    |
| ------------------------------------------------------------------ | ----------------- | ----------------------------------------- |
| Add a public field when no private field exists                    | Major             | `struct-add-public-field-when-no-private` |
| Add a private field when all current fields are public             | Major             | `struct-add-private-field-when-public`    |
| Add or remove a private field when one already exists              | Minor             | `struct-private-fields-with-private`      |
| Add an enum variant without `#[non_exhaustive]`                    | Major             | `enum-variant-new`                        |
| Add `#[non_exhaustive]` to an existing type with no private fields | Major             | `attr-adding-non-exhaustive`              |
| Add a non-defaulted trait item                                     | Major             | `trait-new-item-no-default`               |
| Add a defaulted trait item                                         | Possibly breaking | `trait-new-default-item`                  |
| Add any inherent item                                              | Possibly breaking | `impl-item-new`                           |
| Capture more generic parameters in a return-position `impl Trait`  | Major             | `generic-rpit-capture`                    |
| Change the minimum Rust version                                    | Possibly breaking | `env-new-rust`                            |
| Remove a Cargo feature                                             | Major             | `cargo-feature-remove`                    |
| Remove an optional dependency                                      | Possibly breaking | `cargo-remove-opt-dep`                    |

For a `0.y.z` release, a change in `y` counts as major and a change in `z` as minor. Every `0.0.z` release is major.

Two of the chapter's mitigations are rules to apply when a type or trait is first published: "Mark structs, enums, and enum variants as `#[non_exhaustive]` when first introducing them, rather than adding `#[non_exhaustive]` later on", and "When introducing the trait, use the sealed trait technique to prevent users outside of the crate from implementing the trait."

**Auto traits leak through private fields.** The chapter doesn't name `Send` or `Sync`. `predrag` shows the case: changing a private field from `String` to `Rc<str>` makes the public struct neither `Send` nor `Sync`, which is a major change ([Predrag Gruevski: Toward fearless cargo update](https://predr.ag/blog/toward-fearless-cargo-update/), 2022-08-25). The Rustonomicon's rule that a type made only of `Send` or `Sync` types is itself `Send` or `Sync` is the mechanism ([The Rustonomicon: Send and Sync](https://doc.rust-lang.org/nomicon/send-and-sync.html)). Removing an auto-trait impl was the fourth most common violation in `predrag`'s study, below.

**Adding a public item is technically breaking, and still minor.** A new item can collide with a downstream glob import. RFC 1105 says "all major changes are breaking, but not all breaking changes are major", and a minor release may contain breakage the user can fix locally ([Predrag Gruevski: Some Rust breaking changes don't require a major version](https://predr.ag/blog/some-rust-breaking-changes-do-not-require-major-version/), 2023-01-26, quoting [RFC 1105: API evolution](https://rust-lang.github.io/rfcs/1105-api-evolution.html)).

### MSRV

**Write an MSRV policy, set `package.rust-version`, and treat a bump as minor.** The Cargo Book's `rust-version` page says changing it "is assumed to be a minor incompatibility". It recommends that you "Choose a policy for what Rust versions to support and when that is changed", and says "The simplest policy to support is to always use the latest Rust version". Its example policies include N-2, "latest version with a 2 release grace window for updating" ([The Cargo Book: Rust version](https://doc.rust-lang.org/cargo/reference/rust-version.html)). The SemVer chapter agrees: "It is generally recommended to treat this as a minor change".

The page expects each supported version to be "verified on its supported Rust versions, including automated testing", which is `ci.md` work.

### Cargo features

**Make features additive, and hide optional dependencies behind `dep:`.** The Cargo Book says "enabling a feature should not disable functionality, and it should usually be safe to enable any combination of features". For `no_std`, use a `std` feature that enables `std`, not a `no_std` feature. Don't remove a feature, move public code behind a feature, or remove a feature from another feature's list in a minor release ([The Cargo Book: Features](https://doc.rust-lang.org/cargo/reference/features.html)).

The `dep:` prefix removes the implicit feature an optional dependency would otherwise create. That's the way to keep a dependency an internal detail. It's also a trap when you add it to an existing crate: `predrag` shows that `max_performance = ["dep:rayon"]` "is quietly removing the `rayon` feature", which is a major change ([Predrag Gruevski: Breakage! in the Cargo.toml](https://predr.ag/blog/breakage-in-the-cargo-toml-how-rust-package-features-work/), 2024-12-04).

## Checking SemVer before publishing

**Run `cargo semver-checks` before `cargo publish`, in CI.** It compares the rustdoc JSON of the new version against the last published one, and fails on any violation it has a lint for. The latest release is 0.51.0, from 2026-10-03, with `rust-version` 1.93 (crates.io, checked 2026-10-19).

The evidence that people need a tool, and not more care:

- **Violations are common.** Of 14,389 minor and patch releases of the 1,000 most-downloaded crates, 464 (3.22%) broke SemVer, and 172 crates (17.2%) did so at least once. That's a lower bound, from default features and the lints of 2023 ([Predrag Gruevski: Semver violations are common, better tooling is the answer](https://predr.ag/blog/semver-violations-are-common-better-tooling-is-the-answer/), 2023-09-07).
- **The Rust standard library broke this way.** Rust 1.94.1 removed new methods from `std::os::windows::fs::OpenOptionsExt`, because "the trait itself is not sealed and so cannot be extended with non-default methods" ([Rust Blog: Announcing Rust 1.94.1](https://blog.rust-lang.org/2026/03/26/1.94.1-release/), 2026-03-26).
- **The standard library now runs `cargo-semver-checks`** in its CI, with unstable items treated as non-public, "exactly as if they were `#[doc(hidden)]`" ([Predrag Gruevski: Protecting the Rust standard library from accidental breakage](https://predr.ag/blog/protecting-the-rust-stdlib-from-breakage/), 2026-08-15).

**Its limits:** It has 242 lints as of the end of 2025, but it doesn't type-check, so changing `x: i64` to `x: String` goes unreported. It also can't yet analyse items re-exported from another crate reliably, which is its largest source of false positives ([Predrag Gruevski: Exponential growth continued: cargo-semver-checks 2025 Year in Review](https://predr.ag/blog/cargo-semver-checks-2025-year-in-review/), 2026-01-11). A clean run is necessary, not sufficient, so the rules above still apply in review.

**The case for adopting it rests on `predrag` alone, and that's the blocker.** Predrag Gruevski wrote the tool, and the roster forbids citing him alone on adopting it. The Cargo Book's SemVer chapter names no tool. The 1.94.1 post shows the failure, but doesn't name a remedy. The Rust Project Goals page calls the tool "broadly adopted by the Rust community" and plans to merge it into `cargo publish` ([Rust Project Goals: Continue resolving cargo-semver-checks blockers for merging into cargo](https://goals.rust-lang.org/2026/cargo-semver-checks.html), accepted for 2026). That's a reference: it can corroborate, but it can't carry the claim. The opinion can state the rules and the need from citable sources, and can name the tool only once a second citable source, or an official Rust document, recommends it.

## Future-proofing the shape

### `#[non_exhaustive]` and private fields

**Mark an open set `#[non_exhaustive]`, and leave a closed set exhaustive.** `corrode`: "mark your enum as `#[non_exhaustive]` so you can add variants later", but "If the set of possibilities really is closed, exhaustive matching gives users a useful guarantee". For an open-ended set of codes, such as HTTP status, it recommends an opaque struct with associated constants. Its test before publishing: "am I willing to uphold this guarantee forever?" ([Matthias Endler: Practical Rust API Design](https://corrode.dev/blog/practical-rust-api-design/), 2026-10-02).

`#[non_exhaustive]` works only across crate boundaries, and blocks struct literals outside the crate, so callers go through your constructor ([Matthias Endler: Patterns for Defensive Programming in Rust](https://corrode.dev/blog/defensive-programming/), 2025-11-08). A struct that already has a private field gets the same protection without the attribute (Cargo Book, above).

Clippy's `exhaustive_structs` and `exhaustive_enums`, both restriction lints, report a public type without the attribute. Both fired on 1.99.1 in a probe crate.

### Sealed traits

**Seal a public trait unless downstream code must implement it.** Adding a required item to an unsealed trait is major, and adding a defaulted one is possibly breaking. `predrag` describes three techniques ([Predrag Gruevski: A definitive guide to sealed traits in Rust](https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/), 2023-04-05):

- **Supertrait-sealed:** the trait requires a supertrait whose name isn't exported. Downstream code can call the methods, but can't implement the trait.
- **Method-sealed:** a required method takes an argument of a type downstream code can't name.
- **Partially sealed:** only some methods take the unnameable type, and they have default bodies, so downstream code can call them but not override them.

The usual form is the supertrait seal:

```rust
mod private {
    pub trait Sealed {}
}

/// A shape this crate can measure.
pub trait Shape: private::Sealed {
    /// The area, in square metres.
    fn area(&self) -> f64;
}

/// A square.
#[derive(Debug, Clone, Copy)]
pub struct Square(pub f64);

impl private::Sealed for Square {}

impl Shape for Square {
    fn area(&self) -> f64 {
        self.0 * self.0
    }
}
```

Three things unseal a trait without warning ([Predrag Gruevski: Is this trait sealed, or not sealed](https://predr.ag/blog/is-this-trait-sealed-or-not-sealed/), 2024-09-03): a `pub use` that makes the private type nameable, a blanket impl such as `impl<T: Clone> Sealed for T`, and a public function that returns a value of the unnameable type.

### Options structs and builders

**For a function with many optional parameters, take an options struct with `Default`.** `corrode` recommends it over a builder: "Instead of optional arguments, we deal with data". A builder is for "when construction requires validation or conversion", and "builders should not be the default. They can become their own tiny programming language" ([Matthias Endler: We Have Named Arguments at Home](https://corrode.dev/blog/named-arguments-at-home/), 2026-09-24). `burntsushi`'s Jiff uses builders, such as `ZonedDifference` and `ZonedRound`, only for the parameters of `since`, `until`, and `round`, with `From` impls to keep the simple call short ([Andrew Gallant: Jiff design](https://github.com/BurntSushi/jiff/blob/master/DESIGN.md)).

**A public options struct collides with `#[non_exhaustive]`.** `corrode`'s call form, `Options { retry: true, ..Default::default() }`, fails in another crate once the struct is `#[non_exhaustive]`: "Non-exhaustive structs and enum variants cannot be constructed using struct literal syntax, including functional update syntax" (Cargo Book, above). It fails with `E0639` on 1.99.1. Without the attribute, every new field is a major change. Two forms survive in a library:

- **Mutate a default:** `let mut options = Options::default(); options.retry = true;`. This compiles across crates with `#[non_exhaustive]` and public fields, verified on 1.99.1.
- **Builder lite:** `Options::default().with_retry(true)`, with `with_` methods on the struct itself. `matklad` calls it "an incremental, zero-cost evolution from the `new` method" ([Alex Kladov: Builder Lite](https://matklad.github.io/2022/05/29/builder-lite.html), 2022-05-29). His examples are application code, so applying it to a library is the editor's reading.

Struct-update syntax stays fine inside your own crate, and in application code, where nobody downstream breaks.

## Signatures and types

`corrode` gives the most rules, from one 2026 post and one 2025 post (both above):

- **Make the signature tell the story.** Decide what an empty string and `None` mean, and whether each is an error, when you design the API.
- **Keep `&` on an argument the function doesn't consume,** so the call site shows the value doesn't move.
- **Return `impl Trait` to hide a complex concrete type.** "`impl Iterator<Item = &str>` tells the caller what they can do with the result".
- **Use a newtype with a validating constructor** for a value with an invariant, and give it an explicit accessor such as `as_str()`. Implement `Deref` only "when the wrapper transparently behaves like its target and dereferencing is cheap and unsurprising".
- **Take an enum instead of a `bool` parameter.** Clippy's `fn_params_excessive_bools` (pedantic) reports four or more.
- **Add `#[must_use]` to guard types and to results the caller has to check.** Clippy's `must_use_candidate` and `return_self_not_must_use` (both pedantic) suggest where.
- **Prefer `BorrowedFd` and `OwnedFd` to `RawFd`,** and wrap an unsafe API in a type that carries the ownership.

**Prefer a concrete type to a generic parameter.** `burntsushi` made Jiff's `Zoned` concrete instead of generic over a time-zone trait, because "being able to just write `Zoned` as a concrete type without any generics is a huge win for comprehensibility". He made `TimeZone` closed for the same reason (Jiff design, above). `rust.md` already covers argument-position `impl Trait`, so the opinion links there instead of repeating it.

**Re-export to shape the public path.** The Book says to use `pub use` to "make a public structure that's different from your private structure", and to re-export a dependency's definitions that your API exposes ([The Rust Programming Language: Publishing a Crate to Crates.io](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html)).

## Documentation

**Document every public item, and give it the sections the Book names.** The Book lists `# Panics`, `# Errors` for a function returning `Result`, and `# Safety` for an `unsafe` function (Book, above). The rustdoc book recommends a one-sentence summary, a longer explanation, and "at least one code example that users can copy/paste to try it", and says "If an item is public then it should be documented" ([The rustdoc book: How to write documentation](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)).

Document the crate's features at the top of `lib.rs`, and leave out any optional dependency that users shouldn't enable directly (Cargo Book: Features, above).

## Lints that enforce it

Each lint fired on 1.99.1 in a probe crate, and the group is the one the compiler reported:

| Lint                                                     | Level or group | Reports                                            |
| -------------------------------------------------------- | -------------- | -------------------------------------------------- |
| `missing_docs`                                           | rustc, allow   | A public item, or the crate, without documentation |
| `missing_debug_implementations`                          | rustc, allow   | A public type without `Debug`                      |
| `unreachable_pub`                                        | rustc, allow   | A `pub` item nobody outside the crate can reach    |
| `clippy::missing_safety_doc`                             | style, warn    | An `unsafe fn` without `# Safety`                  |
| `clippy::missing_errors_doc`                             | pedantic       | A function returning `Result` without `# Errors`   |
| `clippy::missing_panics_doc`                             | pedantic       | A function that can panic without `# Panics`       |
| `clippy::must_use_candidate`                             | pedantic       | A pure function or method without `#[must_use]`    |
| `clippy::return_self_not_must_use`                       | pedantic       | A method returning `Self` without `#[must_use]`    |
| `clippy::exhaustive_structs`, `clippy::exhaustive_enums` | restriction    | A public type without `#[non_exhaustive]`          |

The rustc lint descriptions are from [The rustc book: Allowed-by-default lints](https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html). `missing_panics_doc` reports an `unwrap` or `panic!`, but not slice indexing.

**Keep `avoid-breaking-exported-api` at its default, `true`.** It suppresses 19 Clippy lints, among them `wrong_self_convention`, `unnecessary_wraps`, and `trivially_copy_pass_by_ref`, "whenever the suggested change would cause breakage for other crates" ([Clippy: Lint configuration](https://doc.rust-lang.org/clippy/lint_configuration.html)).

The exhaustive lints are a choice for the opinion. They suit a library that commits to `#[non_exhaustive]` by default, and fire on every closed set too, which then needs an `#[expect]` with a reason.

### Where the roster disagrees

- **Options struct or builder.** `corrode` prefers the options struct, `burntsushi` uses builders for parameter sets, and `matklad` offers builder lite. They agree on more than they differ: `corrode` keeps the builder for validation or conversion, which is Jiff's case. In a library, `#[non_exhaustive]` settles it against struct-update syntax, which leaves mutate-a-default or builder lite.
- **`..Default::default()`.** `rust.md` already records that `corrode` warns against it in the 2025 defensive-programming post and recommends it in the 2026 named-arguments post. For a public type, the Cargo Book now decides: a `#[non_exhaustive]` struct can't use it from another crate, and one that isn't `#[non_exhaustive]` can't add a field in a minor release.

## Coming next

Unstable on nightly as of 2026-10-19, not yet an opinion. The tracking issues are references, cited only to date this note.

| Feature                                      | Status, 2026-10-19                                                                                                                                                                                                                                                                                |
| -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `impl` restrictions, `pub impl(crate) trait` | RFC 3323 is accepted, and the tracking issue is open ([rust-lang/rust#105077](https://github.com/rust-lang/rust/issues/105077)). It would replace the sealed-trait techniques. `predrag` notes that "Trait sealing isn't (yet!) a built-in concept" (2025-03-08)                                  |
| Public and private dependencies              | Open in both rustc and Cargo ([rust-lang/rust#44663](https://github.com/rust-lang/rust/issues/44663), [rust-lang/cargo#6129](https://github.com/rust-lang/cargo/issues/6129)). The `exported_private_dependencies` lint, which reports a private dependency's type in a public API, depends on it |
| `#[doc(cfg)]`                                | Open ([rust-lang/rust#43781](https://github.com/rust-lang/rust/issues/43781)). The Cargo Book calls it "experimental"                                                                                                                                                                             |
| `cargo-semver-checks` in `cargo publish`     | An accepted 2026 Rust project goal, with no date. It waits on type-checking lints and cross-crate analysis (Project Goals, above)                                                                                                                                                                 |

## Watch-list and unvetted material

Kept apart from the roster findings above. None of it carried a claim.

- **Watch-list `api-guidelines`:** the checklist has the items this topic covers, among them C-SEALED, C-STRUCT-PRIVATE, C-BUILDER, C-NEWTYPE, C-COMMON-TRAITS, and C-GOOD-ERR ([Rust API Guidelines: Checklist](https://rust-lang.github.io/api-guidelines/checklist.html)). It agrees with the sealed-trait and private-field rules, which rest on the Cargo Book. Its C-COMMON-TRAITS item, to implement `Clone`, `PartialEq`, `Hash`, `Default`, and the rest eagerly on public types, has no citable source here, so it can't go in the opinion yet.
- **Watch-list `ms-rust-guidelines`, `effective-rust`, and `jon-gjengset`:** not swept. The worker assigned to them hit a usage limit before it reported, so this topic has no cross-check from them.
- **Unvetted:** none. `cargo-semver-checks`' own documentation and the Rust Project Goals pages are references, not unvetted sources.

## What this means for the repository

- **No opinion covers the topic.** Run `resolve-research` to create `opinions/api-design.md`. Its first sentence is the opening list here. It needs these sections: the breaking-change table, MSRV, features, `#[non_exhaustive]` and private fields, sealed traits, options structs, signatures, documentation, and the lint table. List it under README.md's Scope and in the layout tree, and change `rust.md`'s two "isn't written yet" pointers to links.
- **`cargo-semver-checks` needs a second citable source before the opinion recommends it.** Until then, the opinion can state the rules and the 1.94.1 evidence, and name the tool only in a sentence that says why it isn't the recommendation. A `vet-source` pass doesn't help here, because the gap is a citable source that names the tool, not a new author. `harvest-sources` should watch for one, and for a Cargo Book or Rust Blog mention once the project goal lands.
- **Lints for `templates/Cargo.toml`, if the owner wants them.** `missing_docs` and `unreachable_pub` in `[workspace.lints.rust]`, and `missing_errors_doc` and `missing_panics_doc` come with `clippy::pedantic`. A library template turning on `exhaustive_structs` and `exhaustive_enums` is a decision for the opinion. Each applies to libraries only, so a template that holds a binary crate needs them per crate, which is `project-structure.md` territory.
- **`ci.md` owns running the checks.** The `cargo semver-checks` step, and verifying `rust-version` with automated testing, belong there.
- **No `vet-source` candidate.** The roster carried every claim except adopting the tool.
