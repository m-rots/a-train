use a_train::Config;
use anyhow::anyhow;
use clap::Clap;
use std::env;
use tokio::signal::ctrl_c;
use tracing_subscriber::fmt::format::FmtSpan;

// Use Jemalloc only for musl 64 bits platforms.
// This fixes worse performance on MUSL builds.
// More info: https://github.com/BurntSushi/ripgrep/blob/94e4b8e301302097dad48b292560ce135c4d4926/crates/core/main.rs#L44
#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

/// Google Drive support for Autoscan.
#[derive(Clap)]
#[clap(name = "A-Train", version = version())]
struct Opt {
    /// Path to the configuration file
    #[clap(short, long, value_name = "FILE", default_value = "a-train.toml")]
    config: String,

    /// Path to the database file
    #[clap(long, alias = "db", value_name = "FILE", default_value = "a-train.db")]
    database: String,

    /// Proxy URL to use for debugging
    #[clap(short, long, value_name = "URL")]
    proxy: Option<String>,
}

fn version() -> &'static str {
    env!("VERGEN_BUILD_SEMVER")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    let build_time = env!("VERGEN_BUILD_TIMESTAMP");
    let build_time = chrono::DateTime::parse_from_rfc3339(build_time)
        .map_err(|_| anyhow!("Whoops! This build is no longer valid."))?;

    if chrono::Utc::now() > build_time + chrono::Duration::weeks(4) {
        return Err(anyhow!("Whoops! This build is no longer valid."));
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "a_train=info,bernard=info".to_owned()),
        )
        .with_span_events(FmtSpan::CLOSE)
        .pretty()
        .init();

    let config = Config::new(&opt.config)?;

    let mut a_train = a_train::AtrainBuilder::new(config, &opt.database)?;
    if let Some(url) = opt.proxy {
        a_train = a_train.proxy(url);
    }

    let a_train = a_train.build().await;

    loop {
        tokio::select! {
            result = a_train.tick() => result?,
            _ = ctrl_c() => break,
        }
    }

    a_train.close().await;

    Ok(())
}
