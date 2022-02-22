//! # Configuration module
//!
//! This module provide utilities to parse configuration
use std::{convert::TryFrom, error::Error, path::PathBuf, env};

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
        Config::builder()
            .set_default("ovh.endpoint", "https://eu.api.ovh.com/1.0")?
            .add_source(File::from(path).required(true))
            .build()
            .map_err(|err| format!("failed to load configuration, {}", err))?
            .try_deserialize()
            .map_err(|err| format!("failed to deserialize configuration, {}", err).into())
    }
}

impl Configuration {
    pub fn try_new() -> Result<Self, Box<dyn Error>> {
        Config::builder()
            .set_default("ovh.endpoint", "https://eu.api.ovh.com/1.0")?
            .add_source(
                File::with_name(&format!("/etc/{}/config", env!("CARGO_PKG_NAME"))).required(false),
            )
            .add_source(
                File::with_name(&format!("{}/.{}", env::var("HOME")?, env!("CARGO_PKG_NAME")))
                    .required(false),
            )
            .add_source(File::with_name("config").required(false))
            .add_source(Environment::with_prefix(env!("CARGO_PKG_NAME")))
            .build()
            .map_err(|err| format!("failed to load configuration, {}", err))?
            .try_deserialize()
            .map_err(|err| format!("failed to deserialize configuration, {}", err).into())
    }
}
