//! # Domain module
//!
//! This module provide controller to handle domain handlers
use std::convert::TryFrom;
use std::sync::Arc;

use ipnetwork::IpNetwork;
use pbr::ProgressBar;
use slog_scope::info;

use crate::cfg::Configuration;
use crate::cmd::fmt::{Formatter, Json, Kind, Short, Wide, Yaml};
use crate::lib::net;
use crate::lib::types::Result;
use crate::ovh::cloud::{list_instances, list_tenants};
use crate::ovh::domain::Record;
use crate::ovh::{domain, RestClient};
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
        Kind::Json => formatter.json()?,
        Kind::Yaml => formatter.yaml()?,
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
        Kind::Json => formatter.json()?,
        Kind::Yaml => formatter.yaml()?,
    };

    println!("{}", o);

    Ok(())
}

// todo(florentin.dubois): handle dedicated servers
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

    // -------------------------------------------------------------------------
    // retrieve instances

    info!("retrieve public cloud instances");
    let tenants = list_tenants(&client).await?;
    let mut pb = ProgressBar::new(tenants.len() as u64);
    let mut instances = vec![];
    for tenant in tenants {
        instances.append(&mut list_instances(&client, &tenant.project_id).await?);
        pb.inc();
    }

    pb.finish();

    // -------------------------------------------------------------------------
    // retrieve records

    // todo(florentin.dubois): use the domains::list_records function once optimized
    info!("retrieve dns records"; "zone" => zone);
    let ids: Vec<i64> = client
        .get(&format!("domain/zone/{}/record", zone))
        .await
        .map_err(|err| format!("could not retrieve records in zone '{}', {}", zone, err))?;

    let mut pb = ProgressBar::new(ids.len() as u64);
    let mut records = vec![];
    for id in ids {
        records.push(
            client
                .get(&format!("domain/zone/{}/record/{}", zone, id))
                .await
                .map_err(|err| {
                    format!(
                        "could not retrieve record '{}' in zone '{}', {}",
                        id, zone, err
                    )
                })?,
        );

        pb.inc();
    }

    pb.finish();

    // -------------------------------------------------------------------------
    // compute records diff

    info!("compute diff to apply"; "instances" => instances.len(), "records" => records.len());
    let mut pb = ProgressBar::new(instances.len() as u64);
    let mut records_to_create = vec![];
    let mut records_to_delete = vec![];
    for instance in instances {
        for address in instance.ip_addresses {
            let record = domain::contains(&records, &address.ip);
            if "public" != address.kind {
                if let Some(record) = record {
                    records_to_delete.push(record);
                }

                continue;
            }

            if net::contains(not_in_cidrs, address.ip).is_some() {
                if let Some(record) = record {
                    records_to_delete.push(record);
                }

                continue;
            }

            let field_type = match address.version {
                4 => String::from("A"),
                6 => String::from("AAAA"),
                _ => continue,
            };

            let new_record = Record {
                id: None,
                field_type,
                sub_domain: String::from(
                    instance.name.trim_end_matches(&(String::from(".") + zone)),
                ),
                ttl: None,
                zone: String::from(zone),
                target: address.ip.to_string(),
            };

            match record {
                Some(r) => {
                    if r != new_record {
                        records_to_delete.push(r);
                        records_to_create.push(new_record);
                    }
                }
                None => {
                    records_to_create.push(new_record);
                }
            }
        }

        pb.inc();
    }

    pb.finish();

    // -------------------------------------------------------------------------
    // Apply diff

    info!("apply diff"; "create" => records_to_create.len(), "delete" => records_to_delete.len());
    let mut pb = ProgressBar::new((records_to_delete.len() + records_to_create.len()) as u64);
    for record in records_to_delete {
        let id = match record.id {
            Some(id) => id,
            None => {
                pb.inc();
                continue;
            }
        };

        domain::delete_record(&client, zone, &id)
            .await
            .map_err(|err| format!("could not delete record '{}', {}", id, err))?;

        pb.inc();
    }

    for record in records_to_create {
        domain::create_record(&client, zone, &record)
            .await
            .map_err(|err| format!("could not create record, {}", err))?;

        pb.inc();
    }

    pb.finish();

    // -------------------------------------------------------------------------
    // Refresh records

    info!("refresh records");
    domain::refresh_records(&client, zone).await?;

    let records = domain::list_records(&client, zone).await?;
    let formatter = Formatter::from(records.to_owned());
    let o = match output {
        Kind::Short => records.short()?,
        Kind::Wide => records.wide()?,
        Kind::Json => formatter.json()?,
        Kind::Yaml => formatter.yaml()?,
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

    Ok(domain::delete_record(&client, zone, id).await?)
}
