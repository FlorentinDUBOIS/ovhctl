//! # Server module
//!
//! This module provide controller to handle server handlers
use std::convert::TryFrom;
use std::sync::Arc;

use crate::cfg::Configuration;
use crate::cmd::fmt::{Formatter, Kind, Short, Wide, JSON, YAML};
use crate::lib::types;
use crate::ovh::dedicated::server;
use crate::ovh::{Client, ClientConfiguration};

pub async fn list_servers(config: Arc<Configuration>, output: &Kind) -> types::Result<()> {
    let client = Client::from(ClientConfiguration::try_from(config).map_err(|err| {
        format!(
            "could not create ovh client configuration from the current configuration, {}",
            err
        )
    })?);

    let servers = server::list_servers(&client).await?;
    let formatter = Formatter::from(servers.to_owned());
    let o = match output {
        Kind::Short => servers.short()?,
        Kind::Wide => servers.wide()?,
        Kind::JSON => formatter.json()?,
        Kind::YAML => formatter.yaml()?,
    };

    println!("{}", o);

    Ok(())
}
