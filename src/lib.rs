use autoscan::{Autoscan, AutoscanBuilder};
use bernard::{Bernard, BernardBuilder};
use reqwest::IntoUrl;
use thiserror::Error;

mod autoscan;
mod config;
mod drive;

pub use config::Config;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Autoscan is unavailable")]
    AutoscanUnavailable(#[from] autoscan::AutoscanError),
    #[error("Bernard")]
    Bernard(#[from] bernard::Error),
    #[error(transparent)]
    Unexpected(#[from] eyre::Report),
    #[error("Invalid configuration")]
    Configuration(#[from] config::ConfigError),
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

    pub async fn build(self) -> Result<Atrain> {
        let a_train = Atrain {
            autoscan: self.autoscan.build(),
            bernard: self.bernard.build().await.unwrap(),
            drives: self.drives,
        };

        // Check whether Autoscan is available.
        a_train.autoscan.available().await?;
        Ok(a_train)
    }
}
