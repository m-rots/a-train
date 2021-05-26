use autoscan::{Autoscan, AutoscanBuilder};
use bernard::{Bernard, BernardBuilder};
use reqwest::IntoUrl;
use snafu::Snafu;
use std::path::PathBuf;

mod autoscan;
mod config;
mod drive;

pub use config::Config;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Autoscan is unavailable"))]
    AutoscanUnavailable { source: reqwest::Error },
    #[snafu(display("Bernard gave up?"))]
    BernardError { source: bernard::Error },
    #[snafu(display("Could not parse the configuration file from {:?}", path))]
    InvalidConfiguration {
        source: toml::de::Error,
        path: PathBuf,
    },
    #[snafu(display("Invalid service account key file"))]
    InvalidServiceAccount { source: bernard::Error },
    #[snafu(display("Unable to read configuration file from {:?}", path))]
    ReadConfiguration {
        source: std::io::Error,
        path: PathBuf,
    },
}

impl From<bernard::Error> for Error {
    fn from(source: bernard::Error) -> Self {
        Self::BernardError { source }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Atrain {
    autoscan: Autoscan,
    bernard: Bernard,
    drives: Vec<String>,
}

impl Atrain {
    pub async fn tick(&self) -> Result<()> {
        use tokio::time::{sleep, Duration};

        self.sync().await?;
        sleep(Duration::from_secs(60)).await;
        Ok(())
    }
}

pub struct AtrainBuilder {
    autoscan: AutoscanBuilder,
    bernard: BernardBuilder,
    drives: Vec<String>,
}

impl AtrainBuilder {
    pub fn new(config: Config, database_path: &str) -> Result<AtrainBuilder> {
        let account = config.account()?;

        Ok(Self {
            autoscan: Autoscan::builder(config.autoscan.url, config.autoscan.authentication),
            bernard: Bernard::builder(database_path, account),
            drives: config.drive.drives,
        })
    }

    pub fn proxy<U: IntoUrl + Clone>(mut self, url: U) -> Self {
        self.autoscan = self.autoscan.proxy(url.clone());
        self.bernard = self.bernard.proxy(url);
        self
    }

    pub async fn build(self) -> Atrain {
        Atrain {
            autoscan: self.autoscan.build(),
            bernard: self.bernard.build().await.unwrap(),
            drives: self.drives,
        }
    }
}
