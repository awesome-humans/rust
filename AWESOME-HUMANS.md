# Awesome humans

These are the vetted sources behind every opinion in this repository. An opinion can use content only if it traces to a source listed here, or carries a reserved `house:<profile>` ID described below.

To propose a source, [suggest it for vetting](https://github.com/awesome-humans/website/issues/new?template=suggest-source.yml&ecosystem=Rust). If you're a listed source and don't want your work cited, [ask to opt out](https://github.com/awesome-humans/website/issues/new?template=opt-out.yml). For how both work, see [Want in or out?](https://awesome-humans.net/want-in-or-out/)

## Admission criteria

**Scope: published writing.** A source qualifies on text a reader can check: blog posts, documentation, books, and newsletters. Videos, conference talks, and podcasts are out of scope for now, both as citable material and as evidence of a track record. A source that moves to those formats stops qualifying, and becomes a `vet-source` candidate again if the writing resumes.

A **source** is the published work of an awesome human: a person or a publication. Admission needs an **established track record**:

1. **Longevity:** At least two years of sustained publishing. A source with less goes on the watch list, however good the writing. The `Since` column shows how far back each record goes.
2. **Depth:** Original insight, such as internals, measurements, or worked reasoning. Paraphrased release notes don't count. A long-lived source with thin depth is admitted with the `**Corroborate.**` marking, which limits how it's cited.
3. **Accuracy:** A history of being right, and of publishing corrections when wrong.
4. **Independence:** The content stands on its merit, not on marketing reach.

Two rules qualify these criteria:

- **An independence concern is a usage limit, not a lower rank.** A concern is live when vendor or project marketing produces adoption-focused content with little critical distance. The source is still admitted, and its section under `## Source notes` says what a citation can rest on, usually how something works, not whether to adopt it. `ferrous-systems`, `tokio-blog`, and `tweede-golf` each record one.
  - Maintaining a crate isn't a concern by itself. A conflict is recorded where an opinion might recommend the source's own crate or product, as for `predrag` and `sunshowers`.
  - A concern that no limit could contain leads to a decline or the watch list.
- **Dormancy blocks admission, with one exception.** A candidate who hasn't published for over a year goes on the watch list, unless the person's **writing** clearly continues elsewhere, such as official documentation, another publication, or a book. Then the track record follows the person, and the notes record the dormant channel.
  - Repository activity isn't evidence. Opinions don't cite commits, releases, or issue threads, so a busy GitHub profile beside a silent blog is still dormant.
  - Documentation written in a repository counts. The test is what was published, not where the commits landed. `burntsushi` is admitted on this basis.
  - The same rule triggers re-evaluation of admitted sources. For details, see `vet-source`.

The [Watch list](#watch-list-track-record-still-forming-or-previously-strong-but-now-dormant) tracks candidates that don't qualify yet, and `vet-source` re-evaluates them. The decision log at the end records every admission and demotion.

### How the roster is organised

- **IDs:** Each source has a stable `id`, which `sources:` metadata uses.
- **Order:** Rows in every roster table, and the sections under [Source notes](#source-notes), are sorted by `id`. Insert a new source in order.
- **Tables:** A row holds only what identifies a source: its ID, its kind, where to read it, what it covers, and how far back it goes.
- **Kind:** `person` or `publication`. A newsletter or link blog that one person writes is still a `publication`. A person's Source link text starts with their name, as in `Ralf Jung: Ralf's Ramblings`, because the site takes the name from it.
- **Notes:** The evidence behind a row goes in its notes section, in the shape [Source notes](#source-notes) describes, one sentence per line. Rows stay short, and a reworded sentence shows in a diff as that one sentence.
- **Decision log:** Each entry gives the decision and the fact behind it. The evidence is in the source's notes and the pull request. It's the only table Prettier doesn't align: `<!-- prettier-ignore -->` stops one long Detail cell from realigning every row.

### Reserved IDs, markings, and references

**Reserved prefix: `house:`.** A house profile's opinions use the source ID `house:<profile>`, such as `house:simon-mckenna`, described in [HOUSE-OPINIONS.md](HOUSE-OPINIONS.md). A profile's authority is its owner's experience, not a track record, so it skips the admission criteria, and every house opinion is visibly marked instead. [HOUSE-OPINIONS.md: How this works](HOUSE-OPINIONS.md#how-this-works) is the canonical definition of the marking, of how a house opinion sits beside the sourced text, and of contributor adoption. A `house:` ID never appears in the roster tables.

**Reserved marking: `**Discovery-only.**`.** Aggregators, such as link roundups, newsletters, and curated lists, are admitted for discovery and never cited. They lead to a primary source, and the opinion cites that. CI rejects the ID of a source whose notes carry this marking in any `sources:` list under `opinions/` or `templates/`. Like `**House:**`, it's a fixed string, so a check can match it exactly.

**Reserved marking: `**Corroborate.**`.** Some sources have a long track record but thin depth: community catalogues, SEO-shaped roundups, or release paraphrase with occasional substance. They're admitted and marked. The marking limits how you cite the source, not its standing. An opinion can cite it, but never alone: every claim it supports also needs an unmarked citable source. `just validate-sources` rejects an opinion or template whose citable sources are all marked. Neither a `house:` ID nor a Discovery-only ID counts as the unmarked source.

**References: a named crate's own material.** When an opinion or template names a crate, you can link the project's own material inline as a reference: its documentation site or in-repository docs, its docs.rs pages, release notes and changelogs, issues and pull requests, or crates.io page. The standard behind a named format or protocol qualifies on the same terms, such as an RFC. So do a RustSec advisory and an accepted RFC in the RFC Book, for what they record.

- A reference corroborates a claim and never carries it. It can confirm how something is set up, what it supports, and where it falls short, but the claim still rests on a roster source. A crate's own material never decides whether to adopt that crate.
- A reference needs no `vet-source` pass and never appears in `sources:`, so `just validate-sources` doesn't see it.
- Cite it in the same `Source: Title` form as anything else, as in `[Clap: Derive Tutorial]`.

## Citable: quotable in opinions and templates

Every source here meets all four criteria. The `Since` column shows the length of each record, and any independence concern, conflict of interest, or usage limit is in the source's section under [Source notes](#source-notes).

The table lists 23 sources: 12 people and 11 publications.

| id                  | Kind        | Source                                                                                                                                              | Focus                                                                       | Since |
| ------------------- | ----------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- | ----- |
| `alice-ryhl`        | person      | [Alice Ryhl](https://ryhl.io/blog/)                                                                                                                 | Tokio and async design, Rust for Linux, smart pointers and in-place init    | 2020  |
| `anssi-rust`        | publication | [ANSSI: Secure Rust Guidelines](https://anssi-fr.github.io/rust-guide/)                                                                             | Secure development, `unsafe`, FFI, tooling for security-critical software   | 2018  |
| `awesome-rust`      | publication | [awesome-rust](https://github.com/rust-unofficial/awesome-rust)                                                                                     | Curated crate and tool list                                                 | 2014  |
| `blessed-rs`        | publication | [Blessed.rs](https://blessed.rs/crates)                                                                                                             | Hand-picked default crates by task                                          | 2022  |
| `burntsushi`        | person      | [Andrew Gallant: burntsushi.net](https://burntsushi.net/) and the [Jiff design documents](https://github.com/BurntSushi/jiff/blob/master/DESIGN.md) | Error handling, regex and text search internals, datetime API design        | 2014  |
| `corrode`           | person      | [Matthias Endler: corrode.dev](https://corrode.dev/blog/) and [endler.dev](https://endler.dev/)                                                     | API design, defensive programming, standard library pitfalls, compile times | 2017  |
| `fasterthanlime`    | person      | [Amos Wenger: fasterthanli.me](https://fasterthanli.me/articles)                                                                                    | Long-form deep dives: async, networking, compilation, serialization         | ~2019 |
| `ferrous-systems`   | publication | [Ferrous Systems blog](https://ferrous-systems.com/blog/)                                                                                           | Embedded and safety-critical Rust, compiler internals                       | 2018  |
| `lib-rs`            | publication | [lib.rs](https://lib.rs/)                                                                                                                           | Quality-ranked crate index                                                  | 2018  |
| `matklad`           | person      | [Alex Kladov: matklad](https://matklad.github.io/)                                                                                                  | Project structure, `xtask`, testing, compiler and IDE architecture          | 2017  |
| `niko-matsakis`     | person      | [Niko Matsakis: Baby Steps](https://smallcultfollowing.com/babysteps/)                                                                              | Borrow checker and trait solving, async design, language direction          | 2011  |
| `nnethercote`       | person      | [Nicholas Nethercote: The Rust Performance Book](https://nnethercote.github.io/perf-book/) and [blog](https://nnethercote.github.io/)               | Program and compiler performance, profiling, allocations, build times       | 2020  |
| `predrag`           | person      | [Predrag Gruevski: predr.ag](https://predr.ag/blog/)                                                                                                | SemVer and API compatibility, sealed traits, Cargo features                 | 2022  |
| `ralf-jung`         | person      | [Ralf Jung: Ralf's Ramblings](https://www.ralfj.de/blog/)                                                                                           | `unsafe` and undefined behaviour, Miri, aliasing and the memory model       | 2015  |
| `raph-levien`       | person      | [Raph Levien](https://raphlinus.github.io/) and posts on the [Linebender blog](https://linebender.org/blog/)                                        | 2D graphics and GPU compute, UI architecture, SIMD                          | 2017  |
| `rust-blog`         | publication | [Rust Blog](https://blog.rust-lang.org/) and [Inside Rust](https://blog.rust-lang.org/inside-rust/)                                                 | Release announcements, language and compiler status, security advisories    | 2014  |
| `rust-docs`         | publication | [Rust documentation](https://doc.rust-lang.org/): the Book, Reference, Rustonomicon, Edition Guide, Cargo Book, Clippy, and `std`                   | Official documentation: language, Cargo, lints, and the standard library    | 2015  |
| `rust-patterns`     | publication | [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)                                                                                 | Idioms, design patterns, and anti-patterns                                  | 2015  |
| `sunshowers`        | person      | [Rain: sunshowers.io](https://sunshowers.io/)                                                                                                       | Async cancellation, testing and `cargo-nextest`, Unix process behaviour     | 2020  |
| `this-week-in-rust` | publication | [This Week in Rust](https://this-week-in-rust.org/)                                                                                                 | Weekly newsletter aggregation                                               | 2013  |
| `tokio-blog`        | publication | [Tokio blog](https://tokio.rs/blog) and [tutorial](https://tokio.rs/tokio/tutorial)                                                                 | The Tokio runtime, axum, tracing, runtime tooling                           | 2017  |
| `tweede-golf`       | publication | [Tweede golf blog](https://tweedegolf.nl/en/blog)                                                                                                   | Embedded and async Rust, memory-safe infrastructure, compiler work          | 2019  |
| `yoshua-wuyts`      | person      | [Yoshua Wuyts](https://blog.yoshuawuyts.com/)                                                                                                       | Async and effects design, iterators, structured concurrency, WebAssembly    | 2016  |

## Watch list: track record still forming, or previously strong but now dormant

A watch-list source is never cited. `just validate-sources` rejects a watch-list ID in any `sources:` list, however good the back catalogue. Use these sources for discovery and cross-checking: follow a claim you find here to a citable source, and cite that. `vet-source` re-evaluates them when a blocker might have cleared.

| id                   | Kind        | Source                                                                                                                  | Focus                                                     | Since |
| -------------------- | ----------- | ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- | ----- |
| `api-guidelines`     | publication | [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)                                                      | Library API design conventions                            | 2014  |
| `effective-rust`     | person      | [David Drysdale: Effective Rust](https://www.lurklurk.org/effective-rust/)                                              | Idiomatic Rust: types, traits, dependencies, tooling, FFI | 2021  |
| `jon-gjengset`       | person      | [Jon Gjengset: Rust for Rustaceans](https://rust-for-rustaceans.com/)                                                   | Intermediate and advanced Rust: API design, `unsafe`      | 2016  |
| `luca-palmieri`      | person      | [Luca Palmieri](https://lpalmieri.com/posts/) and posts on the [Mainmatter blog](https://mainmatter.com/blog/tag/rust/) | Web backends, error handling, testing, telemetry          | 2020  |
| `mara-bos`           | person      | [Mara Bos](https://blog.m-ou.se/) and [Rust Atomics and Locks](https://mara.nl/atomics/)                                | Atomics and memory ordering, `std` internals              | 2020  |
| `ms-rust-guidelines` | publication | [Microsoft: Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/)                                    | Coding standards for application and library teams        | 2025  |
| `sabrina-jewson`     | person      | [Sabrina Jewson](https://sabrinajewson.org/blog)                                                                        | Lifetimes, GATs, error design, async destructors          | 2022  |
| `without-boats`      | person      | [withoutboats: without.boats](https://without.boats/blog/)                                                              | Async and await design, `Pin`, generators                 | 2017  |

## Opted out

A source moves here when its owner asks to stop being cited. Its row keeps the same five columns, and its section under [Source notes](#source-notes) goes. The ID stays reserved, so a later suggestion of the source is turned away, and `just validate-sources` rejects it in any `sources:` list. An opt-out from any awesome-humans ecosystem covers this one too, so `vet-source` reads every other ecosystem's Opted out section before admitting anyone. Opting back in is a suggestion like any other, and the source leaves this section only on admission. For the policy, see [Want in or out?](https://awesome-humans.net/want-in-or-out/)

No source has opted out.

## Source notes

Each section holds the evidence behind a row: the track record, what a citation can rest on, and any conflict to declare. Each source has at most one section, sorted by ID. A source with nothing to record has none.

Write each section in this shape:

1. **Verdict:** Open with the verdict in a line or two. For a watch-listed source, say what blocks it. A short note stops here.
2. **Labelled bullets:** A longer note continues with one `**Label:**` bullet per criterion or rule that has something to record, in this order, leaving out empty labels:
   1. **Scope**
   2. **Longevity** or **Dormancy**
   3. **Depth**
   4. **Accuracy**
   5. **Authorship**
   6. **Independence**
   7. **Conflict of interest**
   8. **Cite** for a citable source, or **Use** for a watch-listed one
   9. **Re-evaluate**
3. **One sentence per line:** Put each sentence on its own line, indented under its bullet, so a reworded sentence shows in a diff as that one sentence.

The check reads markings from the whole section, heading included, so `**Discovery-only.**` or `**Corroborate.**` in either place marks the source.

### `alice-ryhl`: Citable

Citable on async and Tokio design, and on the Rust for Linux abstractions, from a Tokio maintainer.

- **Longevity:** Rust writing since [2020-08-15](https://ryhl.io/blog/temporary-shared-mutation/), at one to three posts a year.
  The longest gap in the last three years is about eight months, from 2023-11-01 to 2024-07-11.
- **Depth:** ["Actors with Tokio"](https://ryhl.io/blog/actors-with-tokio/) works through handles, backpressure, and shutdown.
  ["Async: What is blocking?"](https://ryhl.io/blog/async-what-is-blocking/) gives a measurable rule for how long a task may run between `.await` points.
- **Cite:** The async and Tokio posts for practice.
  The 2024 to 2025 posts are RFCs and design write-ups for the Rust project, such as `#[derive(SmartPointer)]` and in-place initialization, so cite them for design, never as stable Rust.
- **Re-evaluate:** The latest post is from 2025-11-05.
  If nothing follows by 2026-11-05, check whether the design writing continues elsewhere before demoting.

### `anssi-rust`: Citable

Citable for `security.md`: a government security agency's rule-and-rationale guide, maintained since 2018.

- **Longevity:** First commit 2018-12-13, with content changes through 2026-04-07.
- **Depth:** Each rule states a requirement and its rationale, with examples, but no measurements.
- **Cite:** Rules by their ID.
  The guide calls itself unstable and names no Rust version or edition, so check a rule against `rust-docs` before an opinion rests on it.

### `api-guidelines`: Watch list

Dormant: the last change beyond a fixed link is from 2024-03, and nothing since 2025-07-08.

- **Dormancy:** The guidelines call themselves "still in development", and nothing reflects the 2021 or 2024 edition.
- **Use:** As a checklist for discovery. Cite the rule's current treatment in `rust-docs` or a citable source instead.
- **Re-evaluate:** When the guidelines change again, or when `ms-rust-guidelines`, which builds on them, qualifies.

### `awesome-rust`: Citable, **Discovery-only.**

A curated list of crates and tools since 2014-07-17, still taking additions in 2026-10.
Its [contribution rules](https://github.com/rust-unofficial/awesome-rust/blob/main/CONTRIBUTING.md) gate entries on popularity, such as 50 GitHub stars or 2,000 downloads, not on review.

### `blessed-rs`: Citable, **Discovery-only.**

A hand-curated guide to default crates, maintained by Nico Burns since 2022-03-13, with updates through 2026-08-03.
It makes editorial calls between crates, so follow each to the crate's own material and a roster source.

### `burntsushi`: Citable

Citable on error handling, regex and text search, and datetime design, under the dormancy exception.

- **Dormancy:** The blog's latest technical post is from 2023-07-05.
  The writing continues in the Jiff documentation: [`COMPARE.md`](https://github.com/BurntSushi/jiff/blob/master/COMPARE.md) gained new benchmarks on 2026-01-05 and an updated comparison with `time` on 2026-02-06, both written by Andrew Gallant.
- **Depth:** ["Regex engine internals as a library"](https://burntsushi.net/regex-internals/) is a guided tour of the `regex-automata` design.
  ["Error Handling in Rust"](https://burntsushi.net/rust-error-handling/) and ["Using unwrap() in Rust is Okay"](https://burntsushi.net/unwrap/) reason from invariants to rules.
- **Conflict of interest:** Author of `regex`, `ripgrep`, and `jiff`.
  Never cite Andrew Gallant alone on adopting `jiff` over `chrono` or `time`.
- **Cite:** "Error Handling in Rust" dates from 2015, before `?` and the error crates in use today, so cite it for the reasoning, not its code.
- **Re-evaluate:** If the Jiff documentation goes a year without new writing.

### `corrode`: Citable

Citable on practical Rust, with a consultancy conflict on whether to adopt Rust at all.

- **Scope:** Covers both corrode.dev and Matthias Endler's personal blog, endler.dev, where the Rust writing goes back to 2017.
- **Depth:** ["Tips for Faster Rust Compile Times"](https://endler.dev/2020/rust-compile-times/) is practical and measured.
  The corrode posts on standard library sharp edges and defensive programming give rules with code.
- **Conflict of interest:** corrode is a Rust consultancy that sells migration, training, and mentoring.
  Several posts argue for adopting Rust, so never cite it alone on that question.
  Endler also maintains `lychee`, the link checker this repository uses.
- **Cite:** The technical posts.
  Most endler.dev posts since 2026 are on careers and craft, which are out of scope.

### `effective-rust`: Watch list

Dormant: the book came out in print in 2024-04, and no later writing was found.

- **Dormancy:** No edition covers Rust 2024, and the author publishes nowhere else on Rust.
- **Use:** Its 35 items for discovery and cross-checking.
- **Re-evaluate:** On a revised edition or new writing.

### `fasterthanlime`: Citable

Citable on long-form technical articles, text only.

- **Scope:** Articles only. The videos are out of scope.
  Some articles start as sponsor early access, so cite only an article that's public.
- **Longevity:** Articles since 2013, with Rust the main subject since about 2019.
  The latest is from 2025-12-31, which keeps the source inside the dormancy line until 2026-12-31.
- **Conflict of interest:** Funded by sponsors, and articles promote Amos Wenger's own crates, such as `facet`.
  Never cite the source alone on adopting one of them.
- **Re-evaluate:** If no public article appears by 2026-12-31.

### `ferrous-systems`: Citable

Citable on engineering posts, with a vendor limit.

- **Depth:** ["Query cycles: A compiler murder mystery"](https://ferrous-systems.com/blog/query-cycles-a-compiler-murder-mystery/) traces a hung build to three interacting query-cycle bugs.
- **Independence:** Ferrous Systems sells Ferrocene, a qualified toolchain, with training and consulting.
  About half of the 2025 to 2026 posts are product announcements, events, and hiring, and engineering posts end with sales links.
- **Cite:** Engineering posts for how something works.
  Never cite it on adopting Ferrocene, or on whether a project needs a qualified toolchain.

### `jon-gjengset`: Watch list

Blocked on scope: the written Rust work is one book, from 2021, and the rest is video.

- **Dormancy:** The blog's last Rust post is from 2017-05-10.
- **Accuracy:** The book's [errata page](https://rust-for-rustaceans.com/) lists 37 corrections, the latest reported 2026-01-04, which is a strong record.
- **Use:** The book for discovery and cross-checking.
- **Re-evaluate:** On a second edition or new written work.

### `lib-rs`: Citable, **Discovery-only.**

An alternative crates.io front end, announced 2018-06-04 and maintained by one person.
It ranks crates by reverse dependencies, maintenance, and documentation, [as its About page describes](https://lib.rs/about), which helps find candidates but isn't a recommendation.

### `luca-palmieri`: Watch list

Dormant: the latest post is from 2025-02.

- **Dormancy:** Neither lpalmieri.com nor the Mainmatter blog has a post by Luca Palmieri since 2025-02.
  The Pavex repository's 2025 to 2026 changes are maintenance, not new documentation.
- **Use:** _Zero To Production in Rust_ and the error handling posts, for discovery.
- **Re-evaluate:** When the writing resumes.

### `mara-bos`: Watch list

Dormant: the latest blog post is from 2023-12-05.

- **Dormancy:** _Rust Atomics and Locks_ (2023-01) is the latest long-form writing, and no later writing was found.
- **Use:** The book, free to read online, for discovery and cross-checking on atomics and memory ordering.
- **Re-evaluate:** When the writing resumes.

### `matklad`: Citable

Citable on project structure, `xtask`, testing, and software design.

- **Longevity:** About 240 posts from 2017-03-12 to 2026-10-07.
- **Cite:** The Rust posts and the general design posts that apply to Rust.
  Most posts since 2024 are on Zig or general craft, so an opinion on a Rust detail needs a Rust post.

### `ms-rust-guidelines`: Watch list

Short on longevity: the repository's first commit is from 2025-08-11.

- **Use:** For discovery. Version 2026.6, released 2026-09-25, adds guidance on macros, performance, FFI, and agent-assisted coding.
- **Re-evaluate:** After 2027-08-11.

### `niko-matsakis`: Citable

Citable on language design and direction.

- **Longevity:** Since 2011-12-02, with 22 posts in 2025 and the latest on 2026-10-14.
- **Depth:** Design reasoning on the borrow checker, trait solving, and async.
- **Cite:** For how the language works and where it's going.
  Most posts are proposals, such as ["Battery packs"](https://smallcultfollowing.com/babysteps/blog/2026/07/15/battery-packs/), so never cite a proposal as current Rust.
  Use a "Coming next" note for a proposal.

### `nnethercote`: Citable

Citable on performance, with measurements in both the book and the blog.

- **Scope:** Covers The Rust Performance Book, first published in 2020-11, and Nicholas Nethercote's blog.
- **Longevity:** The book has updates through 2026-04-23, and the blog runs from 2021 to 2026-09-30.
  The longest gap in the last three years is about nine months, from 2024-03-06 to 2024-12-19.
- **Depth:** The [Heap Allocations](https://nnethercote.github.io/perf-book/heap-allocations.html) chapter works from DHAT output to a rule of thumb.
  ["How much code does that proc macro generate?"](https://nnethercote.github.io/2025/06/26/how-much-code-does-that-proc-macro-generate.html) measures the code `serde`'s derive generates.

### `predrag`: Citable

Citable on SemVer and API compatibility, with a conflict on `cargo-semver-checks`.

- **Longevity:** Rust writing since 2022-02-03, and the latest post is from 2026-08-15.
- **Depth:** "A definitive guide to sealed traits in Rust" and the posts on how Cargo features break compatibility are practical and exact.
- **Conflict of interest:** Author of `cargo-semver-checks`, which many posts are built around.
  Never cite Predrag Gruevski alone on adopting it.
- **Cite:** Not the 2026-04-01 or other April Fools posts.

### `ralf-jung`: Citable

The reference on `unsafe` Rust semantics, undefined behaviour, and Miri.

- **Longevity:** Since 2015-10-08, and the latest post is from 2026-03-13.
  The longest gap in the last three years is about six and a half months, from 2023-06 to 2023-12.
- **Cite:** For `unsafe` code and its rules.
  Where a post describes a proposed model, such as Tree Borrows, say whether the Reference has adopted it.

### `raph-levien`: Citable

Citable on graphics, GPU compute, UI architecture, and SIMD, with the writing moved to the Linebender blog.

- **Dormancy:** The personal blog's latest post is from 2025-03-21.
  The writing continues on the Linebender blog, where Raph Levien wrote "A plan for SIMD" (2025-06-06), "Linebender in 2026 Q1" (2026-04-19), and a post on hyperbezier curves (2026-08-08).
- **Accuracy:** Publishes retrospectives on failed or replaced projects, such as the xi-editor retrospective.
- **Conflict of interest:** Leads the Linebender projects, including Xilem, Vello, and `kurbo`.
  Never cite Raph Levien alone on adopting one of them.
- **Cite:** Not the 2023-04-01 April Fools post, "Moving from Rust to C++".

### `rust-blog`: Citable

The Rust project's canonical record of what shipped in each release.

- **Cite:** Release announcements for what changed and when.
  Inside Rust posts on process and governance carry no technical claim.

### `rust-docs`: Citable

The Rust project's official documentation.

- **Accuracy:** The [Rustonomicon](https://doc.rust-lang.org/nomicon/) says it's incomplete in places, and that the Reference is more correct where the two disagree.
- **Cite:** The Reference before the Rustonomicon on language rules, and the Book for the idiom it teaches.
  Check the Book's stated Rust version before citing a version-specific claim.

### `rust-patterns`: Citable, **Corroborate.**

Marked for uneven depth: community-written, maintained, and shallow in places.

- **Longevity:** Since 2015-08-30, with a new pattern on 2025-12-14 and changes through 2026-09-21.
- **Depth:** Each entry lists trade-offs, but argues in prose without measurements.

### `sabrina-jewson`: Watch list

Dormant: the latest of six posts is from 2024-09-22.

- **Accuracy:** "Why the 'Null' Lifetime Does Not Exist" carries a correction, dated 2023-07-22, that credits the reader who found the mistake.
- **Re-evaluate:** When the writing resumes.

### `sunshowers`: Citable

Citable on async cancellation, testing, and process behaviour, with a conflict on `cargo-nextest`.

- **Longevity:** Since 2020-08-22, and the latest Rust post is from 2026-06-02.
  The longest gap between Rust posts in the last three years is about ten months, from 2023-10-09 to 2024-08-04.
- **Depth:** ["Cancelling async Rust"](https://sunshowers.io/posts/cancelling-async-rust/) separates cancel safety from cancel correctness, with mitigations for `select!` and Tokio's mutexes.
- **Accuracy:** "Cancelling async Rust" fixed a code sample on 2025-10-24, crediting the reader who found the bug.
- **Conflict of interest:** Rain writes `cargo-nextest`, and posts such as "Why nextest is process-per-test" argue for its design.
  Never cite them alone on adopting it.
  This repository's own checks run on `cargo-nextest`.
- **Cite:** The Rust posts. Many other posts are personal essays, which are out of scope.

### `this-week-in-rust`: Citable, **Discovery-only.**

A weekly newsletter since 2013-06-07, at issue 673 on 2026-10-14.

### `tokio-blog`: Citable

Citable on how Tokio works, with a project limit.

- **Depth:** ["Introducing dial9"](https://tokio.rs/blog/2026-03-18-dial9) works from production traces to scheduling delays and lock contention.
  The tutorial explains futures and the runtime in worked steps, but carries no date or Tokio version.
- **Independence:** A project's view of itself: since 2025, most posts announce TokioConf, Toasty, or Topcoat.
- **Cite:** How Tokio and axum work.
  Never cite it on whether to adopt Tokio or a crate from the same organisation.
  Check a tutorial claim against the current docs.rs pages, because the tutorial names no version.

### `tweede-golf`: Citable

Citable on engineering posts, with a vendor limit.

- **Longevity:** Rust posts since 2019-02-20, monthly through 2026-10-14.
- **Depth:** ["Async Rust never left the MVP state"](https://tweedegolf.nl/en/blog/237/async-rust-never-left-the-mvp-state/) measures async state machine sizes and proposes compiler fixes.
- **Independence:** A Rust consultancy that sells engineering, training, and consulting.
  Some posts are client case studies or ask for funding, and many end with a sales link.
- **Cite:** Engineering posts for how something works.
  Never cite it on adopting a product or service it sells, or alone on adopting Rust.

### `without-boats`: Watch list

Dormant: the latest post is from 2024-10-25.

- **Use:** The async and `Pin` design writing, for how those features came to be.
- **Re-evaluate:** When the writing resumes.

### `yoshua-wuyts`: Citable

Citable on async and effects design, from 177 posts since 2016.

- **Depth:** Mostly language design: effects, linear types, and in-place initialization.
  A minority is practical, such as "A Survey of Every Iterator Variant" and the `futures-concurrency` posts.
- **Conflict of interest:** Maintains `futures-concurrency`, and took part in the keyword generics initiative.
  Never cite Yoshua Wuyts alone on adopting `futures-concurrency`, or on a proposal of their own.
- **Cite:** For design and mechanism.
  Never cite a proposal as current Rust. Use a "Coming next" note.

## Decision log

<!-- prettier-ignore -->
| Date | Decision | Detail |
| --- | --- | --- |
| 2026-10-18 | Initial roster | First vetting pass over the 31 candidates in PLAN.md phase 3. No other ecosystem lists an opted-out source, so none was declined on that ground |
| 2026-10-18 | Admitted `alice-ryhl` | Rust writing since 2020, latest 2025-11-05 |
| 2026-10-18 | Admitted `anssi-rust` | Maintained since 2018-12, with content changes in 2026 |
| 2026-10-18 | Admitted `awesome-rust` (discovery-only) | Curated since 2014-07 |
| 2026-10-18 | Admitted `blessed-rs` (discovery-only) | Curated since 2022-03, updated 2026-08 |
| 2026-10-18 | Admitted `burntsushi` | Blog quiet since 2023-07. Admitted under the dormancy exception, on Jiff documentation written in 2026 |
| 2026-10-18 | Admitted `corrode` | Rust writing since 2017. Conflict recorded: a Rust consultancy |
| 2026-10-18 | Admitted `fasterthanlime` | Articles since 2013, latest 2025-12-31. Text only |
| 2026-10-18 | Admitted `ferrous-systems` | Publishing since 2018. Usage limit for its toolchain and services |
| 2026-10-18 | Admitted `lib-rs` (discovery-only) | Running since 2018 |
| 2026-10-18 | Admitted `matklad` | Writing since 2017, weekly in 2026 |
| 2026-10-18 | Admitted `niko-matsakis` | Writing since 2011. Proposals are never cited as current Rust |
| 2026-10-18 | Admitted `nnethercote` | The Rust Performance Book since 2020, blog through 2026-09. Proposed as `perf-book`, renamed because it's one person's work |
| 2026-10-18 | Admitted `predrag` | Rust writing since 2022. Conflict recorded: author of `cargo-semver-checks` |
| 2026-10-18 | Admitted `ralf-jung` | Writing since 2015, latest 2026-03 |
| 2026-10-18 | Admitted `raph-levien` | Writing since 2017, continuing on the Linebender blog through 2026-08 |
| 2026-10-18 | Admitted `rust-blog` | The Rust project's blog since 2014 |
| 2026-10-18 | Admitted `rust-docs` | The Rust project's documentation since 2015 |
| 2026-10-18 | Admitted `rust-patterns` (corroborate) | Maintained since 2015, uneven depth |
| 2026-10-18 | Admitted `sunshowers` | Writing since 2020, with a correction on record. Conflict recorded: author of `cargo-nextest` |
| 2026-10-18 | Admitted `this-week-in-rust` (discovery-only) | Weekly since 2013-06 |
| 2026-10-18 | Admitted `tokio-blog` | Publishing since 2017. Usage limit as the project's view of itself |
| 2026-10-18 | Admitted `tweede-golf` | Rust posts since 2019, monthly through 2026-10. Proposed for the watch list as dormant, but the checked page was out of date |
| 2026-10-18 | Admitted `yoshua-wuyts` | Writing since 2016. Proposed as Corroborate, but its proposals are deep, so it gets a usage limit instead |
| 2026-10-18 | Watch-listed `api-guidelines` | No content change since 2024-03 |
| 2026-10-18 | Watch-listed `effective-rust` | No writing since the 2024-04 print edition |
| 2026-10-18 | Watch-listed `jon-gjengset` | One 2021 book, with the rest of the output on video |
| 2026-10-18 | Watch-listed `luca-palmieri` | No writing since 2025-02 |
| 2026-10-18 | Watch-listed `mara-bos` | No writing since 2023-12 |
| 2026-10-18 | Watch-listed `ms-rust-guidelines` | First published 2025-08, short of two years |
| 2026-10-18 | Watch-listed `sabrina-jewson` | No writing since 2024-09 |
| 2026-10-18 | Watch-listed `without-boats` | No writing since 2024-10 |
