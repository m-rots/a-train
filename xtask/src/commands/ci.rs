use crate::commands::{Section, XtaskCommand};
use clap::Parser;
use xshell::cmd;

#[derive(Parser)]
pub(crate) struct Ci;

fn cargo_clippy() -> xshell::Result<()> {
    let _s = Section::new("Clippy");
    cmd!("cargo clippy --workspace").run()
}

fn cargo_test() -> xshell::Result<()> {
    let _s = Section::new("Test");
    cmd!("cargo test --workspace").run()
}

impl XtaskCommand for Ci {
    fn run(&self) -> anyhow::Result<()> {
        cargo_clippy()?;
        cargo_test()?;

        Ok(())
    }
}
