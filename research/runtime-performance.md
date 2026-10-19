---
targets: [rust-1.99, edition-2024]
last-reviewed: 2026-10-19
sources:
  [corrode, matklad, nnethercote, rust-blog, rust-docs, tokio-blog, tweede-golf]
---

# Runtime performance: profiling, allocation, and release profile settings

Measure first, then tune what the measurements point at, and keep the build settings that help every program in the workspace root:

- **Set `codegen-units = 1` and `lto = "fat"` in `[profile.release]`** of a workspace that ships a binary, and leave `panic` at `"unwind"`.
- **Add a `[profile.profiling]` that inherits `release` with `debug = "line-tables-only"`,** profile with samply, and find allocation hot spots with DHAT.
- **Cut allocations before anything else in a hot path:** pre-size collections, reuse buffers, buffer and lock I/O, and let Clippy's warn-by-default `perf` group catch the known patterns.
- **Swap in a faster hasher or a global allocator only behind a benchmark,** and only where the input is trusted or the binary is yours.
- **Treat `#[inline]`, `-C target-cpu=native`, and PGO as last steps,** each with a measured reason.

Everything here is stable in Rust 1.99.1 (released 2026-10-15). Nightly-only items are in [Coming next](#coming-next), unstable as of 2026-10-19.

**No opinion file covers this.** `opinions/` holds only `rust.md`. This topic is the `runtime-performance.md` row of [PLAN.md: Phase 6](../PLAN.md#phase-6-first-opinions-and-templates), whose expected source, `perf-book`, is admitted as `nnethercote`. Compile times and linkers are out of scope: they belong with `project-structure.md` and `ci.md`.

## Measure first

`nnethercote` frames the whole subject around measurement: "Benchmark all changes, one at a time, to ensure they have the expected effects", and "it is only worth optimizing hot code" ([The Rust Performance Book: General Tips](https://nnethercote.github.io/perf-book/general-tips.html)). The biggest wins usually come from changing an algorithm or data structure, not from build flags. The book also warns that memory layout alone can cause "significant but ephemeral" swings, so a wall-time difference needs repeat runs ([Benchmarking](https://nnethercote.github.io/perf-book/benchmarking.html)).

`matklad` adds two working rules for benchmarks. Tune the input so a run takes about 300 milliseconds: under about 10 milliseconds, fixed costs distort it ([Alex Kladov: Benchmark In Milliseconds](https://matklad.github.io/2026/10/05/benchmark-milliseconds.html), 2026-10-05). And make benchmark inputs runtime-overridable and print or hash the result, so the compiler can't delete or constant-fold the work, rather than relying on `std::hint::black_box` alone ([Do Not Optimize Away](https://matklad.github.io/2025/12/09/do-not-optimize-away.html), 2025-12-09). `black_box` itself is stable since 1.66, and its documentation calls it a hint, not a guarantee ([`std::hint::black_box`](https://doc.rust-lang.org/std/hint/fn.black_box.html)).

**Benchmark harnesses.** `#[bench]` is nightly-only. `nnethercote` lists Criterion, Divan, Gungraun (renamed from Iai-Callgrind on 2026-04-23) for instruction counts, Bencher for continuous benchmarking, and hyperfine for whole programs, without ranking them ([Benchmarking](https://nnethercote.github.io/perf-book/benchmarking.html)). No roster source picks between Criterion and Divan. Divan's own README shows `#[divan::bench]` attributes and `cargo test --benches` running each benchmark once as a test ([divan](https://github.com/nvzqz/divan), crate documentation). [What this means for the repository](#what-this-means-for-the-repository) leaves the choice open.

## Release profile

Cargo's built-in `release` profile is `opt-level = 3`, `lto = false`, `codegen-units = 16`, `panic = "unwind"`, `debug = false`, and `incremental = false` ([The Cargo Book: Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html#release)). `lto = false` isn't "no LTO": it's thin local LTO across the crate's own codegen units, and `lto = "off"` is the one that turns it off. Cargo reads profiles only from the workspace root manifest, and ignores them in dependencies, so these settings are for the workspace that builds a binary.

```toml
[profile.release]
codegen-units = 1
lto = "fat"

[profile.profiling]
inherits = "release"
debug = "line-tables-only"
```

A probe on 1.99.1 confirms what Cargo passes: `release` gets `-C codegen-units=1 -C lto=fat -C opt-level=3 -C strip=debuginfo`, and `profiling` gets the same with `-C debuginfo=line-tables-only` and no `strip`.

| Setting                     | Effect                                                                                     | Cost                                              | Source                                                                                                                                                          |
| --------------------------- | ------------------------------------------------------------------------------------------ | ------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `codegen-units = 1`         | Better code than thin local LTO; smaller binary                                            | Slower builds: 4.5 s to 9.7 s on `image` 0.24.1   | `nnethercote`: [Back-end parallelism in the Rust compiler](https://nnethercote.github.io/2023/07/11/back-end-parallelism-in-the-rust-compiler.html), 2023-07-11 |
| `lto = "fat"`               | "can improve runtime speed by 10-20% or more", and shrinks the binary; `"thin"` is cheaper | Compile time; occasionally slower than `"thin"`   | `nnethercote`: [Build Configuration](https://nnethercote.github.io/perf-book/build-configuration.html)                                                          |
| `panic = "abort"`           | "might … increase runtime speed slightly"                                                  | No unwinding: `catch_unwind` stops working        | `nnethercote`, same chapter; tests and benchmarks ignore the key ([Cargo Book](https://doc.rust-lang.org/cargo/reference/profiles.html#panic))                  |
| `-C target-cpu=native`      | Uses every instruction the build machine has                                               | The binary may not run on another CPU             | `nnethercote`, same chapter                                                                                                                                     |
| Profile-guided optimization | "can improve runtime speed by 10% or more"                                                 | A training run per build; not for `cargo install` | `nnethercote`, same chapter; steps in [The rustc book: PGO](https://doc.rust-lang.org/rustc/profile-guided-optimization.html)                                   |

`corrode` gives the same pair, `lto = true` (the same as `"fat"`) and `codegen-units = 1`, and shows setting them only in CI through `CARGO_PROFILE_RELEASE_LTO` and `CARGO_PROFILE_RELEASE_CODEGEN_UNITS` ([Matthias Endler: Tips for Faster Rust CI Builds](https://corrode.dev/blog/tips-for-faster-ci-builds/), 2025-01-28, updated 2025-08-16). That's the CI side of the same trade, and belongs in `ci.md`.

**Where the roster disagrees: `panic = "abort"`.** The book's maximum-speed summary includes it beside `codegen-units = 1`, `lto = "fat"`, and an alternative allocator. Its own text rates the gain "slightly", and the Cargo Book records that the test harness needs unwinding. Aborting also changes behaviour, not just speed: a panic in one thread or task takes the process down instead of being caught. So the topic weighs it as a behaviour decision with a small speed upside, and recommends keeping `"unwind"` unless a binary is already designed to die on any panic.

**`strip` needs nothing.** Since 1.77, Cargo sets `strip = "debuginfo"` for any profile that doesn't enable debug info ([Rust Blog: Announcing Rust 1.77.0](https://blog.rust-lang.org/2024/03/21/Rust-1.77.0/)), and the book shortened its section on 2025-09-22 because of it. A profile with `debug` on, such as `profiling`, isn't stripped.

**Target CPU.** For a binary that runs on more machines than it's built on, detect features at run time instead of setting `target-cpu`. `tweede-golf` does this in zlib-rs: a generic `#[inline(always)]` body, a wrapper with `#[target_feature(enable = "avx2")]`, and dispatch on `is_x86_feature_detected!`, which `std` caches in an atomic but which still shouldn't sit in a hot loop ([Tweede golf: SIMD in zlib-rs, part 1](https://tweedegolf.nl/en/blog/153/simd-in-zlib-rs-part-1-autovectorization-and-target-features/), 2025-04-15). The same post shows a plain loop over `chunks_exact_mut` compiling to SIMD instructions without intrinsics.

**PGO.** The rustc book gives the four steps: build with `-Cprofile-generate`, run representative workloads, merge with `llvm-profdata` from the `llvm-tools-preview` component, and rebuild with `-Cprofile-use` and identical flags. It names `cargo-pgo` as a simpler wrapper ([The rustc book: Profile-guided optimization](https://doc.rust-lang.org/rustc/profile-guided-optimization.html)). `nnethercote` reports that the training set decides the result: adding rustdoc benchmarks to rustc's PGO training cut rustdoc's mean wall time by 2.85% ([How to speed up the Rust compiler in July 2026](https://nnethercote.github.io/2026/07/31/how-to-speed-up-the-rust-compiler-in-july-2026.html), 2026-07-31).

## Profiling

Profile the `profiling` profile, never `dev`: the book reports 10 to 100 times speed-ups from release builds ([Build Configuration](https://nnethercote.github.io/perf-book/build-configuration.html)).

| Tool                     | Use it for                                                 | Platforms             |
| ------------------------ | ---------------------------------------------------------- | --------------------- |
| samply                   | Sampling CPU profiles, viewed in the Firefox Profiler      | Linux, macOS, Windows |
| perf                     | Sampling with hardware counters                            | Linux                 |
| Cachegrind and Callgrind | Instruction counts per function and line; low-variance A/B | Linux, some Unixes    |
| DHAT (Valgrind)          | Allocation hot spots, peak memory, hot `memcpy`            | Linux, some Unixes    |
| dhat-rs                  | DHAT-style heap profiles and heap-usage regression tests   | All                   |
| heaptrack, bytehound     | Heap profiles                                              | Linux                 |

All rows are from `nnethercote` ([Profiling](https://nnethercote.github.io/perf-book/profiling.html)), which also names Instruments, VTune, AMD uProf, `cargo flamegraph`, and Coz, and says to use more than one profiler. The book uses Cachegrind and DHAT most ([Nicholas Nethercote: Rust and Valgrind](https://nnethercote.github.io/2022/01/05/rust-and-valgrind.html), 2022-01-05). Its DHAT rule of thumb, from rustc: cutting 10 allocations per million instructions can give about a 1% speed-up ([Heap Allocations](https://nnethercote.github.io/perf-book/heap-allocations.html)).

Three facts update the book's profiling chapter for 1.99:

- **Frame pointers:** Build with `RUSTFLAGS="-C force-frame-pointers=yes"` for frame-pointer unwinding ([Profiling](https://nnethercote.github.io/perf-book/profiling.html)). The prebuilt `std` already has them, since 1.79 ([rust-lang/rust#122646](https://github.com/rust-lang/rust/pull/122646), a reference).
- **Symbol names:** v0 mangling is the default on stable since 1.97, and keeps generic arguments in the symbol instead of a hash ([Rust Blog: Announcing Rust 1.97.0](https://blog.rust-lang.org/2026/07/09/Rust-1.97.0/)). The book's tip to opt in with `-C symbol-mangling-version=v0` is now redundant, and a profiler needs v0 support to show readable names, which `rustfilt` has.
- **Debug info for `std`:** The prebuilt `std` still ships without it, so `std` frames have no file and line ([Profiling](https://nnethercote.github.io/perf-book/profiling.html)).

## Allocation and copying

Each rule here is from the `nnethercote` chapter named, except where marked. An allocation takes a global lock, bookkeeping, and sometimes a system call, and a small one isn't necessarily cheaper than a large one ([Heap Allocations](https://nnethercote.github.io/perf-book/heap-allocations.html)).

- **Pre-size:** `Vec::with_capacity` or `reserve` when the length is known. Pushing 20 elements into an empty `Vec` allocates four times, at capacities 4, 8, 16, and 32.
- **Reuse:** Keep a working `Vec` or `String` outside the loop and `clear()` it. Use `clone_from` to reuse the destination's allocation.
- **Read lines into one buffer:** `BufRead::lines()` allocates a `String` per line. Use `read_line` and clear the buffer ([I/O](https://nnethercote.github.io/perf-book/io.html)).
- **Buffer and lock I/O:** `println!` locks stdout on every call, and files are unbuffered. Wrap a locked handle in `BufWriter`, and call `flush()` yourself, because a drop ignores errors.
- **Box immutable sequences:** A `Box<[T]>` is two words instead of three. Collect into it directly to skip the reallocation `into_boxed_slice` can make ([Type Sizes](https://nnethercote.github.io/perf-book/type-sizes.html)).
- **Guard type sizes:** Types over 128 bytes are copied with `memcpy`. Box an outsized enum variant, use smaller integer indices, and fail the build when a hot type grows.

Before, one `String` per line and one stdout lock per line:

```rust
use std::io;

fn main() -> io::Result<()> {
    for line in io::stdin().lines() {
        println!("{}", line?.len());
    }
    Ok(())
}
```

After, one buffer, one lock, and buffered output:

```rust
use std::io::{self, BufRead, BufWriter, Write};

fn main() -> io::Result<()> {
    let mut input = io::stdin().lock();
    let mut out = BufWriter::new(io::stdout().lock());
    let mut line = String::new();
    while input.read_line(&mut line)? != 0 {
        writeln!(out, "{}", line.trim_end().len())?;
        line.clear();
    }
    out.flush()
}
```

The book guards sizes with the `static_assertions` crate. On 1.99, `size_of` is in the prelude and a `const` assertion needs no crate. This is a fragment: on its own, the unread fields trip `dead_code`.

```rust
#[derive(Clone, Copy)]
struct Token {
    start: u32,
    len: u32,
    kind: u8,
}

// Fails the build if a change grows the type.
const _: () = assert!(size_of::<Token>() <= 12);
```

All three pass `cargo clippy -- -D warnings -W clippy::pedantic` on 1.99.1, the fragment inside a `main` that reads its fields.

**Clippy enforces part of this.** The `perf` group is warn-by-default and has 38 lints on the stable list, including `large_enum_variant`, `box_collection`, `redundant_allocation`, `slow_vector_initialization`, `unnecessary_to_owned`, `unbuffered_bytes`, and `regex_creation_in_loops` ([Clippy lints](https://clippy.rust-lang.org/stable/index.html), checked 2026-10-19). The book doesn't repeat what Clippy catches by default ([Linting](https://nnethercote.github.io/perf-book/linting.html)). Related lints outside `perf`: `large_types_passed_by_value`, `trivially_copy_pass_by_ref`, `needless_pass_by_value`, `inefficient_to_string`, `large_futures`, and `large_stack_arrays` are pedantic, and `redundant_clone` and `needless_collect` are nursery.

**Global allocator.** Alternative allocators have given "large improvements in runtime speed and large reductions in memory usage", varying by platform, at some cost in binary size and compile time ([Build Configuration](https://nnethercote.github.io/perf-book/build-configuration.html)). The book shows `mimalloc`, which works on many platforms, and `tikv-jemallocator`, for Linux and macOS. A program has one `#[global_allocator]`, so only a binary sets it ([`std::alloc`](https://doc.rust-lang.org/std/alloc/index.html#the-global_allocator-attribute)).

**Hashing.** `HashMap` uses SipHash 1-3 for HashDoS resistance, and the `std` documentation says other algorithms beat it for small keys such as integers, without that protection ([`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html)). For a hot map whose keys an attacker can't choose, `nnethercote` recommends `rustc-hash`'s `FxHashMap`. In rustc, moving back from Fx to the default hasher cost 4 to 84% ([Hashing](https://nnethercote.github.io/perf-book/hashing.html)), and Fx has weak high bits, so a key that packs its entropy there collides ([A brutally effective hash function in Rust](https://nnethercote.github.io/2021/12/08/a-brutally-effective-hash-function-in-rust.html), 2021-12-08). The book doesn't mention `foldhash`, `hashbrown`'s default hasher, which calls itself "minimally DoS-resistant" ([foldhash](https://docs.rs/foldhash/latest/foldhash/), crate documentation). To standardise on one hasher, list `std::collections::HashMap` and `HashSet` in `clippy.toml`'s `disallowed-types` ([Linting](https://nnethercote.github.io/perf-book/linting.html)).

## Inlining and hints

**Leave `#[inline]` off by default.** Since 1.75, rustc makes small functions available for inlining across crates on its own ([rust-lang/rust#116505](https://github.com/rust-lang/rust/pull/116505), a reference). `nnethercote` says the best candidates are small functions or ones with a single call site, which the compiler already handles, and that the attributes' effect is "unpredictable", so re-measure after adding one ([Inlining](https://nnethercote.github.io/perf-book/inlining.html)).

**Where the roster disagrees.** `matklad` told library authors to "proactively add `#[inline]` to small non-generic functions", and to prefer `lto = true` for applications ([Alex Kladov: Inline In Rust](https://matklad.github.io/2021/07/09/inline-in-rust.html), 2021-07-09). The post predates 1.75, whose automatic cross-crate inlining covers the trivial leaf functions it had in mind. Its other points still hold: generic functions are already available for inlining, `#[inline]` isn't transitive, and an inlined body is compiled again in each codegen unit. The topic weighs the 2021 advice as superseded for leaf functions, and keeps it for a non-generic public function that wraps a call the compiler won't treat as small.

**Mark the cold path, not the hot one.** Move rarely run code into a `#[cold]` function ([Inlining: Outlining](https://nnethercote.github.io/perf-book/inlining.html)), or call `std::hint::cold_path()` in a branch, stable since 1.95 ([Rust Blog: Announcing Rust 1.95.0](https://blog.rust-lang.org/2026/04/16/Rust-1.95.0/)).

**Recent stable items that replace a crate or a trick:**

| Use                                              | Release | Instead of                                               | Source                                                                       |
| ------------------------------------------------ | ------- | -------------------------------------------------------- | ---------------------------------------------------------------------------- |
| `std::hint::assert_unchecked`                    | 1.81    | `if !cond { unreachable_unchecked() }`                   | [Announcing Rust 1.81.0](https://blog.rust-lang.org/2024/09/05/Rust-1.81.0/) |
| `std::hint::select_unpredictable`                | 1.88    | An `if` on unpredictable data                            | [Announcing Rust 1.88.0](https://blog.rust-lang.org/2025/06/26/Rust-1.88.0/) |
| `std::hint::cold_path`                           | 1.95    | A `#[cold]` helper called only to mark a branch          | [Announcing Rust 1.95.0](https://blog.rust-lang.org/2026/04/16/Rust-1.95.0/) |
| `f32::algebraic_add` and the other algebraic ops | 1.98    | Strict left-to-right float sums that block vectorization | [Announcing Rust 1.98.0](https://blog.rust-lang.org/2026/08/20/Rust-1.98.0/) |
| Integer `format_into` with `NumBuffer`           | 1.98    | The `itoa` crate                                         | [Announcing Rust 1.98.0](https://blog.rust-lang.org/2026/08/20/Rust-1.98.0/) |

The 1.98 announcement says the algebraic methods are non-deterministic, because the compiler may choose different optimizations, and that `format_into` "performs similarly to `itoa`".

## Async

Future size is a runtime cost: `tweede-golf` measured an `async fn` taking `[u8; 1024]` by value at a 2,080-byte future, and 40 bytes taking `&mut [u8; 1024]` ([Tweede golf: Debloat your async Rust](https://tweedegolf.nl/en/blog/235/debloat-your-async-rust/), 2026-04-13, on Rust 1.94). Clippy's pedantic `large_futures` reports the same. The `tokio-blog` dial9 post profiles scheduling delays and lock contention in production ([Introducing dial9](https://tokio.rs/blog/2026-03-18-dial9), 2026-03-18), but it's a guest post about a Tokio-organisation tool, so its roster note limits it to how Tokio works. Both belong with `async.md`, and the opinion should point there.

## Coming next

Unstable on nightly as of 2026-10-19, not yet an opinion. The tracking issues are references, cited only to date the note.

| Feature                                   | Status, 2026-10-19                                                                                                                                       |
| ----------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `std::hint::likely` and `unlikely`        | Unstable ([rust-lang/rust#151619](https://github.com/rust-lang/rust/issues/151619)). `cold_path` was stabilised without them                             |
| Portable SIMD, `std::simd`                | Unstable ([rust-lang/rust#86656](https://github.com/rust-lang/rust/issues/86656))                                                                        |
| `#[optimize(speed)]`, `#[optimize(size)]` | Unstable ([rust-lang/rust#54882](https://github.com/rust-lang/rust/issues/54882))                                                                        |
| `build-std`                               | Nightly-only `-Zbuild-std`. A 2026 project goal is finishing the RFCs ([Rust Project Goals: build-std](https://goals.rust-lang.org/2026/build-std.html)) |
| `-Zprint-type-sizes`                      | Nightly-only. The book uses it to find large types ([Type Sizes](https://nnethercote.github.io/perf-book/type-sizes.html))                               |

Compiled on 1.99.1, `likely`, `std::simd`, and `#[optimize]` each fail with `E0658`.

## Watch-list and unvetted material

Kept apart from the roster findings above. None of it carried a claim.

- **Watch-list `ms-rust-guidelines`:** Its performance chapter has eleven guidelines, among them M-HOTPATH (profile the hot path early), M-MEM-REUSE, M-INITIAL-CAPACITY, M-FAST-HASHER, M-BOX-DST, and M-ASYNC-STACK-SIZE ([Microsoft: Pragmatic Rust Guidelines: Performance](https://microsoft.github.io/rust-guidelines/guidelines/performance/index.html)). Every one agrees with a `nnethercote` or `tweede-golf` rule above. The worker read only summaries, so check the wording before quoting it anywhere.
- **Unvetted, Jakub Beránek (Kobzol), a rustc performance team member:** ["Optimizing Rust programs with PGO and BOLT using cargo-pgo"](https://kobzol.github.io/rust/cargo/2023/07/28/rust-cargo-pgo.html) (2023-07-28) walks through `cargo pgo build`, a representative run of at least a minute, and `cargo pgo optimize`, and warns that the gain isn't guaranteed. `cargo-wizard` offers a `fast-runtime` template. Kobzol writes the tools, so it's a conflict as well as a candidate.
- **Unvetted crate documentation, used as references:** samply suggests a profiling profile with `debug = true`, where the book uses `"line-tables-only"`. The `mimalloc` crate's `secure` feature costs "usually around 10%".

## What this means for the repository

- **No opinion covers the topic.** Run `resolve-research` to create `opinions/runtime-performance.md`. Its first sentence is the opening list here, and it needs sections for measuring, the release profile, profiling, allocation and copying, hashing and the global allocator, inlining and hints, and Coming next. List it under README.md's Scope and in the layout tree.
- **`templates/Cargo.toml` gains two profiles:** `[profile.release]` with `codegen-units = 1` and `lto = "fat"`, and `[profile.profiling]` inheriting `release` with `debug = "line-tables-only"`. Its header then cites `nnethercote`.
- **Two notes for `refresh-rust-versions` or `harvest-sources`:** the book's v0 mangling tip is redundant since 1.97, and `matklad`'s 2021 `#[inline]` advice is superseded for leaf functions since 1.75.
- **Leave async, CI, and compile times to their topics.** Future size and dial9 go to `async.md`, the CI-only profile variables go to `ci.md`, and the linker (`lld` is the default on `x86_64-unknown-linux-gnu` since 1.90) goes with compile times.
- **The opinion has to pick a benchmark harness.** No roster source chooses between Criterion and Divan, so `resolve-research` either finds one that does or leaves the harness to `testing.md`.
- **A `vet-source` candidate:** Jakub Beránek's blog, on PGO and Cargo profiles, from a rustc performance team member. Declare the conflict with `cargo-pgo` and `cargo-wizard`.
