//! # ovhctl
//!
//! A command line interface to improve our life at ovh
use std::{cmp::min, convert::TryFrom, error::Error, sync::Arc};

use slog::{o, Drain, Level, LevelFilter, Logger};
use slog_async::Async;
use slog_scope::{crit, debug, info, set_global_logger, warn, GlobalLoggerGuard as Guard};
use slog_term::{FullFormat, TermDecorator};

use crate::{
    cfg::Configuration,
    cmd::{Args, Execute},
};

// library module should be declare first as it expose macros used by other modules
// https://doc.rust-lang.org/1.2.0/book/macros.html#scoping-and-macro-import/export
#[macro_use]
mod lib;
mod cfg;
mod cmd;
mod ovh;

#[inline]
fn init(verbose: usize) -> Guard {
    let level = min(
        Level::Trace.as_usize(),
        Level::Critical.as_usize() + verbose,
    );
    let level = Level::from_usize(level).unwrap_or(Level::Trace);

    let decorator = TermDecorator::new().build();
    let drain = FullFormat::new(decorator).build().fuse();
    let drain = Async::new(drain).build().fuse();
    let drain = LevelFilter::new(drain, level).fuse();

    set_global_logger(Logger::root(drain, o!()))
}

#[paw::main]
#[tokio::main]
async fn main(args: Args) -> Result<(), Box<dyn Error>> {
    let _guard = init(args.verbose);
    let config = match args.config.to_owned() {
        Some(path) => Configuration::try_from(path),
        None => Configuration::try_new(),
    };

    let config = match config {
        Ok(config) => Arc::new(config),
        Err(err) => {
            crit!("could not load configuration"; "error" => err.to_string());
            return Err(err);
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
            crit!("could not execute command"; "error" => err.to_string());
            return Err(err);
        }
    }

    Ok(())
}
