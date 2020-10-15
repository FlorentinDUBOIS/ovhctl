//! # Cloud module
//!
//! This module provide controller to handle cloud handlers
use std::convert::TryFrom;
use std::sync::Arc;

use crate::cfg::Configuration;
use crate::cmd::fmt::{Formatter, Kind, Short, Wide, JSON, YAML};
use crate::lib::types::Result;
use crate::ovh::cloud;
use crate::ovh::{Client, ClientConfiguration};

pub async fn list_tenants(config: Arc<Configuration>, output: &Kind) -> Result<()> {
    let client = Client::from(ClientConfiguration::try_from(config).map_err(|err| {
        format!(
            "could not create ovh client configuration from the current configuration, {}",
            err
        )
    })?);

    let tenants = cloud::list_tenants(&client).await?;
    let formatter = Formatter::from(tenants.to_owned());
    let o = match output {
        Kind::Short => tenants.short()?,
        Kind::Wide => tenants.wide()?,
        Kind::JSON => formatter.json()?,
        Kind::YAML => formatter.yaml()?,
    };

    println!("{}", o);

    Ok(())
}

pub async fn list_instances(config: Arc<Configuration>, tenant: &str, output: &Kind) -> Result<()> {
    let client = Client::from(ClientConfiguration::try_from(config).map_err(|err| {
        format!(
            "could not create ovh client configuration from the current configuration, {}",
            err
        )
    })?);

    let instances = cloud::list_instances(&client, tenant).await?;
    let formatter = Formatter::from(instances.to_owned());
    let o = match output {
        Kind::Short => instances.short()?,
        Kind::Wide => instances.wide()?,
        Kind::JSON => formatter.json()?,
        Kind::YAML => formatter.yaml()?,
    };

    println!("{}", o);

    Ok(())
}
