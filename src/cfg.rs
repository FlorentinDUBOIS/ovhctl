//! # Configuration module
//!
//! This module provide utilities to parse configuration
use std::{
    convert::TryFrom,
    error::Error,
    path::PathBuf
};

use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Ovh {
    #[serde(rename = "endpoint")]
    pub endpoint: String,
    #[serde(rename = "application-key")]
    pub application_key: String,
    #[serde(rename = "application-secret")]
    pub application_secret: String,
    #[serde(rename = "consumer-key")]
    pub consumer_key: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Configuration {
    #[serde(rename = "ovh")]
    pub ovh: Ovh,
}

impl TryFrom<PathBuf> for Configuration {
    type Error = Box<dyn Error>;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let mut config = Config::default();

        config.set_default("ovh.endpoint", "https://eu.api.ovh.com/1.0")?;
        config
            .merge(File::from(path).required(true))
            .map_err(|err| format!("could not configure the file constraint, {}", err))?;

        Ok(config
            .try_into::<Self>()
            .map_err(|err| format!("could not cast data structure into configuration, {}", err))?)
    }
}

impl Configuration {
    pub fn try_new() -> Result<Self, Box<dyn Error>> {
        let mut config = Config::default();

        config.set_default("ovh.endpoint", "https://eu.api.ovh.com/1.0")?;
        config
            .merge(
                File::with_name(&format!("/etc/{}/config", env!("CARGO_PKG_NAME"))).required(false),
            )
            .map_err(|err| format!("could not configure the file constraint, {}", err))?;

        config
            .merge(
                File::with_name(&format!("{}/.{}", env!("HOME"), env!("CARGO_PKG_NAME")))
                    .required(false),
            )
            .map_err(|err| format!("could not configure the file constraint, {}", err))?;

        config
            .merge(File::with_name("config").required(false))
            .map_err(|err| format!("could not configure the file constraint, {}", err))?;

        config
            .merge(Environment::with_prefix(env!("CARGO_PKG_NAME")))
            .map_err(|err| format!("could not configure the environment constraint, {}", err))?;

        Ok(config
            .try_into::<Self>()
            .map_err(|err| format!("could not cast data structure into configuration, {}", err))?)
    }
}
