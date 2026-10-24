# Every local and CI entry point (AGENTS.md: Scripts). CI calls these recipes, never cargo
# directly, so a check runs the same way on every machine. Run `just` to list them.

set shell := ["bash", "-euo", "pipefail", "-c"]

# List the recipes
default:
    @just --list

# Install the prose tools at the versions package-lock.json pins
install:
    npm ci

# Format Markdown, JSON, YAML, and Rust
format:
    npm run format
    cargo fmt --all

# Check formatting, Markdown lint, and spelling, for prose and Rust together
check: format-check lint spell fmt-check

# Check that Prettier leaves Markdown, JSON, JSONC, and YAML unchanged
format-check:
    npm run format:check

# Lint Markdown with markdownlint
lint:
    npm run lint

# Check spelling with cspell
spell:
    npm run spell

# Check Rust formatting
fmt-check:
    cargo fmt --all --check

# Lint the xtask crate, denying every warning, as CI does
clippy:
    CARGO_BUILD_WARNINGS=deny cargo clippy --workspace --all-targets --locked

# Run the xtask tests (needs cargo-nextest)
test:
    cargo nextest run --package xtask --locked

# Validate every skill against the Agent Skills specification (needs skills-ref from .github/requirements.txt)
validate-skills:
    #!/usr/bin/env bash
    set -euo pipefail
    # An unmatched glob would otherwise loop once over the literal pattern, so the
    # empty-directory check below could never fire.
    shopt -s nullglob
    if ! command -v agentskills > /dev/null; then
        echo "agentskills isn't installed: run uv tool install skills-ref --with-requirements .github/requirements.txt" >&2
        exit 1
    fi
    status=0
    found=0
    for dir in skills/*/; do
        found=1
        if ! agentskills validate "$dir"; then
            status=1
        fi
    done
    if [ "$found" -eq 0 ]; then
        echo "No skill directories found under skills/" >&2
        status=1
    fi
    exit "$status"

# Check the metadata on every opinion, research topic, and template
validate-metadata:
    cargo xtask validate-metadata

# Check the roster, and every source that opinions and templates cite against it
validate-sources:
    cargo xtask validate-sources

# Check that README.md indexes every opinion and skill, in order
validate-readme-index:
    cargo xtask validate-readme-index

# Check that each template's dependency and lint keys are sorted within their group
validate-template-order:
    cargo xtask validate-template-order

# Draw the SVG diagrams in assets/diagrams/
export-diagrams:
    cargo xtask export-diagrams

# Fail if the committed diagrams differ from what export-diagrams draws
check-diagrams: export-diagrams
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "$(git status --porcelain -- assets/diagrams)" ]; then
        git status --short -- assets/diagrams >&2
        echo "assets/diagrams/ is out of date: run just export-diagrams and commit the result" >&2
        exit 1
    fi

# Check every link in the repository with lychee, reusing results under a day old (needs lychee)
links:
    lychee --config lychee.toml --no-progress --cache --max-cache-age 1d .

# Stage the /rust/ pages of awesome-humans.net in <out>/rust/
build-site out="target/site":
    cargo xtask build-site --out "{{ out }}"

# Run every offline check CI runs. It redraws the diagrams, and fails until you commit any SVG it changes
verify: check clippy test validate-skills validate-metadata validate-sources validate-readme-index validate-template-order check-diagrams build-site
