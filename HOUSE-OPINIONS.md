# House opinions

These are the repository owner's opinions: preferences from experience, not from a vetted source. They carry the same weight as sourced opinions, and they're **always visibly marked**, so a reader can tell community best practice from this repository's convention.

## How this works

1. **The owner adds an entry to the [Inbox](#inbox).** A sentence or two is enough, as long as it gives the reasoning.
2. **The `weave-house-opinion` skill writes it in.** The opinion goes into the right `opinions/` file, and into `templates/` where it applies, with `house` in the file's `sources:`. The inbox entry then moves to [Woven](#woven), with a link.
3. **The marking is a fixed string.** Every house opinion starts with exactly `**House:**`, bold with the colon inside, so `grep -F '**House:**'` finds it. The `house` ID and the marking always appear together, and one without the other is an audit finding.
4. **A house opinion wins a conflict with the roster.** This is the canonical statement, and other files link here. Keep the house opinion, and cut the sourced position to a one-line note that cites the source: "The community default is X (source); we do Y because Z." Harvest, refresh, and audit runs never remove, weaken, or unmark house content.
5. **Other contributors propose opinions in a pull request,** as the organisation's [opinion pull request template](https://github.com/awesome-humans/.github/blob/main/PULL_REQUEST_TEMPLATE/opinion.md) describes. When the owner accepts an experience-based opinion, the owner adopts it as a house opinion, and the Woven table's Provenance column credits the contributor and the pull request.

The `house` source ID is reserved and documented in `AWESOME-HUMANS.md`. It skips the admission criteria because its authority is ownership of the repository, which is why it's marked wherever it's used.

No house opinion carries over from the .NET repository. Each one comes back through the Inbox if it applies to Rust.

## Entry format

```markdown
### <short title>

- **Opinion:** <what to do, one sentence>
- **Why:** <the experience or reasoning behind it>
- **Scope:** <which topic/file it belongs to, if known>
```

## Inbox

_Nothing waiting._

## Woven

| Date | Opinion | Woven into | Provenance |
| ---- | ------- | ---------- | ---------- |
