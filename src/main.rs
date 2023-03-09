//! # ovhctl
//!
//! A command line interface to improve our life at ovh
use std::{convert::TryFrom, error::Error as StdError, sync::Arc};

use tracing::{debug, error, info, warn};

use crate::{
    cfg::Configuration,
    cmd::{Args, Execute},
};

// library module should be declare first as it expose macros used by other modules
// https://doc.rust-lang.org/1.2.0/book/macros.html#scoping-and-macro-import/export
#[macro_use]
mod util;
mod cfg;
mod cmd;
pub mod logging;
mod ovh;

// -----------------------------------------------------------------------------
// Error enumeration

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to initialise logging system, {0}")]
    LoggingSystem(logging::Error),
    #[error("failed to load configuration, {0}")]
    Configuration(Box<dyn StdError + Send + Sync>),
    #[error("failed to execute command, {0}")]
    Command(Box<dyn StdError + Send + Sync>),
    #[error("failed to parse arguments, {0}")]
    ParseArgs(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::ParseArgs(err)
    }
}

// -----------------------------------------------------------------------------
// entrypoint

#[paw::main]
#[tokio::main(flavor = "current_thread")]
async fn main(args: Args) -> Result<(), Error> {
    logging::initialize(args.verbose).map_err(Error::LoggingSystem)?;
    let config = match args.config.to_owned() {
        Some(path) => Configuration::try_from(path),
        None => Configuration::try_new(),
    };

    let config = match config {
        Ok(config) => Arc::new(config),
        Err(err) => {
            error!("could not load configuration, {}", err);
            return Err(Error::Configuration(err));
        }
    };

    if args.check {
        debug!("Arguments: {:#?}", args);
        debug!("Configuration: {:#?}", config);
        info!("Configuration is healthy!");
        return Ok(());
    }

    if config.ovh.consumer_key.is_none() {
        warn!(
            "Please login to the ovh api by using '{} connect' before beginning",
            env!("CARGO_PKG_NAME")
        );
    }

    if let Some(cmd) = args.cmd {
        if let Err(err) = cmd.execute(config).await {
            error!("could not execute command, {}", err);
            return Err(Error::Command(err));
        }
    }

    Ok(())
}
