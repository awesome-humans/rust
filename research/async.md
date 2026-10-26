---
targets: [rust-1.99, edition-2024]
last-reviewed: 2026-10-19
sources:
  [
    alice-ryhl,
    corrode,
    rust-blog,
    rust-docs,
    sunshowers,
    tokio-blog,
    tweede-golf,
    yoshua-wuyts,
  ]
---

# Async: Tokio, blocking, cancellation, and structured concurrency

Use async for I/O-bound concurrency only, run it on Tokio, and hold every task to these rules:

- **Keep domain logic synchronous,** and confine async to I/O and the services you call.
- **Never block a runtime thread.** Yield to the runtime at least every 10 to 100 µs. An `.await` yields only when its future is pending, which Tokio's own channels, sockets, and timers force once a task spends its budget, but an always-ready future from elsewhere never does. Send blocking I/O to `spawn_blocking`, CPU-bound work to rayon, and a loop that runs forever to its own thread.
- **Give every spawned task an owner.** Use a `JoinSet` for tasks whose results you collect, and a `TaskTracker` with a `CancellationToken` for tasks that must finish before shutdown, an actor included. `tokio-util` still needs a roster source to carry it (see [the source gap](#tasks-and-structured-concurrency)). Never drop a `JoinHandle` to detach a task that no owner tracks.
- **Treat every `.await` as a cancellation point.** In a `select!` loop, use only cancel-safe futures, or pin the future outside the loop and poll it on every iteration, never awaiting anything else in a branch body while it's pending. Spawn a cancel-unsafe operation on the `TaskTracker`, not a `JoinSet`, so it runs to completion when its caller is cancelled.
- **Hold no lock across an `.await`.** Guard short critical sections with `std::sync::Mutex`, and give a shared I/O resource to one task behind a bounded channel instead of a `tokio::sync::Mutex`.
- **Bound every channel and queue.**

Every API recommended here is stable on Rust 1.99.1 with `tokio` 1.53.3 (an LTS release until September 2027) and `tokio-util` 0.7.20. Async closures (1.85) are the newest async language feature. Everything in [Coming next](#coming-next) is unstable on nightly as of 2026-10-19.

**No opinion file covers this.** `opinions/rust.md` mentions async only in its features table (async closures) and its Coming next list (`async fn` in `dyn` traits). This is the `async.md` topic in [PLAN.md: Phase 6](../PLAN.md#phase-6-first-opinions-and-templates).

## Whether to use async, and which runtime

**Async for concurrency, threads for parallelism.** The Book's rule of thumb: CPU-bound, "very parallelizable" work suits threads, and I/O-bound, "very concurrent" work suits async, and "in many cases, the choice isn't threads _or_ async but rather threads _and_ async" ([The Rust Programming Language: Futures, Tasks, and Threads](https://doc.rust-lang.org/book/ch17-06-futures-tasks-threads.html), read 2026-10-19). The chapter runs its examples on the Book's own `trpl` crate, so it teaches the model, not a runtime.

**Async only where it pays.** `corrode`: "use async Rust only when you really need it", and "In binary crates, think twice if you really need to use async. It's probably easier to just spawn a thread and get away with blocking I/O." It adds: isolate async code so domain logic stays synchronous, and "In your public library code, avoid async-only interfaces" ([Matthias Endler: The State of Async Rust: Runtimes](https://corrode.dev/blog/async/), 2024-02-21, updated 2026-07-30).

**Tokio.** The same post: "Tokio stands as Rust's canonical async runtime", and "for now, Tokio stands out as the pragmatic choice for real-world implementations". `corrode`'s conflict covers adopting Rust, not a runtime, so it can carry this. `tokio-blog` can't: its notes forbid citing it on adopting Tokio. The alternatives lost:

- **`async-std`** is discontinued. Its README says "`async-std` has been discontinued; use `smol` instead", and the repository's last push was 2025-08-15 ([async-rs/async-std](https://github.com/async-rs/async-std), a reference).
- **`smol`** is "A small async runtime, which is easy to understand" (`corrode`, above), but has a fraction of Tokio's ecosystem. Embedded targets use their own executors, which are outside this topic.

**Versions.** Tokio's README lists the LTS lines `1.51.x` until March 2027 and `1.53.x` until September 2027, both with an MSRV of 1.71 ([tokio-rs/tokio: README](https://github.com/tokio-rs/tokio#lts-releases), a reference, read 2026-10-19). No roster source recommends pinning to an LTS line, so the opinion should state the versions it was checked against and leave the pinning policy out. Releases: `tokio` 1.53.3 on 2026-10-13, then 1.54.1 on 2026-10-21 (crates.io, read 2026-10-26), and `tokio-util` 0.7.20 on 2026-10-09, with an MSRV of 1.85 (crates.io).

## Blocking

`alice-ryhl`, a Tokio maintainer, gives the rule: "Async code should never spend a long time without reaching an `.await`", and "a good rule of thumb is no more than 10 to 100 microseconds between each `.await`. That said, this depends on the kind of application you are writing" ([Alice Ryhl: Async: What is blocking?](https://ryhl.io/blog/async-what-is-blocking/), 2020-12-21). An `.await` gives the thread back only when its future returns `Poll::Pending`. A future that's already ready continues in the same poll (`[expr.await.effects]` in [The Rust Reference: Await expressions](https://doc.rust-lang.org/reference/expressions/await-expr.html)), so a loop over futures that are always ready can block as surely as a loop with no `.await`. Tokio narrows the risk with a per-task budget: "Each Tokio resource (socket, timer, channel, ...) is aware of this budget", and once the task is out of budget, they all return not ready until it yields to the scheduler ([Tokio: Reducing tail latencies with automatic cooperative task yielding](https://tokio.rs/blog/2020-04-preemption), `tokio-blog`, 2020-04-01, checked 2026-10-26 against [`tokio::task::coop`](https://docs.rs/tokio/latest/tokio/task/coop/index.html) for `tokio` 1.54.1, where `poll_proceed` "returns Poll::Pending if the budget is depleted"). A loop that awaits a Tokio channel or socket therefore yields on its own. The loops to check await something outside Tokio that is always ready, such as `std::future::ready` or a cache hit. The rule measures time between yields, not between `.await` keywords. Its cheat sheet becomes this table, with the Tokio docs' caveats added:

| Work                                        | Use                           | Why                                                                                                                                                                                                                           |
| ------------------------------------------- | ----------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Blocking I/O: files, a synchronous SDK      | `tokio::task::spawn_blocking` | A pool with "an upper limit of around 500 threads" (`alice-ryhl`). A started task "cannot be aborted", and runtime shutdown waits for it ([`spawn_blocking`](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)) |
| CPU-bound computation                       | rayon, and a `oneshot` back   | `spawn_blocking` is "poorly suited for expensive CPU-bound computations" (`alice-ryhl`). The Tokio docs agree: "Specialized CPU-bound executors, such as rayon, may also be a good fit"                                       |
| A loop that runs forever                    | `std::thread::spawn`          | "If a blocking operation keeps running forever, you should run it on a dedicated thread" (`alice-ryhl`)                                                                                                                       |
| A short `std::sync::Mutex` critical section | Nothing                       | Not blocking in practice. See [Shared state](#shared-state)                                                                                                                                                                   |

No roster source covers `block_in_place`. Its docs say it panics on a `current_thread` runtime, and suspends any other future the same task is running, such as a `join!` sibling ([`block_in_place`](https://docs.rs/tokio/latest/tokio/task/fn.block_in_place.html), a reference). A reference can't carry a rule, so the opinion should leave it out until a roster source covers it. To call async code from a synchronous program, build a `Runtime` and call `block_on`, as the [Tokio topic page on bridging with sync code](https://tokio.rs/tokio/topics/bridging) shows (`tokio-blog`, checked 2026-10-26 against [`Runtime::block_on`](https://docs.rs/tokio/latest/tokio/runtime/struct.Runtime.html#method.block_on) for `tokio` 1.54.1).

## Tasks and structured concurrency

**The principle.** `yoshua-wuyts` defines structured concurrency as a program whose structure "is guaranteed to be a tree regardless of how much concurrency is going on internally", which gives three properties: cancellation propagates to children, errors propagate to the caller, and "When a function returns, you know it is done doing work". A task with no parent is "a _dangling task_" ([Yoshua Wuyts: Tree-Structured Concurrency](https://blog.yoshuawuyts.com/tree-structured-concurrency/index), 2023-07-01). Tokio's `spawn` isn't structured: "A `JoinHandle` _detaches_ the associated task when it is dropped" ([`JoinHandle`](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html), a reference).

**What gives a task an owner on stable Tokio:**

| Owner                                                      | When                                                                        | On drop                                                                                                                                                                                                  |
| ---------------------------------------------------------- | --------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `tokio::join!`, `try_join!`                                | A fixed set of futures in one task, which can borrow                        | Every branch is cancelled with the parent                                                                                                                                                                |
| `tokio::task::JoinSet`                                     | A dynamic set of tasks whose outputs you collect                            | "all tasks in the `JoinSet` are immediately aborted" ([`JoinSet`](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html))                                                                          |
| `tokio_util::task::TaskTracker` with a `CancellationToken` | Tasks that must finish, such as request handlers, before shutdown completes | "dropping a `TaskTracker` does not abort the tasks" ([`TaskTracker`](https://docs.rs/tokio-util/latest/tokio_util/task/task_tracker/struct.TaskTracker.html)). `wait` returns once it's closed and empty |

**Graceful shutdown** has three parts in the Tokio guide: detect the signal, usually with `tokio::signal::ctrl_c`, tell tasks with a `CancellationToken`, and wait with a `TaskTracker`, calling `close` and then `wait` ([Tokio: Graceful Shutdown](https://tokio.rs/tokio/topics/shutdown), `tokio-blog`, which names no version, checked against the docs.rs pages for `tokio-util` 0.7.20). A child token from `child_token` is cancelled with its parent, and `run_until_cancelled` returns `None` once the token is cancelled ([`CancellationToken`](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html), a reference).

**Source gap: `tokio-util`.** `tokio-util` is a tokio-rs crate, and `tokio-blog`'s notes forbid citing it on adopting a crate from the same organisation, so the guide can explain how `TaskTracker` and `CancellationToken` work but can't carry the recommendation to use them. No other roster source does: `alice-ryhl`'s blocking post, `corrode`'s runtime post, and `sunshowers`' cancellation post don't mention `tokio-util` (checked 2026-10-26). `sunshowers` lists "Cooperative cancellation: cancellation tokens" as a pattern, which carries the token idea but not the crate. A Tokio-only form needs no crate beyond Tokio, which `corrode` carries: a `JoinSet` drained on shutdown, and a `tokio::sync::watch` channel as the token. That design is the editor's. `resolve-research` either finds a roster source for `tokio-util` or promotes that form instead.

```rust
// Cargo.toml: tokio 1.53 with "macros", "rt-multi-thread", "signal", "sync", "time";
// tokio-util 0.7.20 with "rt".
use std::path::PathBuf;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

#[tokio::main]
async fn main() {
    let token = CancellationToken::new();
    let tracker = TaskTracker::new();
    let (tx, mut rx) = mpsc::channel::<PathBuf>(64);

    tracker.spawn({
        let token = token.clone();
        async move {
            while let Some(Some(path)) = token.run_until_cancelled(rx.recv()).await {
                let len =
                    tokio::task::spawn_blocking(move || std::fs::metadata(path).map(|m| m.len()))
                        .await
                        .expect("the closure doesn't panic");
                match len {
                    Ok(len) => println!("{len} bytes"),
                    Err(err) => eprintln!("{err}"),
                }
            }
        }
    });

    for path in ["Cargo.toml", "src/main.rs"] {
        if tx.send(PathBuf::from(path)).await.is_err() {
            break;
        }
    }

    tokio::signal::ctrl_c()
        .await
        .expect("can listen for Ctrl-C");
    token.cancel();
    tracker.close();
    if tokio::time::timeout(Duration::from_secs(10), tracker.wait())
        .await
        .is_err()
    {
        eprintln!("tasks still running after 10 s");
    }
}
```

The editor compiled this on 1.99.1 with `tokio` 1.53.3 and 1.54.1 and `tokio-util` 0.7.20, and it passes `cargo clippy -- -D warnings` with `clippy::pedantic` on and `cargo fmt --check` with `templates/rustfmt.toml` (2026-10-26). The blocking `std::fs` call goes to `spawn_blocking`, as the table above says. `rx.recv()` is cancel-safe, so cancelling it mid-wait loses no message.

**Actors for owned state.** `alice-ryhl`'s actor pattern gives state to one task and talks to it through a cloneable handle. "use a bounded channel so that the number of messages waiting in the channel don't grow without bound", and when all senders drop, `recv` returns `None` and the actor shuts down on its own. Avoid a cycle made only of bounded channels, which can deadlock ([Alice Ryhl: Actors with Tokio](https://ryhl.io/blog/actors-with-tokio/), 2021-02-13). The post starts the actor with `tokio::spawn` and drops the `JoinHandle`. Under the ownership rule, spawn it on the `TaskTracker` instead, so shutdown waits for the actor to drain once its senders drop. That reconciliation is the editor's.

## Cancellation

**Two properties.** `sunshowers` separates them: "cancel safety is a local property of an individual future", the ability to be dropped without side effects, while cancel correctness is a property of the whole system. A bug needs a cancel-unsafe future that is actually cancelled and a system property that breaks. Select loops "are particularly prone to cancel correctness issues", but `select!` isn't the only source: `try_join!` cancels its siblings when one fails, and `let _ = fut;` drops a future without polling it ([Rain: Cancelling async Rust](https://sunshowers.io/posts/cancelling-async-rust/), 2025-10-03, corrected 2025-10-24).

**Its mitigations,** each of which an opinion can apply as a rule:

1. **Avoid Tokio mutexes.** A guard held across an `.await` usually leaves an invariant broken when the future is dropped.
2. **Make APIs cancel-safe.** Reserve a channel slot with `reserve` before building the message, and write with `write_all_buf` and a cursor, not `write_all`.
3. **Drive cancel-unsafe futures to completion.** At Oxide, "a task for each HTTP request" fixed a cancellation bug, because dropping a `JoinHandle` doesn't cancel the task. Spawn it on the `TaskTracker`, which doesn't abort on drop. A `JoinSet` won't do: dropping it aborts every task in it.
4. **Pin a future outside a `select!` loop,** and poll `&mut fut`, so a cancelled branch keeps its place. Poll it again on every iteration, and don't `.await` anything else in a branch body while it's pending. Otherwise it can hold a resource, such as a Tokio mutex's turn, that the body waits on, and the task deadlocks (see the unvetted futurelock RFD under [Watch-list and unvetted material](#watch-list-and-unvetted-material)).

**Which Tokio futures are cancel-safe** comes from the [`select!` docs](https://docs.rs/tokio/latest/tokio/macro.select.html), with `JoinSet::join_next` from the [`JoinSet` docs](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html#method.join_next) ("This method is cancel safe") and `&mut JoinHandle` from the [`JoinHandle` docs](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html) ("Awaiting a `&mut JoinHandle<T>` is cancel safe"), all references:

| Cancel-safe                                                                                                                                                                                                      | Not cancel-safe                                                                                                                                                                                 |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `mpsc::Receiver::recv`, `broadcast::Receiver::recv`, `watch::Receiver::changed`, `TcpListener::accept`, `AsyncReadExt::read`, `AsyncWriteExt::write`, `StreamExt::next`, `JoinSet::join_next`, `&mut JoinHandle` | `read_exact`, `read_to_end`, `read_to_string`, and `write_all` lose data. `Mutex::lock`, `RwLock::read` and `write`, `Semaphore::acquire`, and `Notify::notified` lose their place in the queue |

`select!` picks a branch at random by default, for fairness, and a loop marked `biased;` polls top to bottom (the `select!` docs). The editor reads that as "put the shutdown branch first under `biased;`, where a stream that's always ready can't starve it", but no roster source says so, and the opinion should leave it out until one does. `tokio::time::timeout` cancels its future by dropping it, and the same rules apply.

## Shared state

The Tokio docs answer the mutex question directly: "Contrary to popular belief, it is ok and often preferred to use the ordinary Mutex from the standard library in asynchronous code." The async mutex's only advantage is being held across an `.await`, and for a shared I/O resource "it is often better to spawn a task to manage the IO resource, and to use message passing" ([`tokio::sync::Mutex`](https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html), a reference). The tutorial says the same, and warns that a `MutexGuard` from a crate that implements `Send` for it compiles across an `.await` and deadlocks ([Tokio: Shared state](https://tokio.rs/tokio/tutorial/shared-state), `tokio-blog`). No docs.rs page restates the deadlock (checked 2026-10-26 for `tokio` 1.54.1), but it depends on no Tokio version: a `Send` guard makes the future `Send`, so `tokio::spawn` accepts it, and Clippy's `await_holding_lock` catches the case for `std` and `parking_lot` guards. `sunshowers` and `alice-ryhl` both point to the actor pattern instead. Wrap the `Arc<Mutex<T>>` in a type with synchronous methods, so no caller can hold the guard across an `.await`.

**Channels.** "Unbounded queues will eventually fill up all available memory and cause the system to fail in unpredictable ways" ([Tokio: Channels](https://tokio.rs/tokio/tutorial/channels), `tokio-blog`, checked 2026-10-26 against [`tokio::sync::mpsc`](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html) for `tokio` 1.54.1, whose unbounded channel works "without backpressure"). Use `mpsc::channel(capacity)`, `oneshot` for a reply, `broadcast` for fan-out, and `watch` for the latest value. `recv` on `std::sync::mpsc` or crossbeam blocks the thread until a message arrives, so `alice-ryhl`'s blocking rule rules it out on a runtime thread. The `mpsc` docs agree: "use the kind of channel that matches where the receiver is".

## Lints

Clippy enforces several of the rules above: lock and borrow guards held across an `.await`, futures dropped without being polled, and future size. Task ownership, bounded queues, and where blocking work runs have no lint. Groups, levels, and versions are from [Clippy Lints](https://clippy.rust-lang.org/stable/index.html), 1.99, read 2026-10-19, and the editor confirmed each level with a probe on 1.99.1.

| Lint                         | Group       | Level | Added | Catches                                                                                           |
| ---------------------------- | ----------- | ----- | ----- | ------------------------------------------------------------------------------------------------- |
| `async_yields_async`         | correctness | deny  | 1.48  | An `async` block whose output is itself a future that was never awaited                           |
| `await_holding_lock`         | suspicious  | warn  | 1.45  | A `std::sync` or `parking_lot` `MutexGuard` held across an `.await`                               |
| `await_holding_refcell_ref`  | suspicious  | warn  | 1.49  | A `RefCell` borrow held across an `.await`                                                        |
| `await_holding_invalid_type` | suspicious  | warn  | 1.62  | The types listed in `await-holding-invalid-types` in `clippy.toml`, such as a project's own guard |
| `let_underscore_future`      | suspicious  | warn  | 1.67  | `let _ = fut;`, which drops a future without polling it                                           |
| `manual_async_fn`            | style       | warn  | 1.45  | `fn f() -> impl Future { async { … } }` that could be an `async fn`                               |
| `redundant_async_block`      | complexity  | warn  | 1.70  | `async { fut.await }`                                                                             |
| `large_futures`              | pedantic    | allow | 1.70  | A future over `future-size-threshold`, 16384 bytes by default. The fix is `Box::pin`              |
| `unused_async`               | pedantic    | allow | 1.54  | An `async fn` with no `.await`                                                                    |
| `future_not_send`            | nursery     | allow | 1.44  | An `async fn` whose future isn't `Send`, for a library that must run on a multi-threaded runtime  |

Rustc adds `async_fn_in_trait` (warn), covered below. A workspace that warns on `clippy::pedantic` gets `large_futures` and `unused_async` with the group. Future size matters beyond the stack: `tweede-golf` measured duplicated states in the compiler's async state machines, and on stable the fix is to write each `.await` once, hoisting a shared call out of the `match` arms ([Tweede golf: Async Rust never left the MVP state](https://tweedegolf.nl/en/blog/237/async-rust-never-left-the-mvp-state), 2026-05-04). Its compiler changes are a 2026 project goal, not stable Rust.

## Async in traits and closures

Each row is the first stable release, from `rust-blog` and the release notes.

| Feature                                 | Release | Use                                                                                                                                                                                                                                                                                       |
| --------------------------------------- | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `IntoFuture`                            | 1.64    | `.await` on a builder without a `send()` call                                                                                                                                                                                                                                             |
| `std::pin::pin!`                        | 1.68    | Pin a future on the stack for a `select!` loop, instead of `Box::pin` or `tokio::pin!`                                                                                                                                                                                                    |
| `async fn` and RPITIT in traits         | 1.75    | Replace `#[async_trait]` for static dispatch. In a public trait, rustc's `async_fn_in_trait` lint warns because callers can't require `Send`. Use `#[trait_variant::make(SendName: Send)]` for a `Send` twin of the local trait, or write `fn f(&self) -> impl Future<Output = T> + Send` |
| Recursive `async fn` behind indirection | 1.77    | `Box::pin(recurse()).await`, with no helper crate                                                                                                                                                                                                                                         |
| Async closures and `AsyncFn*` bounds    | 1.85    | `async \|x\| …` and `F: AsyncFn(T) -> U`, which can borrow from their arguments, over `\|\| async move { … }` and `Fn() -> impl Future`                                                                                                                                                   |

The trait guidance is from [Rust Blog: Announcing `async fn` and return-position `impl Trait` in traits](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/) (2023-12-21): "We recommend using the `trait_variant::make` proc macro to let your users choose", and these traits "are not object-safe". `trait-variant` is at 0.1.3, from the rust-lang organisation. A trait that needs `dyn` still uses `async-trait` (0.1.92), until the `async fn` in `dyn` traits row in Coming next reaches stable. Releases 1.86 to 1.99 brought no new async feature, only `Pin` soundness fixes (1.92, 1.96, 1.97, and 1.99 in [RELEASES.md](https://github.com/rust-lang/rust/blob/stable/RELEASES.md)).

## Where the sources disagree

- **Spawning versus in-task concurrency.** `yoshua-wuyts` argues that tasks are "a poor fit for non-parallel concurrent execution", and prefers joining futures in one task ([Yoshua Wuyts: Tasks are the wrong abstraction](https://blog.yoshuawuyts.com/tasks-are-the-wrong-abstraction), 2024-04-27). `sunshowers` spawns tasks precisely because a task survives its caller's cancellation. They agree once read as: join futures inside a task when they belong to it, and spawn a task on a `TaskTracker` when it must outlive a cancellation. A `JoinSet` can't do this, because dropping it aborts its tasks. That reading is the editor's. `yoshua-wuyts` maintains `futures-concurrency`, and its notes forbid citing it alone on adopting that crate, so the opinion should recommend `tokio::join!` and `JoinSet`, not the crate.
- **`select!` over `FuturesUnordered`.** The unvetted Oxide RFD below recommends `JoinSet` over `FuturesUnordered` in a loop that awaits anything else. No roster source contradicts it, and `JoinSet` is already the recommendation.
- **CPU-bound work in a task.** Watch-list `ms-rust-guidelines` says a long computation "should contain `yield_now().await` points" (M-YIELD-POINTS). `alice-ryhl` and the Tokio docs move the work off the runtime instead. The weight is with them: one is citable and from a maintainer, and the other is watch-list. Yielding keeps the work on a worker thread, where it still competes with I/O.

## Coming next

Unstable on nightly as of 2026-10-19, not yet an opinion. Statuses are from the tracking issues and [Rust Project Goals: 2026](https://goals.rust-lang.org/2026/goals.html), which are references.

| Feature                     | Status, 2026-10-19                                                                                                                                                                                                                               |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `async fn` in `dyn` traits  | Experimental ([rust-lang/rust#133119](https://github.com/rust-lang/rust/issues/133119)). A 2026 goal, "Native async fn dynamic dispatch in traits", awaits an RFC decision                                                                       |
| Return type notation        | RFC approved, implementation incomplete ([rust-lang/rust#109417](https://github.com/rust-lang/rust/issues/109417)). A 2026 goal prepares it for stabilisation, with `impl Trait` in type aliases. It's what lets a caller write `T::f(..): Send` |
| Async drop                  | Unstable codegen ([rust-lang/rust#126482](https://github.com/rust-lang/rust/issues/126482))                                                                                                                                                      |
| `AsyncIterator`             | Unstable ([rust-lang/rust#79024](https://github.com/rust-lang/rust/issues/79024)). Use `futures::Stream` (0.3.34) and `StreamExt` meanwhile                                                                                                      |
| Pin ergonomics (`&pin mut`) | Experimental. The 2026 goals list "Continue Experimentation with Pin Ergonomics". The `&pin` type syntax has parsed since 1.84                                                                                                                   |
| Async state machine size    | Two 2026 goals, one led by the `tweede-golf` post's author. Compiler work, with no new syntax                                                                                                                                                    |

Return type notation is the row that changes the trait guidance: once stable, a public trait can use bare `async fn`, and callers add `Send` where they need it.

## Watch-list and unvetted material

Kept apart from the roster findings above. None of it carried a claim.

- **Watch-list `ms-rust-guidelines`:** M-ASYNC-FN prefers `async fn` to a `Future`-returning signature, and M-ASYNC-STACK-SIZE asks hot `async fn`s to track their future sizes ([Microsoft: Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/)). Both agree with `manual_async_fn` and `large_futures`. M-YIELD-POINTS is the disagreement above.
- **Watch-list `without-boats`:** "FuturesUnordered and the order of futures" (2024-02-18) and "Asynchronous clean-up" (2024-02-24) explain intra-task concurrency and why async drop is hard. Both are design history, with no practice that the roster lacks.
- **Unvetted: [Oxide RFD 609: Futurelock](https://rfd.shared.oxide.computer/rfd/609)** (David Pacheco, undated on the page). It defines futurelock as a deadlock where one future holds a resource, such as a fair Tokio mutex's turn, that another needs, while the task has stopped polling the first. Triggers: a `&mut fut` branch in `select!` beside an `.await` in another branch, and a `FuturesUnordered` loop that awaits something else. Mitigation: "be extremely careful that the task never stops polling a future that it previously started polling", most simply by spawning the future and selecting on its `JoinHandle`. It sharpens `sunshowers`' mitigation 4: pinning outside the loop keeps the place, but the pinned future must be polled again.

## What this means for the repository

- **No opinion covers the topic.** Run `resolve-research` to create `opinions/async.md`. Its first sentence is the opening list here. It needs these sections: whether to use async and which runtime, blocking, task ownership and graceful shutdown with the example, cancellation with the cancel-safety table, shared state and channels, the lint table, and traits and closures. List it under README.md's Scope and in the layout tree.
- **Leave out what no roster source carries.** Pinning to a Tokio LTS line, avoiding `block_in_place`, and putting the shutdown branch first under `biased;` rest only on references or on the editor's reading. Promote them only if `resolve-research` finds a roster source.
- **Settle the `tokio-util` source gap first.** The opening rule recommends `TaskTracker` and `CancellationToken`, which only `tokio-blog` covers, and it can't carry that recommendation. Find a roster source, or promote the Tokio-only form that [the source gap](#tasks-and-structured-concurrency) describes, and rewrite the example to match.
- **`opinions/rust.md` can link it.** Its features-table row on async closures and its Coming next item on `async fn` in `dyn` traits should point to `async.md` once that exists, and the Coming next item can move there.
- **No template change.** `large_futures` and `unused_async` come with `clippy::pedantic`, which `templates/Cargo.toml` doesn't turn on, and `await-holding-invalid-types` needs a project's own types. A future `clippy.toml` template is the place for it, if one is added.
- **Gaps in this sweep.** The workers that were to read `niko-matsakis`'s async posts and `fasterthanlime`'s async articles stopped on a usage limit, and the editor didn't cover them. Neither source was expected to carry a claim: `niko-matsakis` writes proposals, which could only add to Coming next. `resolve-research` should check both before relying on that list.
- **`vet-source` candidate: Oxide RFDs** (`rfd.shared.oxide.computer`), for RFD 609 and the RFDs 397 and 400 that `sunshowers` recommends on cancellation. They're engineering design documents from a company that ships Rust in production. The vetting has to check their longevity and independence.
