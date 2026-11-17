# Metadata

Resources under `opinions/`, `templates/`, and `research/` each record `targets`, `last-reviewed`, and `sources`, in the syntax their file format allows.

**`opinions/` and `research/` use YAML frontmatter:**

```yaml
---
targets: [rust-1.99, edition-2024]
last-reviewed: 2026-10-14 # last time content was verified against sources
sources: [matklad, rust-docs] # ids from AWESOME-HUMANS.md, sorted
---
```

A `research/` topic has no status field. Promoting or discarding a topic deletes it, so a topic on disk is unresolved by definition, and a status could only ever read `open`. For details, see the `research-topic` and `resolve-research` skills.

**`templates/` use a first-line comment header.** A TOML or Rust file can't open with a `---` block and stay valid for the tools that read it. So the same fields go in a comment instead, separated by pipes after a fixed marker. TOML files, `rust-toolchain.toml` included, use `#`, and `.rs` files use `//`:

```toml
# awesome-humans/rust template | targets: rust-1.99, edition-2024 | last-reviewed: 2026-10-14 | sources: rust-docs
```

```rust
// awesome-humans/rust template | targets: rust-1.99, edition-2024 | last-reviewed: 2026-10-14 | sources: rust-docs
```

Rules:

- **Don't record when a resource was used.** A `last-used` field is rejected. Writing a date on every read left uncommitted edits in every checkout an agent read from, and they conflicted with real changes to the same line. To find pruning candidates, `audit-freshness` counts what refers to each opinion and template instead.
- **Update `last-reviewed` only after you verify the content against its sources.**
- **Use `sources` IDs that exist in `AWESOME-HUMANS.md`,** either in the roster tables or as a house profile's reserved `house:<profile>` ID.
  - CI reads the `**Discovery-only.**` and `**Corroborate.**` markings from a source's section under `## Source notes`, not from its table row.
  - In `opinions/` and `templates/`, the source must also be citable. A watch-list source or a `**Discovery-only.**` source is rejected.
  - A file whose citable sources are all marked `**Corroborate.**` is rejected too, because an unmarked source has to stand beside them.
  - `research/` is exempt: a topic can cite unvetted material if the text flags it. For details, see the `research-topic` skill.
- **Sort `sources` by ID in ordinal order, with `house:<profile>` sorted like any other ID.** Write `[burntsushi, house:simon-mckenna, matklad, rust-docs]`. The rule applies in all three directories and to both syntaxes.
- **Write dates in ISO 8601 (`YYYY-MM-DD`), in UTC.** Always use absolute dates, never relative ones. To get today's date, run `date -u +%F`. CI, GitHub, and the site build run in UTC, as [awesome-humans/website: Writing style](https://github.com/awesome-humans/website/blob/main/pages/conventions/writing-style.md) describes.
- **A JSON template is exempt from the header,** because JSON has no comment syntax. None exists today. If one is added, what it pins is audited against the latest releases instead. Building the template workspace leaves `templates/target/` behind, which is gitignored. That output isn't a resource, and the check skips it.
- **`skills/` are deliberately exempt.** The [Agent Skills specification](https://agentskills.io/specification) defines their frontmatter, they're procedures and not reference material, and nothing reads a date from them. Use `git log` to see when a skill last changed.

CI runs `just validate-metadata` against both syntaxes, and a missing field, a relative date, or an unsorted `sources` list fails the pull request.
