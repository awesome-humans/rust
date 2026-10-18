# Awesome Humans of Rust

An agentic take on [awesome-rust](https://github.com/rust-unofficial/awesome-rust): distilled, opinionated guidance, thanks to awesome humans. Instead of links, it distils what proven Rust writers recommend into guidance your agent can follow, each opinion citing its sources. It documents current best practice in Rust code, and updates it as Rust evolves.

The writers will be listed in `AWESOME-HUMANS.md`: people and publications with at least two years of accurate, independent writing on Rust.

## Usage

- **Point your agent at it.** Tell your coding agent to follow the opinions when it writes code:

  ```text
  Follow the conventions at awesome-humans.net when writing Rust code
  ```

  The opinions cover conventions, project layout, language usage, and crate choices.

- **Check an existing project.** Ask your coding agent to run `verify-project`:

  ```text
  Use the verify project skill at awesome-humans.net on <repo>
  ```

  To narrow the review, say so in the request, such as "focus on testing".

- **Start a project from the templates.** Once they're published, copy the files in `templates/` into a new project, trimming what you don't need.

- **Read the opinions.** Once they're published, browse them at [awesome-humans.net](https://awesome-humans.net/rust/) or on GitHub. Each page gives its recommendation, with the rationale and sources behind it.

## Scope

`opinions/` holds one file per topic. None is published yet.

## Freshness policy

Opinions target the latest stable Rust release and the latest edition, never an older toolchain. Nightly-only features appear only in "Coming next" notes. The maintenance skills move the repository to each new release.

Every resource records when it was last reviewed against its sources, and opinions and templates also record when an agent last used them. [AGENTS.md: Metadata](AGENTS.md#metadata) defines the fields. The `xtask` checks will enforce them.

## Awesome humans

Every opinion cites a vetted source: a person or a publication. A source is admitted on its track record: at least two years of sustained writing, with depth, accuracy, and an independent view. Videos, talks, and podcasts are out of scope for now, because an opinion cites text a reader can check. `AWESOME-HUMANS.md` will hold the roster and the criteria.

## House opinions

The repository owner outranks the roster. The owner's preferences come in through [HOUSE-OPINIONS.md](HOUSE-OPINIONS.md) and the `weave-house-opinion` skill, and they're marked wherever they appear, so you can tell community best practice from local convention. HOUSE-OPINIONS.md defines the marking and which side wins a conflict. Other contributors can propose sourced or experience-based opinions through the organisation's [opinion pull request template](https://github.com/awesome-humans/.github/blob/main/PULL_REQUEST_TEMPLATE/opinion.md).

## Repository layout

```text
├── README.md                 ← you are here
├── AGENTS.md                 ← instructions for AI agents working in this repo
├── HOUSE-OPINIONS.md         ← the owner's own opinions: intake and audit trail
└── PLAN.md                   ← the plan for setting up this repository, deleted when it's done
```

## Acknowledgement

Agents build and maintain this project, but it wouldn't exist without the people whose writing it draws on. Every opinion cites their work. Thank you to all of these awesome humans, and to everyone who supports them.

## Licence

See [LICENSE](LICENSE).
