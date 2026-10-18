# House opinions

House opinions are preferences from experience, not from a vetted source. Each belongs to a **house profile**, a named set of house opinions owned by a person or an organisation. They're **always visibly marked**, so a reader can tell community best practice from one owner's convention.

awesome-humans.net shows no house opinions by default, so every page is sourced community practice. A reader chooses a profile in the header to see its house opinions beside the sourced text, and an agent reads the profile's `llms-<profile>.txt` instead of `llms.txt`.

## Profiles

| Profile         | Owner                                 | Kind   |
| --------------- | ------------------------------------- | ------ |
| `simon-mckenna` | Simon McKenna, the repository's owner | person |

The site registers every profile in [awesome-humans/website: `houses.toml`](https://github.com/awesome-humans/website/blob/main/houses.toml), and its build fails on a house opinion whose profile isn't registered there for this ecosystem. A new profile needs the repository owner's approval, recorded there. `simon-mckenna` is registered for `dotnet` only, so add `rust` to its `ecosystems` before weaving its first Rust opinion.

## How this works

1. **The profile's owner adds an entry to the [Inbox](#inbox),** naming the profile. A sentence or two is enough, as long as it gives the reasoning.
2. **The `weave-house-opinion` skill writes it in, beside the sourced text it changes.** It never rewrites the sourced text.
   - **In `opinions/`:** a block after the sourced recommendation it changes, which opens with `/// house | <profile>` and closes with `///`, each on its own line.
   - **In `templates/`:** a region between two comment lines, `house:<profile> begin` and `house:<profile> end`, in the file's comment syntax, such as `# house:simon-mckenna begin` in TOML or `// house:simon-mckenna begin` in Rust.
   - **In `sources:`:** `house:<profile>`, sorted like any other ID.

   The inbox entry then moves to [Woven](#woven), with a link.

3. **The marking is a fixed string.** Every house opinion starts with exactly `**House:**`, bold with the colon inside, so `grep -F '**House:**'` finds it. A `house:<profile>` ID and that profile's blocks or regions always appear together, `**House:**` appears only inside one, and `cargo xtask validate-sources` fails the pull request otherwise.
4. **Keep both, and the profile decides which a reader sees.** This is the canonical statement, and other files link here. The sourced text stays whole, and the house block sits beside it. A reader or agent with the profile sees both, and the house opinion wins where they conflict. Everyone else sees only the sourced text. Harvest, refresh, and audit runs never remove, weaken, or unmark a house block, and never cut sourced text because a house block disagrees with it.
5. **Other contributors propose opinions in a pull request,** as the organisation's [opinion pull request template](https://github.com/awesome-humans/.github/blob/main/PULL_REQUEST_TEMPLATE/opinion.md) describes. When a profile's owner accepts an experience-based opinion, they adopt it into their profile, and the Woven table's Provenance column credits the contributor and the pull request.

The `house:` prefix is reserved and documented in `AWESOME-HUMANS.md`. A profile skips the admission criteria because its authority is its owner's experience, which is why it's marked wherever it's used.

```markdown
/// house | simon-mckenna

**House:** Treat warnings as errors on every machine, not only in CI. …

///
```

No house opinion carries over from the .NET repository. Each one comes back through the Inbox if it applies to Rust.

## Entry format

```markdown
### <short title>

- **Profile:** <the house profile, such as simon-mckenna>
- **Opinion:** <what to do, one sentence>
- **Why:** <the experience or reasoning behind it>
- **Scope:** <which topic/file it belongs to, if known>
```

## Inbox

_Nothing waiting._

## Woven

| Date | Profile | Opinion | Woven into | Provenance |
| ---- | ------- | ------- | ---------- | ---------- |
