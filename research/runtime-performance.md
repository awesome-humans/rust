---
targets: [rust-1.99, edition-2024]
last-reviewed: 2026-11-12
sources:
  [
    corrode,
    matklad,
    nnethercote,
    rust-blog,
    rust-docs,
    sunshowers,
    tokio-blog,
    tweede-golf,
  ]
---

# Runtime performance: profiling, allocation, and release profile settings

Measure first, then tune what the measurements show is slow, and keep the build settings that help every program in the workspace root:

- **Set `codegen-units = 1` and `lto = "fat"` in `[profile.release]`** of a workspace that ships a binary, and leave `panic` at `"unwind"`.
- **Profile a release build with `debug = "line-tables-only"` in `[profile.release]`.** Profile with samply, and find allocation hot spots with DHAT, or with dhat-rs where Valgrind doesn't run.
- **Cut allocations before anything else in a hot path:** pre-size collections, reuse buffers, buffer and lock I/O, and let Clippy's warn-by-default `perf` group catch the known patterns.
- **Change the hasher or the global allocator only when a benchmark shows a gain.** Use a faster hasher only for keys an attacker can't choose, and set a global allocator only in a binary.
- **Treat `#[inline]`, `-C target-cpu=native`, and PGO as last steps,** each with a measured reason.

Everything here is stable in Rust 1.99.1 (released 2026-10-15), which was still the latest stable release on 2026-11-12. Items that aren't on stable yet are in [Coming next](#coming-next), as of 2026-11-12.

**No opinion file covers this.** `opinions/` holds only `rust.md`. This topic is the `runtime-performance.md` row of [PLAN.md: Phase 6](../PLAN.md#phase-6-first-opinions-and-templates), whose expected source, `perf-book`, is admitted as `nnethercote`. Compile times and linkers are out of scope: they belong with `project-structure.md` and `ci.md`.

## Measure first

`nnethercote` puts measurement first: "Benchmark all changes, one at a time, to ensure they have the expected effects", and "it is only worth optimizing hot code" ([The Rust Performance Book: General Tips](https://nnethercote.github.io/perf-book/general-tips.html)). The largest gains usually come from changing an algorithm or data structure, not from build flags. The book also warns that memory layout alone can cause "significant but ephemeral" swings, so a wall-time difference needs repeat runs ([Benchmarking](https://nnethercote.github.io/perf-book/benchmarking.html)).

`matklad` adds two working rules for benchmarks. Tune the input so a run takes about 300 milliseconds: under about 10 milliseconds, fixed costs distort it ([Alex Kladov: Benchmark In Milliseconds](https://matklad.github.io/2026/10/05/benchmark-milliseconds.html), 2026-10-05). Make benchmark inputs overridable at run time, and print or hash the result, so the compiler can't delete or constant-fold the work. Don't rely on `std::hint::black_box` alone ([Do Not Optimize Away](https://matklad.github.io/2025/12/09/do-not-optimize-away.html), 2025-12-09). `black_box` has been stable since 1.66, and its documentation calls it a hint, not a guarantee ([`std::hint::black_box`](https://doc.rust-lang.org/std/hint/fn.black_box.html)).

**Benchmark harnesses.** `#[bench]` is nightly-only. `nnethercote` lists Criterion, Divan, Gungraun (renamed from Iai-Callgrind on 2026-04-23) for instruction counts, Bencher for continuous benchmarking, and hyperfine for whole programs, without ranking them ([Benchmarking](https://nnethercote.github.io/perf-book/benchmarking.html)). No roster source picks between Criterion and Divan. Divan's own README shows `#[divan::bench]` attributes and `cargo test --benches` running each benchmark once as a test ([divan](https://github.com/nvzqz/divan), crate documentation). [What this means for the repository](#what-this-means-for-the-repository) leaves the choice open.

## Release profile

Cargo's built-in `release` profile is `opt-level = 3`, `lto = false`, `codegen-units = 16`, `panic = "unwind"`, `debug = false`, and `incremental = false` ([The Cargo Book: Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html#release)). `lto = false` doesn't turn LTO off: it means thin local LTO across the crate's own codegen units. `lto = "off"` turns it off. Cargo reads profiles only from the workspace root manifest, and ignores them in dependencies, so these settings are for the workspace that builds a binary.

```toml
[profile.release]
codegen-units = 1
lto = "fat"

[profile.profiling]
inherits = "release"
debug = "line-tables-only"
```

The editor's probe on 1.99.1, on 2026-11-12, confirms what Cargo passes: `release` gets `-C codegen-units=1 -C lto=fat -C opt-level=3 -C strip=debuginfo`, and `profiling` gets the same with `-C debuginfo=line-tables-only` and no `strip`.

**Unsupported: the separate `profiling` profile.** The opening list leaves it out. `nnethercote` puts `debug = "line-tables-only"` in `[profile.release]` itself ([Profiling](https://nnethercote.github.io/perf-book/profiling.html)). No roster source recommends a separate profile, so it's the editor's reading. The Cargo Book documents a custom profile with `inherits` ([The Cargo Book: Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html#custom-profiles)), and a profile with `debug` on isn't stripped, so the book's form leaves line tables in the release binary. samply's README, a reference, suggests the same profile with `debug = true` ([samply](https://github.com/mstange/samply), crate documentation). `resolve-research` needs a roster source for the separate profile, or it uses the book's `[profile.release]` form.

| Setting                     | Effect                                                                                     | Cost                                              | Source                                                                                                                                                          |
| --------------------------- | ------------------------------------------------------------------------------------------ | ------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `codegen-units = 1`         | Better code than thin local LTO; smaller binary                                            | Slower builds: 4.5 s to 9.7 s on `image` 0.24.1   | `nnethercote`: [Back-end parallelism in the Rust compiler](https://nnethercote.github.io/2023/07/11/back-end-parallelism-in-the-rust-compiler.html), 2023-07-11 |
| `lto = "fat"`               | "can improve runtime speed by 10-20% or more", and shrinks the binary; `"thin"` is cheaper | Compile time; occasionally slower than `"thin"`   | `nnethercote`: [Build Configuration](https://nnethercote.github.io/perf-book/build-configuration.html)                                                          |
| `panic = "abort"`           | "might … increase runtime speed slightly"                                                  | No unwinding: `catch_unwind` stops working        | `nnethercote`, same chapter; tests and benchmarks ignore the key ([Cargo Book](https://doc.rust-lang.org/cargo/reference/profiles.html#panic))                  |
| `-C target-cpu=native`      | Uses every instruction the build machine has                                               | The binary might not run on another CPU           | `nnethercote`, same chapter                                                                                                                                     |
| Profile-guided optimization | "can improve runtime speed by 10% or more"                                                 | A training run per build; not for `cargo install` | `nnethercote`, same chapter; steps in [The rustc book: PGO](https://doc.rust-lang.org/rustc/profile-guided-optimization.html)                                   |

`corrode` gives the same pair, `lto = true` (the same as `"fat"`) and `codegen-units = 1`, and shows setting them only in CI through `CARGO_PROFILE_RELEASE_LTO` and `CARGO_PROFILE_RELEASE_CODEGEN_UNITS` ([Matthias Endler: Tips for Faster Rust CI Builds](https://corrode.dev/blog/tips-for-faster-ci-builds/), 2025-01-28, updated 2025-08-16). That's the same trade-off applied in CI, and it belongs in `ci.md`.

**Keep `panic = "unwind"`.** `nnethercote`'s maximum-speed summary includes `panic = "abort"` beside `codegen-units = 1`, `lto = "fat"`, and an alternative allocator. Its own text describes the gain as "slightly", and the Cargo Book records that the test harness needs unwinding. Aborting also changes behaviour, not just speed: a panic in one thread or task ends the process instead of being caught. So the topic treats it as a behaviour decision with a small speed gain, and recommends keeping `"unwind"` unless a binary is already designed to exit on any panic.

`research/error-handling.md` reaches the same key from two more roster sources. `sunshowers` writes that "reusable Rust libraries _cannot_ require `panic = 'abort'`" ([Rain: In defense of lock poisoning in Rust](https://sunshowers.io/posts/on-poisoning/), 2025-12-02), and `corrode` says "Panic behavior is part of your system's failure model" ([Matthias Endler: Hardening Rust Code For Production](https://corrode.dev/blog/hardening-rust/), 2026-07-23). That topic proposes that a binary sets `panic` on purpose and a library stays correct under unwinding, which agrees with this one.

**`strip` needs no setting.** Since 1.77, Cargo sets `strip = "debuginfo"` for any profile that doesn't enable debug info ([Rust Blog: Announcing Rust 1.77.0](https://blog.rust-lang.org/2024/03/21/Rust-1.77.0/)), and the book shortened its section on 2025-09-22 because of it. A profile with `debug` on, such as `profiling`, isn't stripped.

**Target CPU.** For a binary that runs on more machines than it's built on, detect features at run time instead of setting `target-cpu`. `tweede-golf` does this in zlib-rs with a generic `#[inline(always)]` body, a wrapper with `#[target_feature(enable = "avx2")]`, and dispatch on `is_x86_feature_detected!`. `std` caches that check in an atomic, but it still shouldn't run in a hot loop ([Tweede golf: SIMD in zlib-rs, part 1](https://tweedegolf.nl/en/blog/153/simd-in-zlib-rs-part-1-autovectorization-and-target-features/), 2025-04-15). The same post shows a plain loop over `chunks_exact_mut` compiling to SIMD instructions without intrinsics.

**PGO.** The rustc book gives the four steps: build with `-Cprofile-generate`, run representative workloads, merge with `llvm-profdata` from the `llvm-tools-preview` component, and rebuild with `-Cprofile-use` and identical flags. It names `cargo-pgo` as a simpler wrapper ([The rustc book: Profile-guided optimization](https://doc.rust-lang.org/rustc/profile-guided-optimization.html)). `nnethercote` reports that the training set decides the result: adding rustdoc benchmarks to rustc's PGO training cut rustdoc's mean wall time by 2.85% ([How to speed up the Rust compiler in July 2026](https://nnethercote.github.io/2026/07/31/how-to-speed-up-the-rust-compiler-in-july-2026.html), 2026-07-31).

## Profiling

Profile a release build with line tables, never a `dev` build: the book reports 10 to 100 times speed-ups from release builds ([Build Configuration](https://nnethercote.github.io/perf-book/build-configuration.html)).

| Tool                     | Use it for                                                 | Platforms             |
| ------------------------ | ---------------------------------------------------------- | --------------------- |
| samply                   | Sampling CPU profiles, viewed in the Firefox Profiler      | Linux, macOS, Windows |
| perf                     | Sampling with hardware counters                            | Linux                 |
| Cachegrind and Callgrind | Instruction counts per function and line; low-variance A/B | Linux, some Unixes    |
| DHAT (Valgrind)          | Allocation hot spots, peak memory, hot `memcpy`            | Linux, some Unixes    |
| dhat-rs                  | DHAT-style heap profiles and heap-usage regression tests   | All                   |
| heaptrack, bytehound     | Heap profiles                                              | Linux                 |

All rows are from `nnethercote` ([Profiling](https://nnethercote.github.io/perf-book/profiling.html)), which also names Instruments, VTune, AMD uProf, `cargo flamegraph`, and Coz, and says to use more than one profiler. The book uses Cachegrind and DHAT most ([Nicholas Nethercote: Rust and Valgrind](https://nnethercote.github.io/2022/01/05/rust-and-valgrind.html), 2022-01-05). Its DHAT guideline, from rustc: cutting 10 allocations per million instructions can give about a 1% speed-up ([Heap Allocations](https://nnethercote.github.io/perf-book/heap-allocations.html)).

Three facts update the book's profiling chapter for 1.99:

- **Frame pointers:** Build with `RUSTFLAGS="-C force-frame-pointers=yes"` for frame-pointer unwinding ([Profiling](https://nnethercote.github.io/perf-book/profiling.html)). The prebuilt `std` already has them, since 1.79 ([rust-lang/rust#122646](https://github.com/rust-lang/rust/pull/122646), a reference).
- **Symbol names:** v0 mangling is the default on stable since 1.97, and keeps generic arguments in the symbol instead of a hash ([Rust Blog: Announcing Rust 1.97.0](https://blog.rust-lang.org/2026/07/09/Rust-1.97.0/)). The book's tip to opt in with `-C symbol-mangling-version=v0` is now redundant, and a profiler needs v0 support to show readable names, which `rustfilt` has.
- **Debug info for `std`:** The prebuilt `std` still ships without it, so `std` frames have no file and line ([Profiling](https://nnethercote.github.io/perf-book/profiling.html)).

## Allocation and copying

Each rule here is from the `nnethercote` chapter named, except where marked. The details depend on the allocator, but an allocation "typically involves acquiring a global lock", some bookkeeping, and possibly a system call. A small one isn't necessarily cheaper than a large one ([Heap Allocations](https://nnethercote.github.io/perf-book/heap-allocations.html)).

- **Pre-size:** `Vec::with_capacity` or `reserve` when the length is known. `std` guarantees no growth strategy ([`Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html#guarantees)), but the book's example shows the current one: pushing 20 elements into an empty `Vec` allocated four times, at capacities 4, 8, 16, and 32.
- **Reuse:** Keep a working `Vec` or `String` outside the loop and `clear()` it. Use `clone_from` to reuse the destination's allocation.
- **Read lines into one buffer:** `BufRead::lines()` allocates a `String` per line. Use `read_line` and clear the buffer ([I/O](https://nnethercote.github.io/perf-book/io.html)).
- **Buffer and lock I/O:** `println!` locks stdout on every call, and files are unbuffered. Wrap a locked handle in `BufWriter`, and call `flush()` yourself, because dropping it ignores errors.
- **Box immutable sequences:** A `Box<[T]>` is two words instead of three. `Vec::into_boxed_slice` drops any excess capacity, "which may cause a reallocation". Collecting into a `Box<[T]>` directly avoids that only if the iterator's "length is known in advance" ([Type Sizes](https://nnethercote.github.io/perf-book/type-sizes.html)).
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
        // Strip what `lines()` strips: one `\n` or `\r\n`.
        let text = line
            .strip_suffix('\n')
            .map_or(line.as_str(), |l| l.strip_suffix('\r').unwrap_or(l));
        writeln!(out, "{}", text.len())?;
        line.clear();
    }
    out.flush()
}
```

The book guards sizes with the `static_assertions` crate. On 1.99, `size_of` is in the prelude and a `const` assertion needs no crate. This is a fragment: on its own, the unread fields trigger `dead_code`.

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

The two programs pass `CARGO_BUILD_WARNINGS=deny cargo check`, `CARGO_BUILD_WARNINGS=deny cargo clippy`, and `cargo clippy -- -D warnings -W clippy::pedantic` on 1.99.1, with no crate beyond `std` (editor's probe, 2026-11-12). The fragment passes the same checks only inside a `main` that reads its fields, and fails `dead_code` on its own.

**Clippy enforces part of this.** The `perf` group is warn-by-default and has 38 lints on the stable list, including `large_enum_variant`, `box_collection`, `redundant_allocation`, `slow_vector_initialization`, `unnecessary_to_owned`, `unbuffered_bytes`, and `regex_creation_in_loops` ([Clippy lints](https://clippy.rust-lang.org/stable/index.html); the count and every group named in this paragraph checked with `cargo clippy -- -W help` on 1.99.1, 2026-11-14). The book doesn't repeat what Clippy catches by default ([Linting](https://nnethercote.github.io/perf-book/linting.html)). Related lints outside `perf`: `large_types_passed_by_value`, `trivially_copy_pass_by_ref`, `needless_pass_by_value`, `inefficient_to_string`, `large_futures`, and `large_stack_arrays` are pedantic, and `redundant_clone` and `needless_collect` are nursery.

**Global allocator.** Alternative allocators have given "large improvements in runtime speed and large reductions in memory usage", varying by platform, at some cost in binary size and compile time ([Build Configuration](https://nnethercote.github.io/perf-book/build-configuration.html)). The book shows `mimalloc`, which works on many platforms, and `tikv-jemallocator`, for Linux and macOS. A program has one `#[global_allocator]`, so only a binary sets it ([`std::alloc`](https://doc.rust-lang.org/std/alloc/index.html#the-global_allocator-attribute)).

**Hashing.** `HashMap` uses SipHash 1-3 for HashDoS resistance, and the `std` documentation says other algorithms are faster for small keys such as integers, without that protection ([`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html)). For a hot map whose keys an attacker can't choose, `nnethercote` recommends `rustc-hash`'s `FxHashMap`. In rustc, moving back from Fx to the default hasher cost 4 to 84% ([Hashing](https://nnethercote.github.io/perf-book/hashing.html)), The 2021 post's warning that Fx has weak high bits, so a key that packs its entropy there collides ([A brutally effective hash function in Rust](https://nnethercote.github.io/2021/12/08/a-brutally-effective-hash-function-in-rust.html), 2021-12-08), describes the 1.x algorithm. `rustc-hash` 2.0.0 replaced the Fx algorithm, which came from Firefox, with a new hasher that "has better theoretical properties" ([rustc-hash: CHANGELOG](https://github.com/rust-lang/rustc-hash/blob/master/CHANGELOG.md), a reference, read 2026-11-14). Whether 2.1.3 still has the weakness isn't checked, so the opinion leaves the warning out. The book doesn't mention `foldhash`, `hashbrown`'s default hasher, which calls itself "minimally DoS-resistant" ([foldhash](https://docs.rs/foldhash/latest/foldhash/), crate documentation). The book adds one step for "if you decide to universally use one of the alternatives": list `std::collections::HashMap` and `HashSet` in `clippy.toml`'s `disallowed-types` ([Linting](https://nnethercote.github.io/perf-book/linting.html)). A workspace with a map whose keys an attacker can choose doesn't meet that condition, so don't set the lint there.

**Advisories.** The crates named here were checked against the [RustSec Advisory Database](https://rustsec.org/) on 2026-11-12, at their latest releases: `rustc-hash` 2.1.3, `mimalloc` 0.1.52, `tikv-jemallocator` 0.7.0, and `dhat` 0.3.3, which is dhat-rs. None has an open advisory. `mimalloc` has one earlier advisory, for unsound alignment, patched in 0.1.39 ([RustSec: RUSTSEC-2022-0094](https://rustsec.org/advisories/RUSTSEC-2022-0094.html)). samply is an installed tool, not a dependency, and has no advisory.

## Inlining and hints

**Add `#[inline]` only with a measurement.** `nnethercote` says the best candidates are very small functions or ones with a single call site: "The compiler will often inline these functions itself even without an inline attribute. But the compiler cannot always make the best choices, so attributes are sometimes needed." The attributes' effect is "unpredictable", so re-measure after adding one ([Inlining](https://nnethercote.github.io/perf-book/inlining.html)). Since 1.75, rustc also makes small functions available for inlining across crates on its own ([rust-lang/rust#116505](https://github.com/rust-lang/rust/pull/116505), a reference).

**Where the roster disagrees.** `matklad` told library authors to "proactively add `#[inline]` to small non-generic functions", and to prefer `lto = true` for applications ([Alex Kladov: Inline In Rust](https://matklad.github.io/2021/07/09/inline-in-rust.html), 2021-07-09). `nnethercote` asks for a measurement first. The topic follows `nnethercote`, because a measurement settles each case on any compiler version. The post's other points still hold: generic functions are already available for inlining, `#[inline]` isn't transitive, and an inlined body is compiled again in each codegen unit.

**Unsupported: that 1.75 supersedes the 2021 advice.** The post predates 1.75. The editor's reading is that 1.75's automatic cross-crate inlining covers the trivial leaf functions the post meant, and that the advice still holds for a non-generic public function that wraps a call the compiler won't treat as small. That reading rests on the pull request, a reference, so it needs a roster source or a measurement before an opinion says it.

**Mark the cold path, not the hot one.** Move rarely run code into a `#[cold]` function ([Inlining: Outlining](https://nnethercote.github.io/perf-book/inlining.html)), or call `std::hint::cold_path()` in a branch, stable since 1.95 ([Rust Blog: Announcing Rust 1.95.0](https://blog.rust-lang.org/2026/04/16/Rust-1.95.0/)).

**Recent stable items that replace a crate or a trick:**

| Use                                              | Release | Instead of                                               | Source                                                                       |
| ------------------------------------------------ | ------- | -------------------------------------------------------- | ---------------------------------------------------------------------------- |
| `std::hint::assert_unchecked`                    | 1.81    | `if !cond { unreachable_unchecked() }`                   | [Announcing Rust 1.81.0](https://blog.rust-lang.org/2024/09/05/Rust-1.81.0/) |
| `std::hint::select_unpredictable`                | 1.88    | An `if` on unpredictable data                            | [Announcing Rust 1.88.0](https://blog.rust-lang.org/2025/06/26/Rust-1.88.0/) |
| `std::hint::cold_path`                           | 1.95    | A `#[cold]` helper called only to mark a branch          | [Announcing Rust 1.95.0](https://blog.rust-lang.org/2026/04/16/Rust-1.95.0/) |
| `f32::algebraic_add` and the other algebraic ops | 1.98    | Strict left-to-right float sums that block vectorization | [Announcing Rust 1.98.0](https://blog.rust-lang.org/2026/08/20/Rust-1.98.0/) |
| Integer `format_into` with `NumBuffer`           | 1.98    | The `itoa` crate                                         | [Announcing Rust 1.98.0](https://blog.rust-lang.org/2026/08/20/Rust-1.98.0/) |

The 1.98 announcement says the algebraic methods are non-deterministic, because the compiler might choose different optimizations, and that `format_into` "performs similarly to `itoa`".

## Async

Future size is a runtime cost: `tweede-golf` measured an `async fn` taking `[u8; 1024]` by value at a 2,080-byte future, and 40 bytes taking `&mut [u8; 1024]` ([Tweede golf: Debloat your async Rust](https://tweedegolf.nl/en/blog/235/debloat-your-async-rust/), 2026-04-13, on Rust 1.94). Clippy's pedantic `large_futures` reports the same. The `tokio-blog` dial9 post profiles scheduling delays and lock contention in production ([Introducing dial9](https://tokio.rs/blog/2026-03-18-dial9), 2026-03-18), but it's a guest post about a Tokio-organisation tool, and `tokio-blog`'s note forbids citing it on adopting a crate from the same organisation. So no opinion can recommend dial9 on it. Future size belongs with `async.md`, and the opinion should point there.

## Coming next

Not on stable as of 2026-11-12, and not yet an opinion. Each is unstable on nightly, except `#[optimize]`, which is stabilised for 1.101 unless an open pull request reverts part of it. The tracking issues and pull requests are references, cited only to date the note.

| Feature                                   | Status, 2026-11-12                                                                                                                                                                                                                                                                                                                |
| ----------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `std::hint::likely` and `unlikely`        | Unstable ([rust-lang/rust#151619](https://github.com/rust-lang/rust/issues/151619)). `cold_path` was stabilised without them                                                                                                                                                                                                      |
| Portable SIMD, `std::simd`                | Unstable ([rust-lang/rust#86656](https://github.com/rust-lang/rust/issues/86656))                                                                                                                                                                                                                                                 |
| `#[optimize(speed)]`, `#[optimize(size)]` | Stabilised for 1.101 by [rust-lang/rust#157273](https://github.com/rust-lang/rust/pull/157273), merged 2026-10-08, which closed [rust-lang/rust#54882](https://github.com/rust-lang/rust/issues/54882). [rust-lang/rust#165592](https://github.com/rust-lang/rust/pull/165592), open, would make `optimize(speed)` unstable again |
| `build-std`                               | Nightly-only `-Zbuild-std`. A 2026 project goal is finishing the RFCs ([Rust Project Goals: build-std](https://goals.rust-lang.org/2026/build-std.html))                                                                                                                                                                          |
| `-Zprint-type-sizes`                      | Nightly-only. The book uses it to find large types ([Type Sizes](https://nnethercote.github.io/perf-book/type-sizes.html))                                                                                                                                                                                                        |

Compiled on 1.99.1, `likely`, `std::simd`, and `#[optimize]` each fail with `E0658` (editor's probe, 2026-11-12).

## Watch-list and unvetted material

Kept apart from the roster findings above. None of it supported a claim.

- **Watch-list `ms-rust-guidelines`:** Its performance chapter has eleven guidelines, among them M-HOTPATH (profile the hot path early), M-MEM-REUSE, M-INITIAL-CAPACITY, M-FAST-HASHER, M-BOX-DST, and M-ASYNC-STACK-SIZE ([Microsoft: Pragmatic Rust Guidelines: Performance](https://microsoft.github.io/rust-guidelines/guidelines/performance/index.html)). Every one agrees with a `nnethercote` or `tweede-golf` rule above. The worker read only summaries, so check the wording before quoting it anywhere.
- **Unvetted, Jakub Beránek (Kobzol), a rustc performance team member:** ["Optimizing Rust programs with PGO and BOLT using cargo-pgo"](https://kobzol.github.io/rust/cargo/2023/07/28/rust-cargo-pgo.html) (2023-07-28) walks through `cargo pgo build`, a representative run of at least a minute, and `cargo pgo optimize`, and warns that the gain isn't guaranteed. `cargo-wizard` offers a `fast-runtime` template. Kobzol writes the tools, so the blog has a conflict as well as being a candidate.
- **Unvetted crate documentation, used as references:** samply suggests a profiling profile with `debug = true`, where the book uses `"line-tables-only"`. The `mimalloc` crate's `secure` feature costs "usually around 10%".

## What this means for the repository

- **No opinion covers the topic.** Run `resolve-research` to create `opinions/runtime-performance.md`. Its first sentence is the opening list here, and it needs sections for measuring, the release profile, profiling, allocation and copying, hashing and the global allocator, inlining and hints, and Coming next. List it under README.md's Scope and in the layout tree.
- **`templates/Cargo.toml` gains two profiles, for a workspace that ships a binary:** `[profile.release]` with `codegen-units = 1` and `lto = "fat"`. The template is the generic workspace root, so a comment above `[profile.release]` says to delete the two keys in a library-only workspace, where they slow the release build and no binary gains. Its header then cites `nnethercote`. The opinion gives `nnethercote`'s profiling setting, `debug = "line-tables-only"` in `[profile.release]`, and the template leaves it out because it costs every release build. A separate `[profile.profiling]` is unsupported, so it waits for a roster source. `corrode`'s CI-only variables are the alternative for a workspace that wants fast local release builds. `research/ci.md` doesn't mention them (checked 2026-11-14), so `resolve-research` adds them to the CI opinion.
- **One note for `refresh-rust-versions` or `harvest-sources`:** the book's v0 mangling tip is redundant since 1.97. That `matklad`'s 2021 `#[inline]` advice is superseded for leaf functions since 1.75 is unsupported, and needs a roster source or a measurement first.
- **Leave async, CI, and compile times to `async.md`, `ci.md`, and `project-structure.md`.** Future size goes to `async.md`, and the CI-only profile variables go to `ci.md`. dial9 goes nowhere until a source other than `tokio-blog` covers it. The linker goes to `project-structure.md`, whose topic has the build-time section: `lld` is the default on `x86_64-unknown-linux-gnu` since 1.90 ([Rust Blog: Announcing Rust 1.90.0](https://blog.rust-lang.org/2025/09/18/Rust-1.90.0/)).
- **`panic` has two topics.** This one and `research/error-handling.md` both propose a rule for `panic` in `[profile.release]`, and they agree. `resolve-research` states it in `error-handling.md`, which has the failure model, and links it from the release profile section of this opinion.
- **Rust 1.100 was due on 2026-11-12,** and 1.99.1 was still the stable release when this was checked that day. `refresh-rust-versions` retargets the topic and rechecks the [Coming next](#coming-next) table. `#[optimize]` moves to stable guidance in 1.101 only if [rust-lang/rust#165592](https://github.com/rust-lang/rust/pull/165592), which would make `optimize(speed)` unstable again, hasn't merged by then. It was open on 2026-11-14.
- **The opinion has to pick a benchmark harness.** No roster source chooses between Criterion and Divan, so `resolve-research` either finds one that does or leaves the harness to `testing.md`.
- **A `vet-source` candidate:** Jakub Beránek's blog, on PGO and Cargo profiles, from a rustc performance team member. Declare the conflict with `cargo-pgo` and `cargo-wizard`.
