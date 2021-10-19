use crate::commands::XtaskCommand;
use clap::{AppSettings, Parser};
use commands::{Ci, Dist, Docker};

mod commands;

#[derive(Parser)]
#[clap(setting = AppSettings::PropagateVersion)]
enum SubCommand {
    Ci(Ci),
    Dist(Dist),
    Docker(Docker),
}

#[derive(Parser)]
struct Opt {
    #[clap(subcommand)]
    cmd: SubCommand,
}

pub fn run() -> anyhow::Result<()> {
    let opt = Opt::parse();

    match opt.cmd {
        SubCommand::Ci(cmd) => cmd.run(),
        SubCommand::Dist(cmd) => cmd.run(),
        SubCommand::Docker(cmd) => cmd.run(),
    }
}
