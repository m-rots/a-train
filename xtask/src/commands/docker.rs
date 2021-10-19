use crate::commands::XtaskCommand;
use anyhow::ensure;
use clap::Parser;
use std::process::Command;

/// Build the Docker image
#[derive(Parser)]
pub(crate) struct Docker {
    #[clap(subcommand)]
    cmd: SubCommand,
}

impl XtaskCommand for Docker {
    fn run(&self) -> anyhow::Result<()> {
        match self.cmd {
            SubCommand::Build(ref cmd) => cmd.run(),
            SubCommand::Prepare(ref cmd) => cmd.run(),
        }
    }
}

#[derive(Parser)]
enum SubCommand {
    /// Build the Docker image.
    ///
    /// Requires you to run `cargo xtask docker prepare` first.
    Build(Build),

    /// Prepare the binaries for the Docker image.
    ///
    /// Calls `cargo xtask dist` under-the-hood with all the docker
    /// targets.
    Prepare(Prepare),
}

#[derive(Parser)]
struct Build {
    /// Push the Docker image to the remote registry.
    #[clap(long)]
    push: bool,
}

impl XtaskCommand for Build {
    fn run(&self) -> anyhow::Result<()> {
        let platforms = vec!["linux/amd64", "linux/arm64", "linux/arm/v6", "linux/arm/v7"];

        let mut cmd = Command::new("docker");
        cmd.arg("buildx");
        cmd.arg("build");
        cmd.arg(".");
        cmd.arg("-t");
        cmd.arg("ghcr.io/m-rots/a-train");
        cmd.arg("--platform");
        cmd.arg(platforms.join(","));

        if self.push {
            cmd.arg("--push");
        }

        let status = cmd.status()?;

        ensure!(
            status.success(),
            "Docker build unsuccessful with status: {}",
            status
        );

        Ok(())
    }
}

#[derive(Parser)]
struct Prepare {
    #[clap(long)]
    skip_build: bool,
}

impl XtaskCommand for Prepare {
    fn run(&self) -> anyhow::Result<()> {
        let dist = super::dist::Dist::new(
            vec![
                "x86_64-unknown-linux-musl",
                "aarch64-unknown-linux-musl",
                "arm-unknown-linux-musleabihf",
                "armv7-unknown-linux-musleabihf",
            ],
            self.skip_build,
        );

        dist.run()
    }
}
