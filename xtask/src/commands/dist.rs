use crate::commands::{dist_path, source_path, Section, XtaskCommand};
use anyhow::Context;
use clap::Clap;
use flate2::{write::GzEncoder, Compression};
use std::path::{Path, PathBuf};
use std::{fs, io};
use xshell::cmd;

#[derive(Clap)]
pub(crate) struct Dist {
    #[clap(long)]
    skip_build: bool,
    #[clap(required = true)]
    targets: Vec<String>,
}

impl Dist {
    pub(crate) fn new<I, T>(targets: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        Self {
            skip_build: false,
            targets: targets.into_iter().map(Into::into).collect(),
        }
    }
}

impl XtaskCommand for Dist {
    fn run(&self) -> anyhow::Result<()> {
        reset_dist()?;

        for target in &self.targets {
            if !self.skip_build {
                cross_build(&target)?;
            }

            package(&target)?;
        }

        Ok(())
    }
}

fn reset_dist() -> anyhow::Result<()> {
    let _s = Section::new("Resetting dist directory");

    let path = dist_path();
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }

    fs::create_dir_all(&path)?;
    Ok(())
}

fn cross_build(target: &str) -> xshell::Result<()> {
    let _s = Section::new(format!("Build: {}", target));
    cmd!("cross build --release --target {target} --package a-train").run()
}

fn package(target: &str) -> anyhow::Result<()> {
    let _s = Section::new(format!("Package: {}", target));

    let src = source_path(&target);
    strip(&src)?;

    let dst = destination_path(&target);
    gzip(&src, &dst)?;

    Ok(())
}

fn strip<S>(src: S) -> anyhow::Result<()>
where
    S: AsRef<Path>,
{
    let src = src.as_ref();
    cmd!("strip {src}").run()?;
    Ok(())
}

fn gzip<S, T>(src: S, dst: T) -> anyhow::Result<()>
where
    S: AsRef<Path>,
    T: AsRef<Path>,
{
    // Open the source file
    let src = src.as_ref();
    let src = fs::File::open(&src).context(format!("source path {:?} does not exist", &src))?;

    // Create the destination file
    let dst = dst.as_ref();
    let dst =
        fs::File::create(&dst).context(format!("cannot create destination file at {:?}", &dst))?;

    // Compress the source content into the destination file
    let mut encoder = GzEncoder::new(dst, Compression::best());
    let mut input = io::BufReader::new(src);
    io::copy(&mut input, &mut encoder)?;
    encoder.finish()?;

    Ok(())
}

fn destination_path(target: &str) -> PathBuf {
    let suffix = if target.contains("-windows-") {
        ".exe.gz"
    } else {
        ".gz"
    };

    dist_path().join(format!("a-train-{}{}", target, suffix))
}
