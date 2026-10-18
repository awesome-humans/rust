//! This repository's own CI checks, and the staging of its awesome-humans.net pages, as one
//! binary with a subcommand each (AGENTS.md: Scripts). Run them from the repository root
//! through `just`, or as `cargo xtask <name>`.
//!
//! Exit codes: 0 clean, 1 findings printed, 2 not run from the repository root or given
//! arguments it doesn't recognise.

mod build_site;
mod comment_header;
mod export_diagrams;
mod frontmatter;
mod house;
mod repo;
mod validate_metadata;
mod validate_readme_index;
mod validate_sources;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use crate::repo::Repo;

#[derive(Parser)]
#[command(about, disable_version_flag = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check the metadata on every opinion, research topic, and template
    ValidateMetadata,
    /// Check the roster, and every source that opinions and templates cite against it
    ValidateSources,
    /// Check that README.md indexes every opinion and skill, in order
    ValidateReadmeIndex,
    /// Draw the SVG diagrams in assets/diagrams/
    ExportDiagrams,
    /// Stage the /rust/ pages of awesome-humans.net in <dir>/rust/
    BuildSite {
        /// The staging directory, as the awesome-humans/website staging contract defines
        #[arg(long, value_name = "dir")]
        out: PathBuf,
    },
}

/// What a check found: the failure heading and its findings, or the summary when it's clean.
pub struct Report {
    pub failure: &'static str,
    pub errors: Vec<String>,
    pub summary: String,
}

impl Report {
    fn print(&self) -> ExitCode {
        if self.errors.is_empty() {
            println!("{}", self.summary);
            return ExitCode::SUCCESS;
        }

        println!("{}", self.failure);
        for error in &self.errors {
            println!("  - {error}");
        }
        ExitCode::FAILURE
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if !Repo::is_root(Path::new(".")) {
        eprintln!(
            "No AGENTS.md and xtask/ here. Run this from the repository root, as `just` does."
        );
        return ExitCode::from(2);
    }

    let repo = Repo::new(".");
    match cli.command {
        Command::ValidateMetadata => validate_metadata::run(&repo).print(),
        Command::ValidateSources => validate_sources::run(&repo).print(),
        Command::ValidateReadmeIndex => validate_readme_index::run(&repo).print(),
        Command::ExportDiagrams => match export_diagrams::run(&repo) {
            Ok(()) => {
                println!("Exported to {}/", export_diagrams::OUTPUT);
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("Couldn't write {}/: {error}", export_diagrams::OUTPUT);
                ExitCode::FAILURE
            }
        },
        Command::BuildSite { out } => {
            let errors = build_site::run(&repo, &out);
            if errors.is_empty() {
                println!("Staged to {}/", out.join(build_site::ECOSYSTEM).display());
                return ExitCode::SUCCESS;
            }

            for error in errors {
                eprintln!("{error}");
            }
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use crate::repo::Repo;

    /// A throwaway repository for a test, built file by file.
    pub struct Fixture {
        dir: TempDir,
    }

    impl Fixture {
        pub fn new() -> Self {
            let dir = tempfile::tempdir().expect("temporary directory");
            Self { dir }
        }

        pub fn file(self, path: &str, content: &str) -> Self {
            let full = self.dir.path().join(path);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).expect("fixture directory");
            }
            fs::write(full, content).expect("fixture file");
            self
        }

        pub fn dir(self, path: &str) -> Self {
            fs::create_dir_all(self.dir.path().join(path)).expect("fixture directory");
            self
        }

        pub fn repo(&self) -> Repo {
            Repo::new(self.dir.path())
        }

        pub fn path(&self) -> &std::path::Path {
            self.dir.path()
        }
    }
}
