//! # loadbalancer module
//!
//! This module provide handlers to manage load balancer
use std::convert::TryFrom;
use std::sync::Arc;

use crate::cfg::Configuration;
use crate::cmd::fmt::{Formatter, Json, Kind, Short, Wide, Yaml};
use crate::lib::types;
use crate::ovh::cloud::loadbalancer;
use crate::ovh::{Client, ClientConfiguration};

#[tracing::instrument]
pub async fn list(config: Arc<Configuration>, output: &Kind, tenant: &str) -> types::Result<()> {
    let client = Client::from(ClientConfiguration::try_from(config).map_err(|err| {
        format!(
            "could not create internal client configuration from the current configuration, {}",
            err
        )
    })?);

    let loadbalancers = loadbalancer::list(&client, tenant).await?;
    let formatter = Formatter::from(loadbalancers.to_owned());
    let o = match output {
        Kind::Short => loadbalancers.short()?,
        Kind::Wide => loadbalancers.wide()?,
        Kind::Json => formatter.json()?,
        Kind::Yaml => formatter.yaml()?,
    };

    println!("{}", o);

    Ok(())
}

#[tracing::instrument]
pub async fn create(
    config: Arc<Configuration>,
    output: &Kind,
    tenant: &str,
    region: &str,
) -> types::Result<()> {
    let client = Client::from(ClientConfiguration::try_from(config).map_err(|err| {
        format!(
            "could not create internal client configuration from the current configuration, {}",
            err
        )
    })?);

    let loadbalancers = vec![loadbalancer::create(&client, tenant, &region.into()).await?];
    let formatter = Formatter::from(loadbalancers.to_owned());
    let o = match output {
        Kind::Short => loadbalancers.short()?,
        Kind::Wide => loadbalancers.wide()?,
        Kind::Json => formatter.json()?,
        Kind::Yaml => formatter.yaml()?,
    };

    println!("{}", o);

    Ok(())
}

#[tracing::instrument]
pub async fn delete(
    config: Arc<Configuration>,
    output: &Kind,
    tenant: &str,
    id: &str,
) -> types::Result<()> {
    let client = Client::from(ClientConfiguration::try_from(config).map_err(|err| {
        format!(
            "could not create internal client configuration from the current configuration, {}",
            err
        )
    })?);

    loadbalancer::delete(&client, tenant, id).await?;

    let loadbalancers = loadbalancer::list(&client, tenant).await?;
    let formatter = Formatter::from(loadbalancers.to_owned());
    let o = match output {
        Kind::Short => loadbalancers.short()?,
        Kind::Wide => loadbalancers.wide()?,
        Kind::Json => formatter.json()?,
        Kind::Yaml => formatter.yaml()?,
    };

    println!("{}", o);

    Ok(())
}
