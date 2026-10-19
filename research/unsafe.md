---
targets: [rust-1.99, edition-2024]
last-reviewed: 2026-10-19
sources:
  [anssi-rust, corrode, matklad, ralf-jung, rust-blog, rust-docs, tweede-golf]
---

# Unsafe: soundness, `unsafe_op_in_unsafe_fn`, and Miri

Write no `unsafe` unless a crate needs it for FFI, hardware access, a new abstraction, or a measured hot path. When it does, put the `unsafe` inside one module whose safe API no safe caller can misuse, and test that module under Miri:

- **Deny `unsafe_code` across the workspace,** and opt one module back in with `#[expect(unsafe_code, reason = "…")]`. A manifest `forbid` can't be relaxed for a module, and a crate that inherits `[lints]` can't override one key.
- **Treat the module as the soundness boundary.** Keep each invariant in a private field, and make the safe API sound for every safe caller, including a misbehaving `Ord`, `Clone`, or `Drop` implementation.
- **Explain every `unsafe`.** Write a `// SAFETY:` comment on each block and `unsafe impl`, and a `# Safety` section on each public `unsafe fn` and `unsafe trait`. Turn on `clippy::undocumented_unsafe_blocks` and the two `unnecessary_safety_*` lints to enforce both.
- **Keep an `unsafe fn` body safe by default.** Edition 2024 makes `unsafe_op_in_unsafe_fn` warn, so each unsafe operation inside an `unsafe fn` needs its own `unsafe {}` block and comment.
- **Prefer the 1.82 to 1.99 raw-pointer APIs:** `&raw const` and `&raw mut` over `&x as *const _`, `.cast()` over pointer `as`, and the strict provenance methods over integer round trips.
- **Run `cargo miri test` in CI** on a nightly toolchain, with the default aliasing model. A pass means the tests hit no undefined behaviour, not that the code is sound.

Everything recommended here is stable in 1.99.1 (released 2026-10-15) except Miri itself, which ships only as a nightly component. That's a tool, not a language feature, and the code it tests stays on stable. [Coming next](#coming-next) lists what's unstable as of 2026-10-19.

**No opinion file covers this.** `opinions/rust.md` mentions `unsafe` only in its edition 2024 migration table, for `static_mut_refs`, unsafe attributes, and `set_var`. This is the `unsafe.md` topic in [PLAN.md: Phase 6](../PLAN.md#phase-6-first-opinions-and-templates).

## What `unsafe` promises

The Reference defines the keyword in two directions. `unsafe fn`, `unsafe trait`, and `unsafe static` create an obligation that callers or implementers must meet. `unsafe {}`, `unsafe impl`, `unsafe extern`, and `#[unsafe(attr)]` assert that an obligation defined elsewhere is met ([The Rust Reference: The `unsafe` keyword](https://doc.rust-lang.org/reference/unsafe-keyword.html)). An unsafe block states "that all relevant proof obligations of functions or operations called inside the block have been discharged".

`unsafe` doesn't make undefined behaviour allowed. Code that "can be misused by safe code to exhibit undefined behavior" is unsound, and the list of undefined behaviour "is not exhaustive": there's "no formal model of Rust's semantics" ([The Rust Reference: Behavior considered undefined](https://doc.rust-lang.org/reference/behavior-considered-undefined.html#r-undefined.soundness)). The page asks readers to read the Rustonomicon before writing `unsafe`. On aliasing it gives only an outline, "The exact aliasing rules are not determined yet", and adopts neither Stacked Borrows nor Tree Borrows.

`ralf-jung` splits a type's invariants in two ([Ralf Jung: Two Kinds of Invariants: Safety and Validity](https://www.ralfj.de/blog/2018/08/22/two-kinds-of-invariants.html), 2018-08-22):

- **The validity invariant** is what the compiler assumes for optimisation, such as a `bool` being 0 or 1. "Unsafe code must always uphold validity invariants."
- **The safety invariant** is what safe code may assume, such as `Vec`'s first `len` elements being initialised. "Unsafe code only has to uphold safety invariants at the boundaries to safe code."

That split is why a `&mut T` to a half-built value can be fine inside a module and unsound once it escapes.

## Encapsulation: the module is the boundary

The Rustonomicon's [Working with Unsafe](https://doc.rust-lang.org/nomicon/working-with-unsafe.html) shows that an unsafe block's soundness can depend on state that safe code in the same module can change: setting `Vec`'s `len` in safe code breaks the unsafe code that reads it. Its conclusion is that `unsafe` "pollutes a whole module", and that privacy at the module boundary contains it. `matklad` reaches the same place from code review: reasoning about one block is local, but "correctness of two `unsafe` blocks in isolation does not guarantee that they together are correct!", and "the boundary for unsafe code is a module" ([Alex Kladov: Unsafe as a Human-Assisted Type System](https://matklad.github.io/2019/07/25/unsafe-as-a-type-system.html), 2019-07-25).

Unsafe code also can't trust safe code it doesn't control. `BTreeMap` must stay sound when `Ord` lies, because a safe trait implementation can't be relied on for memory safety ([The Rustonomicon: How Safe and Unsafe Interact](https://doc.rust-lang.org/nomicon/safe-unsafe-meaning.html)). The same applies to `Clone`, `Drop`, `Deref`, and any closure the module calls.

`anssi-rust` makes encapsulation a rule. LANG-UNSAFE-ENCP requires that a component either expose only safe behaviour, "in which no safe interaction can result in UB", or mark its unsafe features with "exhaustively documented" usage conditions ([ANSSI: General information on `unsafe`](https://anssi-fr.github.io/rust-guide/unsafe/generalities.html), changed 2026-03-25). Its note in `AWESOME-HUMANS.md` asks for a `rust-docs` cross-check before an opinion rests on a rule. The Reference's soundness rule and the Rustonomicon carry the same point.

The editor's example compiles with `clippy::pedantic`, the lints below, and `-D warnings` on 1.99.1, and passes `cargo +nightly miri test` (nightly 2026-10-18):

```rust
//! A non-empty vector whose invariant lives in one module.

pub use non_empty::NonEmpty;

#[expect(unsafe_code, reason = "`first` skips a bounds check that the invariant makes redundant")]
mod non_empty {
    /// A vector with at least one element.
    pub struct NonEmpty<T> {
        // Invariant: `items` is never empty. Only this module can touch the field.
        items: Vec<T>,
    }

    impl<T> NonEmpty<T> {
        /// Creates a vector holding `first`.
        pub fn new(first: T) -> Self {
            Self { items: vec![first] }
        }

        /// Appends an element.
        pub fn push(&mut self, item: T) {
            self.items.push(item);
        }

        /// Returns the first element, without an `Option`.
        #[must_use]
        pub fn first(&self) -> &T {
            // SAFETY: `items` is never empty, so index 0 is in bounds.
            unsafe { self.items.get_unchecked(0) }
        }
    }
}
```

It needs `unsafe_code = "deny"` in `[lints.rust]`. A `pub` field, or a `pub fn` that removes elements, would make `first` unsound without touching the `unsafe` line.

## When to write `unsafe` at all

`anssi-rust` LANG-UNSAFE says unsafe blocks "SHOULD be avoided, or MUST be justified" by FFI, hardware access at fixed addresses, or "a performance wall on a small portion of code". Otherwise `#![forbid(unsafe_code)]` "must appear in the crate root". The Book says to "keep unsafe blocks small" and to wrap them in a safe abstraction ([The Rust Programming Language: Unsafe Rust](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html)).

**`deny` beats `forbid` in a workspace.** The editor's probe on 1.99.1 (2026-10-19) found two things:

- `unsafe_code = "forbid"` in a manifest turns a module's `#[expect(unsafe_code, …)]` into error E0453. With `deny`, the `expect` compiles cleanly, and it warns if the module stops needing `unsafe`.
- A crate with `[lints] workspace = true` can't add a `[lints.rust]` key of its own: Cargo refuses to load the manifest.

So ANSSI's `forbid` would force a crate that needs one unsafe module to drop the workspace lints entirely. Workspace `deny`, plus `expect` on one module, gets the same audit trail. A crate that never needs `unsafe` can still add `#![forbid(unsafe_code)]` to its root.

## Documenting `unsafe`

The Book calls it idiomatic to write a comment "starting with SAFETY" on every unsafe function, saying what the caller must do, and on every unsafe operation, saying how the rules are upheld. Clippy enforces each half (lint groups from `--explain` and a probe on 1.99.1, 2026-10-19):

| Lint                            | Group       | Default | Requires                                                                                    |
| ------------------------------- | ----------- | ------- | ------------------------------------------------------------------------------------------- |
| `missing_safety_doc`            | style       | warn    | A `# Safety` section on every public `unsafe fn` and `unsafe trait`                         |
| `undocumented_unsafe_blocks`    | restriction | allow   | A `// SAFETY:` comment on every `unsafe` block and `unsafe impl`                            |
| `unnecessary_safety_comment`    | restriction | allow   | No `// SAFETY:` comment on safe code, so comments don't outlive the `unsafe` they explained |
| `unnecessary_safety_doc`        | restriction | allow   | No `# Safety` section on a safe public function                                             |
| `multiple_unsafe_ops_per_block` | restriction | allow   | One unsafe operation per block. The probe counted `(*p, *p.add(1))` as three, so it's noisy |
| `borrow_as_ptr`                 | pedantic    | allow   | `&raw const x` and `&raw mut x`, not `&x as *const _`                                       |
| `ptr_as_ptr`                    | pedantic    | allow   | `.cast::<U>()`, not `as *const U`                                                           |

`unsafe_op_in_unsafe_fn` is a rustc lint, not a Clippy one. The Edition Guide says it "now warns by default" on edition 2024, because an `unsafe fn` keyword that also unlocked its whole body was "too risky without explicit unsafe blocks" ([Rust Edition Guide: `unsafe_op_in_unsafe_fn` warning](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html)). Under CI's `-D warnings` it's a hard error, so an edition 2024 crate needs no configuration for it.

**Leave out `multiple_unsafe_ops_per_block`.** The Book's "keep unsafe blocks small" is the goal. The lint turns it into one block per dereference and pointer offset, so a two-field read needs three blocks and three comments. This is the editor's judgement from the probe, and no roster source weighs in.

## Raw pointers, provenance, and uninitialized memory

Each row is a stable replacement for an older idiom, from the release notes on `rust-blog` and the `std` documentation:

| Use                                                           | Release | Instead of                                                       |
| ------------------------------------------------------------- | ------- | ---------------------------------------------------------------- |
| `&raw const x` and `&raw mut x`                               | 1.82    | `addr_of!`, or `&x as *const _`, which creates a reference first |
| `unsafe extern "C" { … }`, with `safe fn` items               | 1.82    | A bare `extern` block, now an error on edition 2024              |
| `#[unsafe(no_mangle)]` and `#[unsafe(export_name)]`           | 1.82    | The unmarked attributes, now an error on edition 2024            |
| `ptr.with_addr`, `map_addr`, and `addr`                       | 1.84    | `ptr as usize` and back, which loses provenance                  |
| `ptr::without_provenance` and `ptr::dangling`                 | 1.84    | `addr as *const T` for a sentinel or a dangling pointer          |
| `[MaybeUninit<T>]::assume_init_ref` and `write_copy_of_slice` | 1.93    | Casting a `&[MaybeUninit<T>]` to `&[T]` by hand                  |

`MaybeUninit` itself replaced `mem::uninitialized` in 1.36, which is older than this topic's window. Sources: [Rust Blog: Announcing Rust 1.82.0](https://blog.rust-lang.org/2024/10/17/Rust-1.82.0/), [Announcing Rust 1.84.0](https://blog.rust-lang.org/2025/01/09/Rust-1.84.0/), and the 1.93 entries in [rust-lang/rust: RELEASES.md](https://github.com/rust-lang/rust/blob/main/RELEASES.md). The `std::ptr` documentation calls strict provenance "a set of APIs designed to make working with provenance more explicit", "intended as substitutes for casting a pointer to an integer and back", and says "most code needs no changes to conform" ([`std::ptr`: Strict Provenance](https://doc.rust-lang.org/std/ptr/index.html#strict-provenance)). Code that must turn an integer into a pointer uses `expose_provenance` and `with_exposed_provenance`, which Miri checks less thoroughly.

The compiler now catches some of these mistakes itself. `dangerous_implicit_autorefs` (warn in 1.88, deny in 1.89) rejects an implicit reference through a raw-pointer dereference, `integer_to_ptr_transmutes` (warn in 1.91) flags `transmute::<usize, *const T>`, and `static_mut_refs` denies a reference to a `static mut` on edition 2024. 1.98 makes `unsafe_code` fire on every unsafe attribute. These come from [RELEASES.md](https://github.com/rust-lang/rust/blob/main/RELEASES.md), and `opinions/rust.md` already covers the edition 2024 rows.

## FFI

`anssi-rust` has the roster's only FFI rules, in [ANSSI: Foreign Function Interface](https://anssi-fr.github.io/rust-guide/unsafe/ffi.html). The ones an opinion can apply mechanically:

| Rule                                 | Says                                                                              |
| ------------------------------------ | --------------------------------------------------------------------------------- |
| FFI-SAFEWRAPPING (recommendation)    | Wrap the foreign library in a safe API                                            |
| FFI-AUTOMATE (recommendation)        | Generate bindings with a tool such as `bindgen` or `cbindgen`                     |
| FFI-CTYPE and FFI-PFTYPE             | Use only `#[repr(C)]` types and the `core::ffi::c_*` aliases                      |
| FFI-CKNONROBUST and FFI-CK-PTR-VALID | Check foreign values and pointers before using them as Rust values                |
| FFI-NOENUM                           | Don't accept a Rust `enum` from foreign code, whose discriminant could be invalid |
| FFI-MEM-OWNER and FFI-MEM-NODROP     | Make ownership explicit, and pass no `Drop` type across the boundary              |
| FFI-NOPANIC (recommendation)         | Don't let a panic cross the boundary                                              |

Since 1.81, a panic that unwinds out of an `extern "C"` function aborts the process instead of being undefined behaviour ([RELEASES.md: 1.81.0](https://github.com/rust-lang/rust/blob/main/RELEASES.md)). FFI-NOPANIC still applies, because an abort is rarely what the caller wants. `tweede-golf`'s bzip2 port shows the direction of travel: c2rust output is "full of unsafe", and changing a pointer argument to a slice removed every `unsafe` from one function ([Tweede golf: Translating bzip2 with c2rust](https://tweedegolf.nl/en/blog/151/translating-bzip2-with-c2rust/), 2025-03-11).

## Testing `unsafe` code

**Miri.** "Miri is an Undefined Behavior testing tool for Rust", and it aims to "find all de-facto Undefined Behavior in deterministic Rust programs" ([Ralf Jung: What's "new" in Miri](https://www.ralfj.de/blog/2025/12/22/miri.html), 2025-12-22, written with the POPL 2026 Miri paper). The Book recommends it and installs it with `rustup +nightly component add miri`. `corrode` recommends it for production hardening, with the same CI job ([Matthias Endler: Hardening Rust Code For Production](https://corrode.dev/blog/hardening-rust/), 2026-07-23). `tweede-golf` found it "extremely helpful to confirm the correctness of our code" in the bzip2 port.

The Miri README is a reference for how to run it ([rust-lang/miri: README](https://github.com/rust-lang/miri)):

```yaml
miri:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Install Miri
      run: |
        rustup toolchain install nightly --component miri
        rustup override set nightly
        cargo miri setup
    - run: cargo miri test
```

- **Keep the default aliasing model, Stacked Borrows.** The README says Tree Borrows "is even more experimental", and that "it is likely that the eventual final aliasing model of Rust will be stricter than Tree Borrows", so code that passes only under `-Zmiri-tree-borrows` "might be declared UB in the future". The editor's probe on nightly 2026-10-18 confirmed Stacked Borrows is still the default.
- **Add `MIRIFLAGS="-Zmiri-many-seeds=0..16"`** to concurrent code, which explores more thread interleavings and allocation addresses. The default range of 64 seeds is slow.
- **Mark tests Miri can't run with `#[cfg_attr(miri, ignore)]`.** Miri has no networking and most FFI, and it interprets slowly.
- **Read a pass for what it is.** "Miri fundamentally cannot ensure that your code is sound." It checks the executions the tests reach. When it finds undefined behaviour, the code is unsound. When it doesn't, you might need more inputs.

The `ci.md` topic owns the CI job itself, such as pinning the nightly date and caching.

**`cargo-careful`** runs tests against a `std` built with debug assertions. It's "much faster than executing your program in Miri", "fully FFI-compatible", and misses much of what Miri catches ([Ralf Jung: cargo careful](https://www.ralfj.de/blog/2022/09/26/cargo-careful.html), 2022-09-26). Use it for unsafe code that calls C and so can't run under Miri. It also needs nightly.

**Sanitizers** (`-Zsanitizer=address`) are still nightly-only: 1.99.1 rejects the flag with "the option `Z` is only accepted on the nightly compiler". See [Coming next](#coming-next).

## Where the roster disagrees

- **`forbid` or `deny`.** `anssi-rust` LANG-UNSAFE says `#![forbid(unsafe_code)]` in each crate root without a justified exception. In a workspace that inherits `[lints]`, `forbid` can't be relaxed for the one module that needs `unsafe` (probe above). The proposal follows ANSSI's intent, `unsafe` only where justified and visible, through `deny` plus `#[expect(unsafe_code, reason = "…")]`. The `reason` string records the justification ANSSI asks for.
- **Leaking memory.** `anssi-rust` MEM-FORGET and MEM-LEAK forbid `mem::forget` and `Box::leak` in secure code ([ANSSI: Memory management](https://anssi-fr.github.io/rust-guide/unsafe/memory.html)). In `std`, both are safe functions, because Rust's safety guarantees don't include running destructors ([`std::mem::forget`](https://doc.rust-lang.org/std/mem/fn.forget.html)). The ANSSI rules are about resource exhaustion in security-critical software, not soundness, so they belong in `security.md`, not `unsafe.md`.

## Coming next

Unstable on nightly as of 2026-10-19, not yet an opinion. Statuses are from the tracking issues and [Rust Project Goals: 2026](https://goals.rust-lang.org/2026/goals.html), which are references.

| Feature                             | Status, 2026-10-19                                                                                                                                                                                                                                                                                                                                                                                    |
| ----------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Unsafe fields (RFC 3458)            | RFC accepted, partly implemented behind `unsafe_fields`, and a 2026 goal to stabilise ([rust-lang/rust#132922](https://github.com/rust-lang/rust/issues/132922), [goal](https://goals.rust-lang.org/2026/unsafe-fields.html)). It would let `NonEmpty`'s invariant live on the field                                                                                                                  |
| A normative aliasing model          | Not decided. Tree Borrows was published at PLDI 2025 ([Ralf Jung: The Tree Borrows paper is finally published](https://www.ralfj.de/blog/2025/07/07/tree-borrows-paper.html), 2025-07-07), but neither the Reference nor Miri's default has adopted it. A 2026 goal works on [normative documentation for sound `unsafe` Rust](https://goals.rust-lang.org/2026/safe-unsafe-for-safety-critical.html) |
| MemorySanitizer and ThreadSanitizer | Nightly-only, with a 2026 goal to stabilise both ([goal](https://goals.rust-lang.org/2026/stabilization-of-sanitizer-support.html), [rust-lang/rust#123615](https://github.com/rust-lang/rust/issues/123615))                                                                                                                                                                                         |
| BorrowSanitizer                     | A 2026 goal for an aliasing checker faster than Miri that also covers C and C++ ([goal](https://goals.rust-lang.org/2026/borrowsanitizer.html))                                                                                                                                                                                                                                                       |
| Unsafe binders                      | Experimental ([rust-lang/rust#130516](https://github.com/rust-lang/rust/issues/130516))                                                                                                                                                                                                                                                                                                               |
| Miri on stable                      | No plan found. Miri remains a nightly component                                                                                                                                                                                                                                                                                                                                                       |

None is due on stable in 1.100 (2026-11-12). Unsafe fields is the one to watch: once stable, the `# Safety` and `// SAFETY:` rules extend to unsafe field access, and Clippy's `missing_safety_doc` already supports them ([rust-lang/rust-clippy#16767](https://github.com/rust-lang/rust-clippy/pull/16767)).

## Watch-list and unvetted material

Kept apart from the roster findings above. None of it carried a claim.

- **Watch-list `ms-rust-guidelines`:** M-UNSAFE allows `unsafe` only for "novel abstractions", performance, or FFI, and requires "plain-text reasoning" and a Miri pass. M-UNSAFE-IMPLIES-UB says `unsafe` "must not be used to mark functions that are dangerous to call for other reasons". M-STATIC-VERIFICATION enables `undocumented_unsafe_blocks`, `unnecessary_safety_comment`, `unnecessary_safety_doc`, and `unsafe_op_in_unsafe_fn` ([Microsoft: Pragmatic Rust Guidelines, Correctness](https://microsoft.github.io/rust-guidelines/guidelines/correctness/index.html) and [Universal](https://microsoft.github.io/rust-guidelines/guidelines/universal/index.html)). It agrees with every recommendation here, and picks the same three Clippy lints.
- **Watch-list `effective-rust`:** Item 16, "Avoid writing unsafe code", says to check `std` and crates first, keep a wrapper layer, write safety comments, and "consider running Miri" ([David Drysdale: Effective Rust](https://www.lurklurk.org/effective-rust/unsafe.html)). It agrees.
- **Not swept:** `jon-gjengset` (_Rust for Rustaceans_, chapter 9), `mara-bos`, and the unvetted candidates the plan listed: the Rust for Linux safety standard, Android's and Chromium's unsafe guidance, the Rust Foundation's study of unsafe in published crates, `cargo-geiger`, and Kani. The worker sweep stopped on a usage limit, and the editor didn't need them: roster sources carry every recommendation.

## What this means for the repository

- **No opinion covers the topic.** Run `resolve-research` to create `opinions/unsafe.md`. Its first sentence is the opening list here. It needs sections for when to write `unsafe`, encapsulation and the module boundary, documentation and the lint table, raw pointers and provenance, FFI, and Miri. List it under README.md's Scope and in the layout tree.
- **`templates/Cargo.toml` gains four lints:** `unsafe_code = "deny"` in a new `[workspace.lints.rust]`, and `undocumented_unsafe_blocks`, `unnecessary_safety_comment`, and `unnecessary_safety_doc` in `[workspace.lints.clippy]`. They cost nothing in a crate without `unsafe`. `borrow_as_ptr` and `ptr_as_ptr` come with `clippy::pedantic`, if a later topic turns the group on.
- **Miri runs on nightly.** That's a tool, and the repository's freshness rule is about features the code uses, but `resolve-research` should confirm the opinion may recommend a nightly-only tool. The Book, `corrode`, and the Miri README all do. The CI job belongs in `ci.md`, which this topic should hand the job to.
- **ANSSI's leak rules go to `security.md`,** with the disagreement above.
- **No `vet-source` candidate.** The roster carried every claim. `ralf-jung` and `rust-docs` cover the semantics, `anssi-rust` the rules, and `corrode`, `matklad`, and `tweede-golf` the practice.
