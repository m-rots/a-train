use colored::Colorize;
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use std::time::Instant;

mod ci;
mod dist;
mod docker;

pub(crate) use ci::Ci;
pub(crate) use dist::Dist;
pub(crate) use docker::Docker;

pub(crate) trait XtaskCommand {
    fn run(&self) -> anyhow::Result<()>;
}

static GH_ACTIONS: Lazy<bool> = Lazy::new(|| std::env::var("GITHUB_ACTIONS").is_ok());

enum Platform {
    GitHubActions,
    Local,
}

impl Platform {
    fn current() -> Self {
        if *GH_ACTIONS {
            Self::GitHubActions
        } else {
            Self::Local
        }
    }
}

struct Section {
    name: String,
    start: Instant,
}

impl Section {
    fn new<S: Into<String>>(name: S) -> Self {
        let name = name.into();

        match Platform::current() {
            Platform::GitHubActions => println!("::group::{}", &name),
            Platform::Local => println!(
                "\n{} {}",
                "-->".bright_purple(),
                name.bright_purple().bold()
            ),
        }

        let start = Instant::now();
        Self { name, start }
    }
}

impl Drop for Section {
    fn drop(&mut self) {
        let info = format!("<-- {} ({:.2?})", self.name.bold(), self.start.elapsed());
        println!("\n{}", info.dimmed());

        match Platform::current() {
            Platform::GitHubActions => println!("::endgroup::"),
            Platform::Local => (),
        }
    }
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn dist_path() -> PathBuf {
    project_root().join("target").join("dist")
}

fn source_path(target: &str) -> PathBuf {
    let path = project_root()
        .join("target")
        .join(target)
        .join("release")
        .join("a-train");

    if target.contains("-windows-") {
        path.with_extension("exe")
    } else {
        path
    }
}
