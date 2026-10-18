//! Reads the repository the checks run against. Every path a check reports is relative to
//! the repository root, with forward slashes, so a finding reads the same on any platform.
//!
//! Unlike the .NET scripts, a missing `opinions/`, `templates/`, or `skills/` directory isn't
//! a sign of the wrong working directory: this repository gains them phase by phase
//! (PLAN.md), so an absent directory reads as empty, and the root is recognised by
//! `AGENTS.md` and `xtask/` instead.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

pub const OPINIONS: &str = "opinions";
pub const RESEARCH: &str = "research";
pub const TEMPLATES: &str = "templates";
pub const SKILLS: &str = "skills";

/// Whether a path has the extension, in any case.
pub fn has_extension(path: &str, extension: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|found| found.eq_ignore_ascii_case(extension))
}

/// Build output that `just verify` leaves under `templates/`, gitignored and never a resource.
const BUILD_OUTPUT: &[&str] = &["target"];

pub struct Repo {
    root: PathBuf,
}

impl Repo {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Whether `root` is this repository's root, which holds `AGENTS.md` and the `xtask` crate.
    pub fn is_root(root: &Path) -> bool {
        root.join("AGENTS.md").is_file() && root.join("xtask/Cargo.toml").is_file()
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    pub fn is_dir(&self, relative: &str) -> bool {
        self.path(relative).is_dir()
    }

    pub fn is_file(&self, relative: &str) -> bool {
        self.path(relative).is_file()
    }

    pub fn exists(&self, relative: &str) -> bool {
        self.path(relative).exists()
    }

    /// A text file with its line endings normalised to LF, as every check reads it.
    pub fn read(&self, relative: &str) -> io::Result<String> {
        Ok(fs::read_to_string(self.path(relative))?.replace("\r\n", "\n"))
    }

    /// The `.md` files directly in `directory`, sorted, or none when it's absent.
    pub fn markdown_in(&self, directory: &str) -> Vec<String> {
        let Ok(entries) = fs::read_dir(self.path(directory)) else {
            return Vec::new();
        };

        let mut paths: Vec<String> = entries
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_file())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| has_extension(name, "md"))
            .map(|name| format!("{directory}/{name}"))
            .collect();
        paths.sort();
        paths
    }

    /// The opinion files, as `opinions/<name>.md`, sorted.
    pub fn opinions(&self) -> Vec<String> {
        self.markdown_in(OPINIONS)
    }

    /// The directories under `skills/`, sorted.
    pub fn skills(&self) -> Vec<String> {
        let Ok(entries) = fs::read_dir(self.path(SKILLS)) else {
            return Vec::new();
        };

        let mut names: Vec<String> = entries
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect();
        names.sort();
        names
    }

    /// Every file under `templates/`, build output aside, in ordinal order.
    pub fn template_files(&self) -> Vec<String> {
        let base = self.path(TEMPLATES);
        if !base.is_dir() {
            return Vec::new();
        }

        let mut paths: Vec<String> = WalkDir::new(&base)
            .into_iter()
            .filter_entry(|entry| {
                entry.depth() == 0
                    || !(entry.file_type().is_dir()
                        && BUILD_OUTPUT.iter().any(|name| entry.file_name() == *name))
            })
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter_map(|entry| {
                let relative = entry.path().strip_prefix(&base).ok()?;
                let parts: Vec<&str> = relative
                    .components()
                    .map(|part| part.as_os_str().to_str())
                    .collect::<Option<_>>()?;
                Some(format!("{TEMPLATES}/{}", parts.join("/")))
            })
            .collect();
        paths.sort();
        paths
    }

    /// The `templates/` files that carry a comment header: every one but JSON, which has no
    /// comment syntax (AGENTS.md: Metadata).
    pub fn header_templates(&self) -> Vec<String> {
        self.template_files()
            .into_iter()
            .filter(|path| !has_extension(path, "json"))
            .collect()
    }
}
