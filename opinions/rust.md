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
    yoshua-wuyts,
  ]
---

# Rust

Write every crate on edition 2024 with Rust 1.99, and use the features that releases 1.65 to 1.99 added where they replace an older idiom. Let the compiler and Clippy enforce the idioms, so a review doesn't have to.

## Opinions

### Edition 2024

**Set `edition = "2024"` in `[workspace.package]`, and inherit it in every crate with `edition.workspace = true`.** A manifest without an `edition` key still means 2015 ([The Cargo Book: The manifest format](https://doc.rust-lang.org/cargo/reference/manifest.html#the-edition-field)). Edition 2024 shipped with Rust 1.85.0 on 2025-02-20 ([Rust Blog: Announcing Rust 1.85.0 and Rust 2024](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)).

- **Set `resolver = "3"` in a virtual workspace's `[workspace]`.** Edition 2024 implies it in a package, but a virtual manifest has no edition to imply it from. Resolver 3 prefers dependency versions whose `rust-version` your toolchain meets ([The Cargo Book: Resolver versions](https://doc.rust-lang.org/cargo/reference/resolver.html#resolver-versions)).
- **Set `edition = "2024"` and `style_edition = "2024"` in `rustfmt.toml`.** `cargo fmt` reads the edition from the manifest, but an editor that runs bare `rustfmt` defaults to 2015 for both. Without `edition`, it can't parse a let chain.

To migrate a crate, follow the Edition Guide's steps ([Rust Edition Guide: Transitioning an existing project to a new edition](https://doc.rust-lang.org/edition-guide/editions/transitioning-an-existing-project-to-a-new-edition.html)):

1. Run `cargo update`, and commit it on its own.
2. Run `cargo fix --edition --all-features`, once for each `--target` you ship. Otherwise cfg-gated code isn't migrated.
3. Set `edition = "2024"`.
4. Run `cargo build` and `cargo test`.
5. Run `cargo fmt`, and commit the formatting on its own.

`cargo fix` doesn't touch doctests or code that macros and build scripts generate. Review these changes by hand, because it either can't make them or makes them too broadly ([Rust Edition Guide: Rust 2024](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)):

| Change                      | Lint                                     | What you do                                                                                           |
| --------------------------- | ---------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| Tail-expression temporaries | `tail_expr_drop_order`                   | Move a temporary that the tail expression borrows into a `let`                                        |
| `if let` temporary scope    | `if_let_rescope`                         | Revert each `match` that `cargo fix` wrote, unless the code needs the old drop order                  |
| `static mut` references     | `static_mut_refs`                        | Use an atomic, a `Mutex`, or `&raw const` and `&raw mut`                                              |
| Never-type fallback         | `dependency_on_unit_never_type_fallback` | Name the type: `f::<()>()?` or `() = f()?`                                                            |
| RPIT lifetime capture       | `impl_trait_overcaptures`                | Delete each `use<..>` that `cargo fix` added where capturing every lifetime is fine                   |
| Unsafe attributes           | `unsafe_attr_outside_unsafe`             | Check each `#[unsafe(no_mangle)]`, `export_name`, and `link_section`, which the compiler can't verify |
| `set_var` and `remove_var`  | `deprecated_safe_2024`                   | Remove the call, or prove no other thread is running, and delete the TODO comment                     |
| `expr` fragment specifier   | `edition_2024_expr_fragment_specifier`   | Revert `expr_2021` to `expr` in most macros                                                           |
| Combined doctests           | none                                     | Mark a doctest that depends on `Location`, `type_name`, or line numbers as `standalone_crate`         |

### Lints

**Deny warnings in CI with `CARGO_BUILD_WARNINGS=deny cargo clippy`, and add three lints to `[workspace.lints.clippy]`:**

```toml
[workspace.lints.clippy]
allow_attributes = "warn"
allow_attributes_without_reason = "warn"
manual_let_else = "warn"
```

- **Deny warnings through Cargo, not with `cargo clippy -- -D warnings`.** Cargo's `build.warnings` setting, stable since 1.97, fails the build on a rustc or Clippy lint warning in your own crates, and doesn't invalidate the build cache as `-D warnings` does ([Rust Blog: Announcing Rust 1.97.0](https://blog.rust-lang.org/2026/07/09/Rust-1.97.0/#cargo-support-for-denying-warnings), [The Cargo Book: Configuration](https://doc.rust-lang.org/cargo/reference/config.html#buildwarnings), [Clippy Documentation: Usage](https://doc.rust-lang.org/clippy/usage.html#elevating-warning-to-errors)).
- **Silence a lint on an item with `#[expect(…, reason = "…")]`, never `#[allow]`.** `#[expect]` raises `unfulfilled_lint_expectations` once the lint stops firing, so a stale suppression surfaces ([The Rust Reference: Diagnostic attributes](https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-expect-attribute), [rust-lang/rfcs: RFC 2383, `lint_reasons`](https://github.com/rust-lang/rfcs/blob/master/text/2383-lint-reasons.md#expect-lint-attribute)). `allow_attributes` reports an outer `#[allow]` and suggests `#[expect]` in its place, and `allow_attributes_without_reason` reports an attribute with no `reason`. Both are restriction lints, so they're off until you set them ([Clippy Lints: `allow_attributes`](https://rust-lang.github.io/rust-clippy/stable/index.html#allow_attributes), [`allow_attributes_without_reason`](https://rust-lang.github.io/rust-clippy/stable/index.html#allow_attributes_without_reason)).
- **`manual_let_else` is pedantic,** so it's off by default, and the table above turns it on without the rest of the group. It reports a `match` or `if let` that `let … else` would replace ([Clippy Lints: `manual_let_else`](https://rust-lang.github.io/rust-clippy/stable/index.html#manual_let_else)).

```rust
#[expect(clippy::needless_pass_by_value, reason = "the callback API takes ownership")]
pub fn log_owned(message: String) {
    eprintln!("{message}");
}
```

The warn-by-default lints already enforce the rest of this file. Among them are `collapsible_if`, `question_mark`, `needless_return`, `manual_ok_err`, `unnecessary_map_or`, `io_other_error`, `manual_is_multiple_of`, and `manual_pop_if` ([Clippy Lints](https://rust-lang.github.io/rust-clippy/stable/index.html)). Two rustc lints change everyday code: `mismatched_lifetime_syntaxes` (1.89) asks for `Iter<'_, u8>` where a return type hides an elided lifetime, and `dangling_pointers_from_locals` (1.91) catches a raw pointer to a local that escapes its function ([Rust Blog: Announcing Rust 1.89.0](https://blog.rust-lang.org/2025/08/07/Rust-1.89.0/), [Announcing Rust 1.91.0](https://blog.rust-lang.org/2025/10/30/Rust-1.91.0/)).

### Flat control flow

**Use let chains and `if let` guards instead of nested `if let` blocks.** Both are stable on edition 2024: let chains since 1.88 and `if let` guards since 1.95 ([Rust Blog: Announcing Rust 1.88.0](https://blog.rust-lang.org/2025/06/26/Rust-1.88.0/), [Announcing Rust 1.95.0](https://blog.rust-lang.org/2026/04/16/Rust-1.95.0/), [rust-lang/rust#132833](https://github.com/rust-lang/rust/pull/132833), [rust-lang/rust#141295](https://github.com/rust-lang/rust/pull/141295)). Clippy's `collapsible_if` rewrites the nested form into a chain on edition 2024.

Before:

```rust
fn port(config: &Config) -> Option<u16> {
    if let Some(server) = &config.server {
        if let Some(port) = server.port {
            if port != 0 {
                return Some(port);
            }
        }
    }
    match config.fallback.as_deref() {
        Some(s) => match s.parse() {
            Ok(port) => Some(port),
            Err(_) => None,
        },
        None => None,
    }
}
```

After:

```rust
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

Both are fragments: they need a `Config` with `server: Option<Server>` and `fallback: Option<String>`, and a `Server` with `port: Option<u16>`. The Book's chapter 19 still says the two conditions can't be combined ([The Rust Programming Language: All the Places Patterns Can Be Used](https://doc.rust-lang.org/book/ch19-01-all-the-places-for-patterns.html), read 2026-10-19). That's wrong on edition 2024, and the Reference allows it ([The Rust Reference: `if` expressions](https://doc.rust-lang.org/reference/expressions/if-expr.html#chains-of-conditions)).

**Use `?` when the failure branch only propagates, and `let … else` when it does anything else.** `let … else` keeps the happy path unindented ([Matthias Endler: Don't Unwrap Options: There Are Better Ways](https://corrode.dev/blog/rust-option-handling-best-practices/)). Clippy's `question_mark` rewrites `let Some(x) = o else { return None };` as `let x = o?;`, so keep `let … else` for an `else` that logs, returns a different value, or uses `continue` or `break`:

```rust
fn total(lines: &[&str]) -> u32 {
    let mut sum = 0u32;
    for line in lines {
        let Ok(n) = line.trim().parse::<u32>() else {
            eprintln!("skipping {line:?}");
            continue;
        };
        sum = sum.saturating_add(n);
    }
    sum
}
```

**Put guard clauses at the top, and end with one tail expression.** `corrode` prefers a tail expression to an explicit `return` ([Matthias Endler: Thinking in Expressions](https://corrode.dev/blog/expressions/)), and Clippy's `needless_return` reports a `return` in tail position.

### Features that replace an older idiom

**Use the `std` item in the left column wherever the code does what the right column describes.** Each row is from the release announcement of the version shown, on the [Rust Blog](https://blog.rust-lang.org/). A row that retires a crate also removes a dependency.

| Use                                        | Release | Instead of                                                             |
| ------------------------------------------ | ------- | ---------------------------------------------------------------------- |
| `LazyLock` and `LazyCell`                  | 1.80    | The `once_cell` crate                                                  |
| Precise capturing `use<..>`                | 1.82    | The `Captures<'a>` trait and `+ 'a` outlives tricks                    |
| Async closures and `AsyncFn*` bounds       | 1.85    | `\|\| async move { … }` and `Fn() -> impl Future` bounds               |
| Trait upcasting to a supertrait object     | 1.86    | An `as_any(&self) -> &dyn Any` method on every implementer             |
| `get_disjoint_mut` on slices and `HashMap` | 1.86    | `split_at_mut` and index arithmetic                                    |
| `Vec::pop_if`                              | 1.86    | Checking `last()`, then calling `pop()`                                |
| `extract_if` on `Vec`                      | 1.87    | `retain` plus a second pass. Maps and sets have it since 1.88 and 1.91 |
| `Cell::update`                             | 1.88    | `c.set(c.get() + 1)`                                                   |
| `cfg(true)` and `cfg(false)`               | 1.88    | `cfg(all())` and `cfg(any())`                                          |
| `File::lock` and `try_lock`                | 1.89    | A third-party file-locking crate                                       |
| `fmt::from_fn`                             | 1.93    | A one-off wrapper struct with a `Display` impl                         |
| `cfg_select!`                              | 1.95    | The `cfg-if` crate                                                     |
| `Vec::push_mut` and `insert_mut`           | 1.95    | `push` followed by `last_mut().unwrap()`                               |
| `assert_matches!`                          | 1.96    | `assert!(matches!(…))`, which prints no value on failure               |
| `core::range` types                        | 1.96    | Storing `std::ops::Range`, which isn't `Copy`                          |
| `{integer}::format_into`                   | 1.98    | The `itoa` crate                                                       |

Use `thread::scope` for threads that borrow local data, instead of `thread::spawn` with `Arc` clones ([Matthias Endler: Sharp Edges In The Rust Standard Library](https://corrode.dev/blog/sharp-edges-in-rust-std/)).

### Panics

**Return `Result` for every failure a caller can cause, and panic only on a bug.** `unwrap()` and `expect()` are fine in tests, in examples, and where a panic means an invariant broke ([Andrew Gallant: Using unwrap() in Rust is Okay](https://burntsushi.net/unwrap/)). Outside those, use `ok_or_else` to turn an `Option` into an error ([Matthias Endler: Don't Unwrap Options: There Are Better Ways](https://corrode.dev/blog/rust-option-handling-best-practices/)).

- **Use `assert!`, not `debug_assert!`, for an invariant that guards production behaviour.** `debug_assert!` compiles out of release builds ([Matthias Endler: Hardening Rust Code For Production](https://corrode.dev/blog/hardening-rust/)).
- **Avoid the safe operations that fail silently or panic** ([Matthias Endler: Pitfalls of Safe Rust](https://corrode.dev/blog/pitfalls-of-safe-rust/)):
  - Use `checked_*` or `saturating_*` on arithmetic over input values. Plain arithmetic wraps in release builds.
  - Use `From` or `TryFrom` instead of `as` between integer types. `as` truncates.
  - Use `.get()` instead of indexing with an index from input.
  - Write `Debug` by hand for a type that holds a secret, so it can't print the secret.

For error types, see `error-handling.md`, which isn't written yet.

### Ownership and signatures

**Clone an `Arc` or a small value instead of adding a lifetime parameter to avoid it,** outside a hot path you've measured. `corrode` treats explicit lifetimes as "a last resort" ([Matthias Endler: Be Simple](https://corrode.dev/blog/simple/), [Don't Worry About Lifetimes](https://corrode.dev/blog/lifetimes/)). Treating the clone as an anti-pattern lost: its source, `rust-patterns`, is marked Corroborate on the roster.

- **Take `impl Trait` in argument position when a generic appears once,** and a named type parameter when it recurs ([Matthias Endler: Practical Rust API Design](https://corrode.dev/blog/practical-rust-api-design/)).
- **Move a condition to the caller.** A function takes `Walrus`, not `Option<Walrus>`, and the caller decides what `None` means ([Alex Kladov: Push Ifs Up And Fors Down](https://matklad.github.io/2023/11/15/push-ifs-up-and-fors-down.html)).
- **Chain iterator adaptors, and choose the collection at `collect`.** Split a long chain into named `let` steps when it no longer reads clearly ([Matthias Endler: Thinking in Iterators](https://corrode.dev/blog/iterators/)).

For public API shape, see `api-design.md`, which isn't written yet.

## Coming next

**Review by 2026-11-26.** The never type `!` reaches stable in Rust 1.100, due 2026-11-12 ([rust-lang/rust#155499](https://github.com/rust-lang/rust/pull/155499)). It makes `Infallible` an alias of `!`, makes `!` the fallback in every edition, and removes `dependency_on_unit_never_type_fallback`. When `refresh-rust-versions` moves the targets to 1.100, drop this note and the lint's row from the migration table.

These are unstable on nightly as of 2026-10-19, so don't use them yet:

- `try` blocks ([rust-lang/rust#154391](https://github.com/rust-lang/rust/issues/154391)) and `gen` blocks ([rust-lang/rust#117078](https://github.com/rust-lang/rust/issues/117078)).
- Default field values ([rust-lang/rust#132162](https://github.com/rust-lang/rust/issues/132162)).
- `async fn` in `dyn` traits ([rust-lang/rust#133119](https://github.com/rust-lang/rust/issues/133119)).
- Cheaper captures of reference-counted values in closures. `niko-matsakis` proposes a `Handle` trait ([Niko Matsakis: The Handle trait](https://smallcultfollowing.com/babysteps/blog/2025/10/07/the-handle-trait/)), and `yoshua-wuyts` proposes capture clauses ([Yoshua Wuyts: Capture Clauses as Effects](https://blog.yoshuawuyts.com/capture-clauses-as-effects)). Until one lands, clone before the closure.

## Sources

- **`rust-blog`:** the release announcements for 1.80 to 1.98, and edition 2024.
- **`rust-docs`:** the Edition Guide, the Cargo Book, the Reference, the Book, and Clippy's usage guide and lint list.
- **`corrode`:** `let … else`, panics, safe-Rust pitfalls, cloning, signatures, and iterators.
- **`burntsushi`:** when a panic is the right response.
- **`matklad`:** moving conditions to the caller.
- **`niko-matsakis` and `yoshua-wuyts`:** the closure-capture proposals in Coming next only.
