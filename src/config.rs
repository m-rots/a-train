use crate::autoscan::Credentials;
use bernard::Account;
use eyre::WrapErr;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error(transparent)]
    UnexpectedError(#[from] eyre::Report),
}

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
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        let config = std::fs::read_to_string(path)
            .wrap_err_with(|| format!("Could not read config file at: {:?}", path))?;

        let config: Config = toml::from_str(&config).wrap_err("Configuration file is invalid")?;

        Ok(config)
    }

    pub fn account(&self) -> Result<Account, ConfigError> {
        let account = Account::from_file(&self.drive.account)
            .wrap_err_with(|| format!("Service Account is invalid: {:?}", self.drive.account))?;

        Ok(account)
    }
}
