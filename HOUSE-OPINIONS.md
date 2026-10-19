# House opinions

House opinions are preferences from experience, not from a vetted source. Each belongs to a **house profile**, a named set of house opinions owned by a person or an organisation. They're **always visibly marked**, so a reader can tell community best practice from one owner's convention.

awesome-humans.net shows no house opinions by default, so every page is sourced community practice. A reader chooses a profile in the header to see its house opinions beside the sourced text, and an agent reads the profile's `llms-<profile>.txt` instead of `llms.txt`.

## Profiles

| Profile         | Owner                                 | Kind   |
| --------------- | ------------------------------------- | ------ |
| `simon-mckenna` | Simon McKenna, the repository's owner | person |

The site registers every profile in [awesome-humans/website: `houses.toml`](https://github.com/awesome-humans/website/blob/main/houses.toml), and its build fails on a house opinion whose profile isn't registered there for this ecosystem. A new profile needs the repository owner's approval, recorded there. `simon-mckenna` is registered for `dotnet` only, so add `rust` to its `ecosystems` before weaving its first Rust opinion.

## How this works

Every awesome-humans ecosystem weaves house opinions the same way, as [awesome-humans/website: House opinions](https://github.com/awesome-humans/website/blob/main/pages/conventions/house-opinions.md) defines. It gives the [entry format](https://github.com/awesome-humans/website/blob/main/pages/conventions/house-opinions.md#entry-format) for the [Inbox](#inbox), how `weave-house-opinion` writes an entry in beside the sourced text, the fixed `**House:**` marking, and how a contributor's opinion is adopted. It's the canonical statement, so read it before you add or weave an entry, and work from it, not a paraphrase.

No house opinion carries over from the .NET repository. Each one comes back through the Inbox if it applies to Rust.

## Inbox

_Nothing waiting._

## Woven

| Date | Profile | Opinion | Woven into | Provenance |
| ---- | ------- | ------- | ---------- | ---------- |
