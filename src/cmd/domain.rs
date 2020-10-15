//! # Domain module
//!
//! This module provide controller to handle domain handlers
use std::convert::TryFrom;
use std::sync::Arc;
use std::net::IpAddr;

use ipnetwork::IpNetwork;

use crate::cfg::Configuration;
use crate::cmd::fmt::{Formatter, Kind, Short, Wide, JSON, YAML};
use crate::lib::types::Result;
use crate::ovh::cloud;
use crate::ovh::domain;
use crate::ovh::domain::{create_record, Record};
use crate::ovh::{Client, ClientConfiguration};

pub async fn list_zones(config: Arc<Configuration>, output: &Kind) -> Result<()> {
    let client = Client::from(ClientConfiguration::try_from(config).map_err(|err| {
        format!(
            "could not create ovh client configuration from the current configuration, {}",
            err
        )
    })?);

    let zones = domain::list_zones(&client).await?;
    let formatter = Formatter::from(zones.to_owned());
    let o = match output {
        Kind::Short => zones.short()?,
        Kind::Wide => zones.wide()?,
        Kind::JSON => formatter.json()?,
        Kind::YAML => formatter.yaml()?,
    };

    println!("{}", o);

    Ok(())
}

pub async fn list_records(config: Arc<Configuration>, zone: &str, output: &Kind) -> Result<()> {
    let client = Client::from(ClientConfiguration::try_from(config).map_err(|err| {
        format!(
            "could not create ovh client configuration from the current configuration, {}",
            err
        )
    })?);

    let records = domain::list_records(&client, zone).await?;
    let formatter = Formatter::from(records.to_owned());
    let o = match output {
        Kind::Short => records.short()?,
        Kind::Wide => records.wide()?,
        Kind::JSON => formatter.json()?,
        Kind::YAML => formatter.yaml()?,
    };

    println!("{}", o);

    Ok(())
}

// todo(florentin.dubois): handle dedicated servers
// todo(florentin.dubois): handle update and deletion
pub async fn sync_records(
    config: Arc<Configuration>,
    zone: &str,
    output: &Kind,
    not_in_cidrs: &[IpNetwork],
) -> Result<()> {
    let client = Client::from(ClientConfiguration::try_from(config).map_err(|err| {
        format!(
            "could not create ovh client configuration from the current configuration, {}",
            err
        )
    })?);

    let tenants = cloud::list_tenants(&client).await?;
    let mut instances = vec![];
    for tenant in &tenants {
        instances.push(cloud::list_instances(&client, &tenant.project_id).await?);
    }

    let records: Vec<Record> = instances
        .iter()
        .flatten()
        .map(|instance| {
            let mut records = vec![];
            for address in &instance.ip_addresses {
                if "public" != address.kind {
                    continue;
                }

                for cidr in not_in_cidrs {
                    if cidr.contains(address.ip.parse::<IpAddr>().unwrap()) {
                        continue;
                    }
                }

                let field_type = match address.version {
                    4 => String::from("A"),
                    6 => String::from("AAAA"),
                    _ => continue,
                };

                records.push(Record {
                    id: None,
                    field_type,
                    sub_domain: String::from(
                        instance.name.trim_end_matches(&(String::from(".") + zone)),
                    ),
                    ttl: None,
                    zone: String::from(zone),
                    target: address.ip.to_owned(),
                });
            }

            records
        })
        .flatten()
        .collect();

    let existing_records = domain::list_records(&client, zone).await?;
    let mut records_to_create = vec![];
    for record in &records {
        if existing_records.contains(record) {
            continue;
        }

        records_to_create.push(record.to_owned());
    }

    let mut record_created = vec![];
    for record in &records_to_create {
        record_created.push(create_record(&client, zone, record).await?);
    }

    let formatter = Formatter::from(record_created.to_owned());
    let o = match output {
        Kind::Short => record_created.short()?,
        Kind::Wide => record_created.wide()?,
        Kind::JSON => formatter.json()?,
        Kind::YAML => formatter.yaml()?,
    };

    println!("{}", o);

    Ok(())
}

pub async fn refresh_records(config: Arc<Configuration>, zone: &str) -> Result<()> {
    let client = Client::from(ClientConfiguration::try_from(config).map_err(|err| {
        format!(
            "could not create ovh client configuration from the current configuration, {}",
            err
        )
    })?);

    Ok(domain::refresh_records(&client, zone).await?)
}

pub async fn delete_record(config: Arc<Configuration>, zone: &str, id: &i64) -> Result<()> {
    let client = Client::from(ClientConfiguration::try_from(config).map_err(|err| {
        format!(
            "could not create ovh client configuration from the current configuration, {}",
            err
        )
    })?);

    Ok(domain::delete_record(&client, zone, &id).await?)
}