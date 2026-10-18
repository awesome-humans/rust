---
targets: [rust-1.99, edition-2024]
last-reviewed: 2026-10-19
sources:
  [
    burntsushi,
    corrode,
    matklad,
    niko-matsakis,
    rust-blog,
    rust-docs,
    rust-patterns,
    yoshua-wuyts,
  ]
---

# Rust: edition 2024, recent stable features, and idioms

Write every crate on edition 2024 with Rust 1.99, and let the compiler and Clippy enforce the idioms that releases 1.85 to 1.99 made possible:

- **Migrate with `cargo fix --edition`, then review by hand** the four changes it can't rewrite: tail-expression temporaries, `static mut` references, never-type fallback, and combined doctests.
- **Flatten control flow with let chains and `if let` guards,** use `let … else` for an early exit that isn't a plain `?`, and keep the happy path unindented.
- **Replace the helpers that new `std` items retire:** `cfg-if` with `cfg_select!`, `as_any` methods with trait upcasting, `retain`-and-collect loops with `extract_if`, `split_at_mut` index juggling with `get_disjoint_mut`, and `Captures` tricks with `use<..>`.
- **Return `Result` for failure, and keep `unwrap` for bugs and tests.**
- **Silence a lint with `#[expect(…, reason = "…")]`, never `#[allow]`.**

Every feature recommended here is stable in 1.99.1 (released 2026-10-15). The never type reaches stable in 1.100, and everything else in [Coming next](#coming-next) is unstable on nightly as of 2026-10-19.

**No opinion file covers this.** The repository has no `opinions/` directory yet. This is the first topic in [PLAN.md: Phase 6](../PLAN.md#phase-6-first-opinions-and-templates), whose target is `opinions/rust.md`.

## Edition 2024

Edition 2024 shipped with Rust 1.85.0 on 2025-02-20, and its announcement calls it "the largest edition we have released" ([Rust Blog: Announcing Rust 1.85.0 and Rust 2024](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)). `cargo new` creates edition 2024 crates, and a manifest without an `edition` key still means 2015 ([The Cargo Book: The manifest format](https://doc.rust-lang.org/cargo/reference/manifest.html#the-edition-field)). No later edition is planned: the compiler's `LATEST_STABLE_EDITION` is `Edition2024`, and the 2026 project goals have no edition goal (worker check of `rustc_span/src/edition.rs` and [Rust Project Goals: 2026](https://goals.rust-lang.org/2026/goals.html), 2026-10-19). The Edition Guide expects "roughly a three year window" between editions.

### Migration steps

The Edition Guide gives five steps ([Rust Edition Guide: Transitioning an existing project to a new edition](https://doc.rust-lang.org/edition-guide/editions/transitioning-an-existing-project-to-a-new-edition.html)):

1. Run `cargo update`, and commit it on its own, because proc macros and build-time code generation can break on a new edition.
2. Run `cargo fix --edition`. It runs `cargo check` with the `rust-2024-compatibility` lint group, applies the fixes, and backs a fix out if it breaks the build ([Rust Edition Guide: Advanced migration strategies](https://doc.rust-lang.org/edition-guide/editions/advanced-migrations.html)). Add `--all-features` and each `--target` you ship, or cfg-gated code isn't migrated.
3. Set `edition = "2024"` in `Cargo.toml`.
4. Run `cargo build` and `cargo test`.
5. Run `cargo fmt`, and commit the formatting on its own.

The automatic migration doesn't touch doctests, and it can't see code that macros or build scripts generate.

### What needs a person

`cargo fix` rewrites most changes. These ones need review or manual work (each from its page under [Rust Edition Guide: Rust 2024](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)):

| Change                                | Lint                                                     | What `cargo fix` does                    | What you do                                                                                                                                                                                                                                                                           |
| ------------------------------------- | -------------------------------------------------------- | ---------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tail-expression temporaries           | `tail_expr_drop_order`                                   | Warns only                               | Move a temporary that the tail expression borrows into a `let`                                                                                                                                                                                                                        |
| `if let` temporary scope              | `if_let_rescope`                                         | Rewrites to `match`                      | Revert each `match` that didn't need the old drop order. The guide recommends the review                                                                                                                                                                                              |
| `static mut` references               | `static_mut_refs`, deny                                  | Nothing                                  | Use an atomic, a `Mutex`, or `&raw const` and `&raw mut`. The lint also fires on implicit borrows such as `NUMS.len()`                                                                                                                                                                |
| Never-type fallback                   | `dependency_on_unit_never_type_fallback`                 | Nothing                                  | Name the type: `f::<()>()?` or `() = f()?`                                                                                                                                                                                                                                            |
| RPIT lifetime capture                 | `impl_trait_overcaptures`                                | Adds `use<..>` to keep the old capture   | Delete the added bound where capturing everything is fine. With an argument-position `impl Trait`, name the type parameter by hand                                                                                                                                                    |
| `unsafe extern` and unsafe attributes | `missing_unsafe_on_extern`, `unsafe_attr_outside_unsafe` | Adds `unsafe`                            | Check each extern signature and each `#[unsafe(no_mangle)]`, `export_name`, and `link_section`, which the compiler can't verify                                                                                                                                                       |
| `set_var` and `remove_var`            | `deprecated_safe_2024`                                   | Wraps in `unsafe {}` with a TODO comment | Remove the call, or prove no other thread is running                                                                                                                                                                                                                                  |
| `expr` fragment specifier             | `edition_2024_expr_fragment_specifier`                   | Rewrites `expr` to `expr_2021`           | Revert it in most macros. The guide says "you will likely want to keep the expr specifier"                                                                                                                                                                                            |
| Combined doctests                     | none                                                     | Nothing                                  | Mark a doctest that depends on `Location`, `type_name`, or line numbers `standalone_crate`                                                                                                                                                                                            |
| MSRV-aware resolver                   | none                                                     | Nothing                                  | Edition 2024 implies `resolver = "3"`, which prefers dependency versions that match `rust-version`. A virtual workspace still sets `resolver = "3"` in `[workspace]` ([The Cargo Book: Resolver versions](https://doc.rust-lang.org/cargo/reference/resolver.html#resolver-versions)) |
| rustfmt style edition                 | none                                                     | `cargo fmt` applies it                   | Add `style_edition = "2024"` to `rustfmt.toml`, because an editor that runs bare `rustfmt` defaults to 2015                                                                                                                                                                           |

The guide also expects the never-type fallback to change "across all editions at a later date". It does in Rust 1.100, which removes `dependency_on_unit_never_type_fallback` ([rust-lang/rust#155499](https://github.com/rust-lang/rust/pull/155499), merged 2026-08-24).

## Stable features that change the idiom

Each row replaces an older pattern. The release is the first stable one, from the release announcements on `rust-blog`.

| Feature                                    | Release | Replaces                                                                                                                       |
| ------------------------------------------ | ------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `let … else`                               | 1.65    | `let x = if let Some(x) = o { x } else { return … };`                                                                          |
| `#[expect]` lint attribute                 | 1.81    | `#[allow]`, which stays silent after the lint stops firing                                                                     |
| Precise capturing `use<..>` on RPIT        | 1.82    | The `Captures<'a>` trait and `+ 'a` outlives tricks                                                                            |
| Async closures and `AsyncFn*` traits       | 1.85    | `\|\| async move { … }` and `Fn() -> impl Future` bounds                                                                       |
| Trait upcasting                            | 1.86    | `fn as_any(&self) -> &dyn Any` on every implementer                                                                            |
| `get_disjoint_mut` on slices and `HashMap` | 1.86    | `split_at_mut` and index arithmetic                                                                                            |
| `Vec::pop_if`                              | 1.86    | `last()` checked, then `pop()`                                                                                                 |
| `use<..>` inside traits                    | 1.87    | Nothing: the 1.82 form couldn't be used in a trait                                                                             |
| `Vec::extract_if`                          | 1.87    | `retain` plus a second pass, or a manual drain loop. `HashMap` and `HashSet` follow in 1.88, `BTreeMap` and `BTreeSet` in 1.91 |
| Let chains, edition 2024 only              | 1.88    | Nested `if let` blocks, and `matches!` followed by a second `if let`                                                           |
| `cfg(true)` and `cfg(false)`               | 1.88    | `cfg(all())` and `cfg(any())`                                                                                                  |
| `Cell::update`                             | 1.88    | `c.set(c.get() + 1)`                                                                                                           |
| Inferred const arguments `[x; _]`          | 1.89    | Repeating the length the type already gives                                                                                    |
| `File::lock` and `try_lock`                | 1.89    | A third-party file-locking crate                                                                                               |
| `fmt::from_fn`                             | 1.93    | A one-off wrapper struct with a `Display` impl                                                                                 |
| `cfg_select!`                              | 1.95    | The `cfg-if` crate                                                                                                             |
| `if let` guards in `match` arms            | 1.95    | A nested `match` or `if let` inside the arm body                                                                               |
| `Vec::push_mut` and `insert_mut`           | 1.95    | `push` followed by `last_mut().unwrap()`                                                                                       |
| `assert_matches!`                          | 1.96    | `assert!(matches!(…))`, which prints no value on failure                                                                       |
| `core::range` types                        | 1.96    | Storing `std::ops::Range`, which isn't `Copy`                                                                                  |
| `{integer}::format_into`                   | 1.98    | The `itoa` crate, at "similar" speed by the announcement's account                                                             |

Sources, in order: [Rust Blog: Announcing Rust 1.65.0](https://blog.rust-lang.org/2022/11/03/Rust-1.65.0/), [1.81.0](https://blog.rust-lang.org/2024/09/05/Rust-1.81.0/), [1.82.0](https://blog.rust-lang.org/2024/10/17/Rust-1.82.0/), [1.85.0](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/), [1.86.0](https://blog.rust-lang.org/2025/04/03/Rust-1.86.0/), [1.87.0](https://blog.rust-lang.org/2025/05/15/Rust-1.87.0/), [1.88.0](https://blog.rust-lang.org/2025/06/26/Rust-1.88.0/), [1.89.0](https://blog.rust-lang.org/2025/08/07/Rust-1.89.0/), [1.91.0](https://blog.rust-lang.org/2025/10/30/Rust-1.91.0/), [1.93.0](https://blog.rust-lang.org/2026/01/22/Rust-1.93.0/), [1.95.0](https://blog.rust-lang.org/2026/04/16/Rust-1.95.0/), [1.96.0](https://blog.rust-lang.org/2026/05/28/Rust-1.96.0/), and [1.98.0](https://blog.rust-lang.org/2026/08/20/Rust-1.98.0/). The editor compiled a probe using let chains, an `if let` guard, `let … else`, trait upcasting, `cfg_select!`, `extract_if`, `get_disjoint_mut`, an async closure, `assert_matches!`, and `#[expect]` on 1.99.1 on 2026-10-19.

```rust
// Flat control flow on edition 2024 and Rust 1.95 or later.
fn port(config: &Config) -> Option<u16> {
    if let Some(server) = &config.server
        && let Some(port) = server.port
        && port != 0
    {
        return Some(port);
    }
    match config.fallback.as_deref() {
        Some(s) if let Ok(port) = s.parse() => Some(port),
        _ => None,
    }
}
```

This is a fragment: it needs a `Config` type with `server: Option<Server>` and `fallback: Option<String>`.

### `let … else` beside `?`

`corrode` makes `let … else` the default for unwrapping an `Option`, because "it clearly highlights the 'happy path'" ([Matthias Endler: Don't Unwrap Options: There Are Better Ways](https://corrode.dev/blog/rust-option-handling-best-practices/), 2024-08-29). Clippy narrows that. `clippy::question_mark` (style, warn) rewrites `let Some(x) = o else { return None };` as `let x = o?;` (editor's probe on 1.99.1, 2026-10-19). So use `?` when the `else` branch only propagates, and `let … else` when it logs, returns a different value, `continue`s, or `break`s.

### The Book lags on let chains

The Book targets Rust 1.97.0 and edition 2024, and teaches `let … else` in chapter 6. Chapter 19 still says "We can't combine these two conditions into `if let Ok(age) = age && age > 30`" ([The Rust Programming Language: All the Places Patterns Can Be Used](https://doc.rust-lang.org/book/ch19-01-all-the-places-for-patterns.html), read 2026-10-19). That's wrong on edition 2024. The Reference allows it (`[expr.if.chains]` in [The Rust Reference: `if` expressions](https://doc.rust-lang.org/reference/expressions/if-expr.html#chains-of-conditions)), the 1.88 announcement shows it, and the probe compiles it. `rust-docs`'s notes already say to prefer the Reference on language rules.

## Lints that enforce the idioms

The lints in this table are warn-by-default, so `cargo clippy -- -D warnings` enforces them with no configuration. Groups and versions are from [Clippy Lints](https://rust-lang.github.io/rust-clippy/stable/index.html), 1.99, read 2026-10-19.

| Lint                                   | Group      | Added | Enforces                                                                                   |
| -------------------------------------- | ---------- | ----- | ------------------------------------------------------------------------------------------ |
| `collapsible_if`                       | style      | <1.29 | Collapses an `if let` with an inner `if` into a let chain on edition 2024 (editor's probe) |
| `question_mark`                        | style      | <1.29 | `?` over a `let … else` or `match` that only propagates                                    |
| `unnecessary_map_or`                   | style      | 1.84  | `opt == Some(5)` and `is_none_or` over `map_or(false, …)` and `map_or(true, …)`            |
| `manual_ok_err`                        | complexity | 1.86  | `.ok()` and `.err()` over a hand-written `match`                                           |
| `manual_repeat_n`, `manual_slice_fill` | style      | 1.86  | `iter::repeat_n` and `fill`                                                                |
| `io_other_error`                       | style      | 1.87  | `io::Error::other` over `io::Error::new(ErrorKind::Other, …)`                              |
| `manual_is_multiple_of`                | complexity | 1.90  | `is_multiple_of` over `% n == 0`                                                           |
| `manual_pop_if`                        | complexity | 1.96  | `Vec::pop_if`                                                                              |

Rustc adds two warn-by-default lints that change everyday signatures. `mismatched_lifetime_syntaxes` (1.89) asks for `Iter<'_, u8>` where a return type hides an elided lifetime. `dangling_pointers_from_locals` (1.91) catches a raw pointer to a local escaping its function ([Rust Blog: Announcing Rust 1.89.0](https://blog.rust-lang.org/2025/08/07/Rust-1.89.0/), [Announcing Rust 1.91.0](https://blog.rust-lang.org/2025/10/30/Rust-1.91.0/)).

Two lints need opting in:

- **`manual_let_else`** is pedantic. A crate that warns on `clippy::pedantic`, as this repository's `xtask` does, gets it with the group.
- **`allow_attributes`** is a restriction lint that rewrites `#[allow]` as `#[expect]`. Add it, with `allow_attributes_without_reason`, to `[workspace.lints.clippy]`. `#[expect]` raises `unfulfilled_lint_expectations` once the lint stops firing ([The Rust Reference: Diagnostic attributes](https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-expect-attribute)), so stale suppressions surface.

`uninlined_format_args` is pedantic too, so `println!("{x}")` over `println!("{}", x)` comes with the group.

## Idioms from the roster

**Panics are for bugs.** `burntsushi`: "using `unwrap()` is okay if it's in test/example code or when panicking indicates a bug", and "Panicking should not be used for error handling in either applications or libraries" ([Andrew Gallant: Using unwrap() in Rust is Okay](https://burntsushi.net/unwrap/), 2022-08-08, still his latest on the subject). `corrode` agrees that `unwrap` "shouldn't be the first intuition", and suggests `ok_or_else` when unsure (2024-08-29, above). Its hardening post adds "use `assert!` or return a `Result` instead of relying on `debug_assert!`" for invariants that guard production behaviour ([Matthias Endler: Hardening Rust Code For Production](https://corrode.dev/blog/hardening-rust/), 2026-07-23). `error-handling.md` is the place for the full error design. This topic only fixes the boundary.

**Make invalid states and silent truncation hard to write.** `corrode` lists safe-Rust pitfalls with a fix for each ([Matthias Endler: Pitfalls of Safe Rust](https://corrode.dev/blog/pitfalls-of-safe-rust/), 2025-04-01):

- Arithmetic wraps silently in release builds. Use `checked_*`, or set `overflow-checks = true`.
- `as` truncates silently. Use `From` and `TryFrom`.
- Indexing panics. Use `.get()` and `split_at_checked`.
- A derived `Debug` can print a secret, and a derived `Default` can build an invalid value.

Its defensive-programming post adds slice patterns over indexing, no wildcard `_` arm on an enum you own, `#[non_exhaustive]`, `#[must_use]`, and enums over `bool` parameters. It names the Clippy lints for each, among them `indexing_slicing`, `wildcard_enum_match_arm`, and `fn_params_excessive_bools` ([Matthias Endler: Patterns for Defensive Programming in Rust](https://corrode.dev/blog/defensive-programming/), 2025-11-08). Most are restriction or pedantic lints, so an opinion has to pick which to turn on.

**Prefer `std` over a small crate.** Several rows in the features table retire a dependency: `cfg-if`, `itoa`, and a file-locking crate. `LazyLock` did the same for `once_cell` in 1.80 ([Rust Blog: Announcing Rust 1.80.0](https://blog.rust-lang.org/2024/07/25/Rust-1.80.0/)). `corrode` says the same of `thread::scope` over `thread::spawn` for borrowed data ([Matthias Endler: Sharp Edges In The Rust Standard Library](https://corrode.dev/blog/sharp-edges-in-rust-std/), 2025-05-21).

**Signatures.** Take `impl Trait` in argument position when a generic appears once, so the signature reads left to right, and a named parameter when the type recurs ([Matthias Endler: Practical Rust API Design](https://corrode.dev/blog/practical-rust-api-design/), 2026-10-02). Take `&str` and `&[T]`, not `&String` and `&Vec<T>` ([Rust Design Patterns: Use borrowed types for arguments](https://rust-unofficial.github.io/patterns/idioms/coercion-arguments.html), **Corroborate.**, so it can't carry this alone, and here `corrode` doesn't state it). Move a condition to the caller, so a function takes `Walrus`, not `Option<Walrus>` ([Alex Kladov: Push Ifs Up And Fors Down](https://matklad.github.io/2023/11/15/push-ifs-up-and-fors-down.html), 2023-11-15). `api-design.md` is the place for the rest.

**Iterators.** Chain adaptors, choose the collection at `collect`, and split a long chain into named steps when it stops reading clearly ([Matthias Endler: Thinking in Iterators](https://corrode.dev/blog/iterators/), 2024-05-15).

### Where the roster disagrees

- **Cloning to satisfy the borrow checker.** `rust-patterns` calls it an anti-pattern, with `Rc` and `Arc` as the exception ([Rust Design Patterns: Clone to satisfy the borrow checker](https://rust-unofficial.github.io/patterns/anti_patterns/borrow_clone.html)). `corrode` says "Clone liberally" and treats explicit lifetimes as "a last resort" ([Matthias Endler: Be Simple](https://corrode.dev/blog/simple/), 2025-09-11, and [Don't Worry About Lifetimes](https://corrode.dev/blog/lifetimes/), 2024-05-29). `niko-matsakis`'s 2025 series accepts the clone and proposes making it shorter to write (see Coming next). The weight is with cloning: `rust-patterns` is **Corroborate.**, and the other two are unmarked and newer. An opinion could say: clone an `Arc` or a small value freely, and don't add a lifetime parameter to avoid one clone outside a measured hot path.
- **`..Default::default()`.** `corrode` contradicts itself. The 2025-11-08 defensive-programming post says to avoid it, so the compiler makes you handle each new field. The 2026-09-24 post recommends a `Default` options struct with `..Default::default()` as named arguments ([Matthias Endler: We Have Named Arguments at Home](https://corrode.dev/blog/named-arguments-at-home/)). They reconcile by type: struct-update syntax suits an options struct, whose new fields have safe defaults, and not a domain value whose new field changes its meaning. That reading is the editor's, and an opinion needs a second source before it relies on it.
- **Early return.** `corrode` prefers a tail expression to an explicit `return` ([Matthias Endler: Thinking in Expressions](https://corrode.dev/blog/expressions/), 2025-01-21), yet recommends `let … else`, which always leaves early. The two agree once read as: guard clauses at the top, then one tail expression. Clippy's `needless_return` (style) enforces the second half.

## Coming next

Unstable on nightly as of 2026-10-19, not yet an opinion. Statuses are from the tracking issues and [Rust Project Goals: 2026](https://goals.rust-lang.org/2026/goals.html), which are references. The opinion should cite them only to date a "Coming next" note.

| Feature                                            | Status, 2026-10-19                                                                                                                                                                                                    |
| -------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Never type `!`                                     | Stable in 1.100, in beta now, due 2026-11-12 on the six-week cycle. Makes `Infallible` an alias of `!` and the fallback `!` in every edition ([rust-lang/rust#155499](https://github.com/rust-lang/rust/pull/155499)) |
| `try` blocks                                       | Unstable. Homogeneous `try` (RFC 3721) is implemented, with no stabilisation PR ([rust-lang/rust#154391](https://github.com/rust-lang/rust/issues/154391))                                                            |
| `gen` blocks                                       | Unstable, with two open soundness issues. Edition 2024 reserved the keyword ([rust-lang/rust#117078](https://github.com/rust-lang/rust/issues/117078))                                                                |
| Return type notation, `impl Trait` in type aliases | Unstable. A 2026 goal prepares both, blocked on the next trait solver ([Rust Project Goals: Prepare TAIT + RTN for stabilization](https://goals.rust-lang.org/2026/rtn.html))                                         |
| `async fn` in `dyn` traits                         | Experimental. A goal spans 2026 to 2027 ([rust-lang/rust#133119](https://github.com/rust-lang/rust/issues/133119))                                                                                                    |
| Arbitrary self types                               | Stabilisation PR open and blocked on concerns ([rust-lang/rust#135881](https://github.com/rust-lang/rust/pull/135881))                                                                                                |
| Default field values                               | Unstable, implemented, no stabilisation PR ([rust-lang/rust#132162](https://github.com/rust-lang/rust/issues/132162))                                                                                                 |
| Ergonomic reference counting                       | Unstable. The goal moved from `.use` to a `Share` trait and `move($expr)` captures ([Rust Project Goals: Ergonomic ref-counting](https://goals.rust-lang.org/2026/ergonomic-rc.html))                                 |
| Next trait solver, Polonius alpha                  | On by default on nightly since 2026-08-21 and 2026-08-06, not on stable                                                                                                                                               |

The never type is the only row that's due on stable soon. `refresh-rust-versions` should pick it up in the 1.100 bump. Its effect on everyday code: `dependency_on_unit_never_type_fallback` goes away, and `Result<T, Infallible>` becomes `Result<T, !>`.

**Language direction.** `niko-matsakis` is working through the cost of explicit `clone` around closures and async tasks: a `Handle` trait, then "just call clone (or alias)", with a future edition linting redundant clones away ([Niko Matsakis: The Handle trait](https://smallcultfollowing.com/babysteps/blog/2025/10/07/the-handle-trait/), 2025-10-07, and [Just call clone (or alias)](https://smallcultfollowing.com/babysteps/blog/2025/11/10/just-call-clone/), 2025-11-10). `yoshua-wuyts` proposes capture clauses with an `own` keyword instead ([Yoshua Wuyts: Capture Clauses as Effects](https://blog.yoshuawuyts.com/capture-clauses-as-effects), 2026-07). Both are proposals, and their roster notes forbid citing a proposal as current Rust. They can support a "Coming next" note on closure captures, nothing more.

## Watch-list and unvetted material

Kept apart from the roster findings above. None of it carried a claim.

- **Watch-list `ms-rust-guidelines`:** M-LINT-OVERRIDE-EXPECT says to suppress with `#[expect]`, not `#[allow]` ([Microsoft: Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/)). It agrees with the `#[expect]` recommendation, which rests on `rust-docs` and Clippy.
- **Watch-list `effective-rust`:** Item 3 prefers `Option` and `Result` combinators to `match`, and Item 18 says not to panic in a library ([David Drysdale: Effective Rust](https://www.lurklurk.org/effective-rust/)). It predates let chains and doesn't teach `let … else`. It agrees with `burntsushi` on panics.
- **Unvetted:** the workers found no strong source outside the roster on modern idioms. Searches returned tutorials and generated pages, which were discarded.

## What this means for the repository

- **No opinion covers the topic.** Run `resolve-research` to create `opinions/rust.md`. Its first sentence is the opening list here. It needs these sections: edition 2024 and its migration, the features table, the `let … else` and `?` split, the lint configuration, panics, and the cloning position. List it under README.md's Scope and in the layout tree.
- **The templates need three settings this topic fixes.** `templates/Cargo.toml` sets `edition = "2024"`, `resolver = "3"`, and `[workspace.lints.clippy]` with `allow_attributes` and `allow_attributes_without_reason`. `templates/rustfmt.toml` sets `style_edition = "2024"`. They're phase 6 work and can land with the opinion.
- **Leave error design, API shape, and CI to their own topics.** `error-handling.md`, `api-design.md`, and `ci.md` are later in the plan. Rust 1.97's `build.warnings = "deny"`, which replaces `RUSTFLAGS=-Dwarnings` without invalidating the build cache ([Rust Blog: Announcing Rust 1.97.0](https://blog.rust-lang.org/2026/07/09/Rust-1.97.0/)), belongs in `ci.md`.
- **The 1.100 release** brings the never type to stable on 2026-11-12. That's `refresh-rust-versions`.
- **No `vet-source` candidate.** The roster carried every claim. `rust-docs` already covers the Edition Guide, the Reference, and Clippy.
