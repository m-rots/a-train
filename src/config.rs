use crate::autoscan::Credentials;
use crate::{InvalidConfiguration, InvalidServiceAccount, ReadConfiguration, Result};
use bernard::Account;
use serde::Deserialize;
use snafu::ResultExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub(crate) autoscan: AutoscanConfig,
    pub(crate) drive: DriveConfig,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AutoscanConfig {
    #[serde(flatten)]
    pub(crate) authentication: Option<Credentials>,
    pub(crate) url: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DriveConfig {
    pub(crate) account: PathBuf,
    pub(crate) drives: Vec<String>,
}

impl Config {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Self> {
        let path = path.as_ref();

        let config = std::fs::read_to_string(path).context(ReadConfiguration { path })?;
        let config: Config = toml::from_str(&config).context(InvalidConfiguration { path })?;

        Ok(config)
    }

    pub fn account(&self) -> Result<Account> {
        let account = Account::from_file(&self.drive.account).context(InvalidServiceAccount)?;

        Ok(account)
    }
}
