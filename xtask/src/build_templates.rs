//! Builds the files under `templates/` the way a consumer who copied them would. The templates
//! are a workspace with no members, so `cargo` run in `templates/` stops before it reads a
//! lint or a rustfmt setting, and a template that the toolchain rejects stays unnoticed until
//! someone copies it.
//!
//! This writes the templates into `<out>/<profile>/`, once for the default copy (every house
//! region removed, as `none`) and once for each house profile's, so a template is also built
//! without its regions. In each copy it adds one member with `cargo new --lib`, which joins
//! the workspace and inherits its edition and lints, and then runs the pipeline the opinions
//! prescribe: build, test, Clippy with warnings denied, and the rustfmt check. A warning from
//! rustfmt fails the check too, because rustfmt exits 0 after ignoring a setting.
//!
//! A copy with a `rust-toolchain.toml` of its own builds on the toolchain that file names.
//! Any other copy builds on the toolchain this check runs with, which the repository pins to
//! the release the templates target.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use crate::Report;
use crate::house;
use crate::repo::{self, Repo};

/// The default profile, which has no house opinions. Its copy goes under `<out>/none/`.
const NO_PROFILE: &str = "none";

/// Where the added member goes, inside a copy.
const MEMBER: &str = "crates/example";

/// The files rustup reads a directory's toolchain from.
const TOOLCHAIN_FILES: &[&str] = &["rust-toolchain.toml", "rust-toolchain"];

/// One cargo command of the pipeline: its arguments, and how it treats warnings.
struct Stage {
    arguments: &'static [&'static str],
    /// Sets Cargo's `build.warnings` to `deny`, for rustc and Clippy lints.
    deny_warnings: bool,
    /// Fails on a line of standard error that starts with this, for a tool with no such setting.
    rejected: Option<&'static str>,
}

/// The member first, then the stages in the order a failure is cheapest to read: a template
/// Cargo can't parse fails at once, and a lint or a rustfmt setting only after the code builds.
const STAGES: &[Stage] = &[
    // Without `--vcs none`, cargo would start a git repository in a copy written outside one.
    Stage {
        arguments: &["new", "--lib", "--vcs", "none", MEMBER],
        deny_warnings: false,
        rejected: None,
    },
    Stage {
        arguments: &["build"],
        deny_warnings: false,
        rejected: None,
    },
    Stage {
        arguments: &["test"],
        deny_warnings: false,
        rejected: None,
    },
    // Cargo's `build.warnings`, as opinions/rust.md prescribes for CI. It also turns a lint
    // name that Clippy doesn't know into a failure, because that's reported as a warning.
    Stage {
        arguments: &["clippy", "--all-targets"],
        deny_warnings: true,
        rejected: None,
    },
    // rustfmt only warns about a key it doesn't know, or one that needs nightly, and then
    // formats without it and exits 0. Either means a setting in rustfmt.toml does nothing.
    Stage {
        arguments: &["fmt", "--check"],
        deny_warnings: false,
        rejected: Some("Warning: "),
    },
];

/// Assembles each profile's copy under `out` and runs the pipeline on it, stopping at the
/// first stage that fails.
pub fn run(repo: &Repo, out: &Path) -> Report {
    let mut report = Report {
        failure: "Template build failed:",
        errors: Vec::new(),
        summary: String::new(),
    };

    let templates = repo.template_files();
    if templates.is_empty() {
        report
            .errors
            .push(format!("{}/: no templates to build", repo::TEMPLATES));
        return report;
    }

    let profiles = profiles(repo, &templates, &mut report.errors);
    if !report.errors.is_empty() {
        return report;
    }

    for profile in &profiles {
        let root = out.join(profile);
        if let Err(error) = assemble(repo, &templates, profile, &root) {
            report
                .errors
                .push(format!("{}: can't be written: {error}", root.display()));
            return report;
        }

        println!("== {profile}: assembled in {}", root.display());
        for stage in STAGES {
            let command = format!("cargo {}", stage.arguments.join(" "));
            println!("== {profile}: {command}");
            if let Err(reason) = cargo(&root, stage) {
                report.errors.push(format!(
                    "{command} {reason} for the {profile} copy, in {}",
                    root.display()
                ));
                return report;
            }
        }
    }

    report.summary = format!(
        "The templates build, test, and pass Clippy and rustfmt as {} {}: {}.",
        profiles.len(),
        if profiles.len() == 1 {
            "copy"
        } else {
            "copies"
        },
        profiles.join(", ")
    );
    report
}

/// The default profile, then every profile with a region in any template, sorted.
fn profiles(repo: &Repo, templates: &[String], errors: &mut Vec<String>) -> Vec<String> {
    let mut found = BTreeSet::new();
    for path in templates {
        match repo.read(path) {
            Ok(text) => found.extend(house::profiles(path, &text, errors)),
            Err(error) => errors.push(format!("{path}: can't be read: {error}")),
        }
    }

    std::iter::once(NO_PROFILE.to_string())
        .chain(found)
        .collect()
}

/// Writes one profile's copy of every template into `root`, replacing what was there, so a
/// file a template no longer ships, or the last run's member, can't pass for this run's.
fn assemble(repo: &Repo, templates: &[String], profile: &str, root: &Path) -> io::Result<()> {
    if root.exists() {
        fs::remove_dir_all(root)?;
    }

    let kept = (profile != NO_PROFILE).then_some(profile);
    for path in templates {
        let relative = path
            .strip_prefix(repo::TEMPLATES)
            .unwrap_or(path)
            .trim_start_matches('/');
        let destination = root.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(
            destination,
            house::for_profile(path, &repo.read(path)?, kept),
        )?;
    }

    Ok(())
}

/// Runs one stage in a copy, and says how it failed.
fn cargo(root: &Path, stage: &Stage) -> Result<(), String> {
    let mut command = Command::new("cargo");
    command.args(stage.arguments).current_dir(root);
    if stage.deny_warnings {
        command.env("CARGO_BUILD_WARNINGS", "deny");
    }

    // rustup exports the toolchain it chose for this check, and that outranks a toolchain
    // file. Dropping it lets a copy that ships one build on the toolchain the file names.
    if TOOLCHAIN_FILES.iter().any(|name| root.join(name).is_file()) {
        command.env_remove("RUSTUP_TOOLCHAIN");
    }

    let Some(rejected) = stage.rejected else {
        return match command.status() {
            Ok(status) => exit(status),
            Err(error) => Err(format!("couldn't start: {error}")),
        };
    };

    // Standard error is read here and passed on, so the lines still reach the log.
    let output = command
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("couldn't start: {error}"))?;
    let messages = String::from_utf8_lossy(&output.stderr);
    eprint!("{messages}");
    exit(output.status)?;
    match warning(&messages, rejected) {
        Some(line) => Err(format!("printed \"{line}\"")),
        None => Ok(()),
    }
}

/// How a command that didn't succeed ended.
fn exit(status: ExitStatus) -> Result<(), String> {
    if status.success() {
        return Ok(());
    }

    Err(status.code().map_or_else(
        || "was stopped by a signal".to_string(),
        |code| format!("exited with code {code}"),
    ))
}

/// The first line of a tool's messages that starts with the rejected prefix.
fn warning<'a>(messages: &'a str, rejected: &str) -> Option<&'a str> {
    messages
        .lines()
        .map(str::trim_end)
        .find(|line| line.starts_with(rejected))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::tests::Fixture;

    const CARGO: &str = "# awesome-humans/rust template | sources: house:alice, rust-docs\n[workspace]\nmembers = []\n# house:alice begin\n# **House:** Alice's.\n[workspace.lints.rust]\nunsafe_code = \"forbid\"\n# house:alice end\n";
    const CONFIG: &str = "# awesome-humans/rust template | sources: house:bob, rust-docs\n[build]\n# house:bob begin\n# **House:** Bob's.\nwarnings = \"deny\"\n# house:bob end\n";

    fn fixture() -> Fixture {
        Fixture::new()
            .file("templates/Cargo.toml", CARGO)
            .file("templates/.cargo/config.toml", CONFIG)
    }

    #[test]
    fn profiles_lists_the_default_then_each_profile_in_any_template() {
        let fixture = fixture();
        let repo = fixture.repo();
        let mut errors = Vec::new();
        assert_eq!(
            profiles(&repo, &repo.template_files(), &mut errors),
            ["none", "alice", "bob"]
        );
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn profiles_reports_a_region_that_never_closes() {
        let fixture =
            Fixture::new().file("templates/Cargo.toml", "# header\n# house:alice begin\n");
        let repo = fixture.repo();
        let mut errors = Vec::new();
        profiles(&repo, &repo.template_files(), &mut errors);
        assert_eq!(errors, ["templates/Cargo.toml:2: house:alice never closes"]);
    }

    #[test]
    fn assemble_writes_the_default_copy_without_any_region() {
        let fixture = fixture();
        let repo = fixture.repo();
        let root = fixture.path().join("out/none");
        assemble(&repo, &repo.template_files(), "none", &root).expect("assembled");
        assert_eq!(
            fs::read_to_string(root.join("Cargo.toml")).expect("copy"),
            "# awesome-humans/rust template | sources: rust-docs\n[workspace]\nmembers = []\n"
        );
        assert_eq!(
            fs::read_to_string(root.join(".cargo/config.toml")).expect("copy"),
            "# awesome-humans/rust template | sources: rust-docs\n[build]\n"
        );
    }

    #[test]
    fn assemble_keeps_only_the_profiles_regions() {
        let fixture = fixture();
        let repo = fixture.repo();
        let root = fixture.path().join("out/bob");
        assemble(&repo, &repo.template_files(), "bob", &root).expect("assembled");
        assert_eq!(
            fs::read_to_string(root.join("Cargo.toml")).expect("copy"),
            "# awesome-humans/rust template | sources: rust-docs\n[workspace]\nmembers = []\n"
        );
        assert_eq!(
            fs::read_to_string(root.join(".cargo/config.toml")).expect("copy"),
            "# awesome-humans/rust template | sources: house:bob, rust-docs\n[build]\n# **House:** Bob's.\nwarnings = \"deny\"\n"
        );
    }

    #[test]
    fn assemble_replaces_what_the_last_run_left() {
        let fixture = fixture().file("out/none/crates/example/Cargo.toml", "[package]\n");
        let repo = fixture.repo();
        let root = fixture.path().join("out/none");
        assemble(&repo, &repo.template_files(), "none", &root).expect("assembled");
        assert!(!root.join("crates").exists());
    }

    #[test]
    fn warning_finds_the_setting_rustfmt_ignored() {
        let messages = "Warning: Unknown configuration option `not_a_setting`\nDiff in lib.rs\n";
        assert_eq!(
            warning(messages, "Warning: "),
            Some("Warning: Unknown configuration option `not_a_setting`")
        );
        assert_eq!(
            warning("Diff in lib.rs: a Warning: here\n", "Warning: "),
            None
        );
    }

    #[test]
    fn run_reports_a_repository_without_templates() {
        let fixture = Fixture::new();
        let report = run(&fixture.repo(), &fixture.path().join("out"));
        assert_eq!(report.errors, ["templates/: no templates to build"]);
    }
}
