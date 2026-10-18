# Every local and CI entry point (AGENTS.md: Scripts). CI calls these recipes, never cargo
# directly, so a check runs the same way on every machine. Run `just` to list them.

set shell := ["bash", "-euo", "pipefail", "-c"]

# List the recipes
default:
    @just --list

# Format Markdown, JSON, YAML, and Rust
format:
    npm run format
    cargo fmt --all

# Check prose formatting, lint, and spelling, and Rust formatting
check: fmt-check
    npm run check

# Check Rust formatting
fmt-check:
    cargo fmt --all --check

# Lint the xtask crate, denying every warning, as CI does
clippy:
    cargo clippy --workspace --all-targets --locked -- -D warnings

# Run the xtask tests
test:
    cargo nextest run --package xtask --locked

# Check the metadata on every opinion, research topic, and template
validate-metadata:
    cargo xtask validate-metadata

# Check the roster, and every source that opinions and templates cite against it
validate-sources:
    cargo xtask validate-sources

# Check that README.md indexes every opinion and skill, in order
validate-readme-index:
    cargo xtask validate-readme-index

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

# Stage the /rust/ pages of awesome-humans.net in <out>/rust/
build-site out="target/site":
    cargo xtask build-site --out {{ out }}

# Run everything CI checks, and redraw the diagrams in place, so commit any SVG it changes
verify: check clippy test validate-metadata validate-sources validate-readme-index export-diagrams build-site
