//! # Domain module
//!
//! This module provide structure to interact with the domain api
use std::error::Error;

use prettytable::{Cell, Row, Table};
use serde::{Deserialize, Serialize};

use crate::cmd::fmt::{Short, Wide};
use crate::lib::types;
use crate::ovh::{Client, RestClient};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Zone {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "dnssecSupported")]
    pub dnssec_supported: bool,
    #[serde(rename = "hasDnsAnycast")]
    pub has_dns_anycast: bool,
    #[serde(rename = "nameServers")]
    pub name_servers: Vec<String>,
}

impl Short for Vec<Zone> {
    type Error = Box<dyn Error + Send + Sync>;

    fn short(&self) -> Result<String, Self::Error> {
        let mut rows = vec![Row::new(vec![
            Cell::new("Name"),
            Cell::new("DNS Sec"),
            Cell::new("DNS AnyCast"),
            Cell::new("Servers"),
        ])];

        for zone in self {
            let row = Row::new(vec![
                Cell::new(&zone.name),
                Cell::new(&format!("{}", zone.dnssec_supported)),
                Cell::new(&format!("{}", zone.has_dns_anycast)),
                Cell::new(&zone.name_servers.join(", ")),
            ]);

            rows.push(row);
        }

        Ok(format!("{}", Table::init(rows)))
    }
}

impl Wide for Vec<Zone> {
    type Error = Box<dyn Error + Send + Sync>;

    fn wide(&self) -> Result<String, Self::Error> {
        let mut rows = vec![Row::new(vec![
            Cell::new("Name"),
            Cell::new("DNS Sec"),
            Cell::new("DNS AnyCast"),
            Cell::new("Servers"),
        ])];

        for zone in self {
            let row = Row::new(vec![
                Cell::new(&zone.name),
                Cell::new(&format!("{}", zone.dnssec_supported)),
                Cell::new(&format!("{}", zone.has_dns_anycast)),
                Cell::new(&zone.name_servers.join(", ")),
            ]);

            rows.push(row);
        }

        Ok(format!("{}", Table::init(rows)))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Record {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(rename = "fieldType")]
    pub field_type: String,
    #[serde(rename = "subDomain")]
    pub sub_domain: String,
    #[serde(rename = "ttl", skip_serializing_if = "Option::is_none")]
    pub ttl: Option<i64>,
    #[serde(rename = "zone", skip_serializing)]
    pub zone: String,
    #[serde(rename = "target")]
    pub target: String,
}

impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.field_type == other.field_type
            && self.sub_domain == other.sub_domain
            && self.zone == other.zone
            && self.target == other.target
    }
}

impl Short for Vec<Record> {
    type Error = Box<dyn Error + Send + Sync>;

    fn short(&self) -> Result<String, Self::Error> {
        let mut rows = vec![Row::new(vec![
            Cell::new("Identifier"),
            Cell::new("Zone"),
            Cell::new("Type"),
            Cell::new("Sub domain"),
            Cell::new("TTL"),
            Cell::new("Target"),
        ])];

        for record in self {
            let id = match record.id {
                Some(id) => format!("{}", id),
                None => String::from("<none>"),
            };

            let ttl = match record.id {
                Some(ttl) => format!("{}", ttl),
                None => String::from("<none>"),
            };

            let row = Row::new(vec![
                Cell::new(&id),
                Cell::new(&record.zone),
                Cell::new(&record.field_type),
                Cell::new(&record.sub_domain),
                Cell::new(&ttl),
                Cell::new(&record.target),
            ]);

            rows.push(row);
        }

        Ok(format!("{}", Table::init(rows)))
    }
}

impl Wide for Vec<Record> {
    type Error = Box<dyn Error + Send + Sync>;

    fn wide(&self) -> Result<String, Self::Error> {
        let mut rows = vec![Row::new(vec![
            Cell::new("Identifier"),
            Cell::new("Zone"),
            Cell::new("Type"),
            Cell::new("Sub domain"),
            Cell::new("TTL"),
            Cell::new("Target"),
        ])];

        for record in self {
            let id = match record.id {
                Some(id) => format!("{}", id),
                None => String::from("<none>"),
            };

            let ttl = match record.id {
                Some(ttl) => format!("{}", ttl),
                None => String::from("<none>"),
            };

            let row = Row::new(vec![
                Cell::new(&id),
                Cell::new(&record.zone),
                Cell::new(&record.field_type),
                Cell::new(&record.sub_domain),
                Cell::new(&ttl),
                Cell::new(&record.target),
            ]);

            rows.push(row);
        }

        Ok(format!("{}", Table::init(rows)))
    }
}

pub async fn list_zones(client: &Client) -> types::Result<Vec<Zone>> {
    let ids: Vec<String> = client
        .get("domain/zone")
        .await
        .map_err(|err| format!("could not retrieve zones, {}", err))?;

    let mut zones = vec![];
    for id in ids {
        zones.push(
            client
                .get(&format!("domain/zone/{}", id))
                .await
                .map_err(|err| format!("could not retrieve zone '{}', {}", id, err))?,
        );
    }

    Ok(zones)
}

pub async fn list_records(client: &Client, zone: &str) -> types::Result<Vec<Record>> {
    let ids: Vec<i64> = client
        .get(&format!("domain/zone/{}/record", zone))
        .await
        .map_err(|err| format!("could not retrieve records in zone '{}', {}", zone, err))?;

    let mut zones = vec![];
    for id in ids {
        zones.push(
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
    }

    Ok(zones)
}

pub async fn create_record(client: &Client, zone: &str, record: &Record) -> types::Result<Record> {
    Ok(client
        .post(&format!("domain/zone/{}/record", zone), record)
        .await?)
}

pub async fn delete_record(client: &Client, zone: &str, id: &i64) -> types::Result<()> {
    Ok(client
        .delete(&format!("domain/zone/{}/record/{}", zone, id))
        .await?)
}

pub async fn refresh_records(client: &Client, zone: &str) -> types::Result<()> {
    Ok(client
        .post(&format!("domain/zone/{}/refresh", zone), &"")
        .await
        .map_err(|err| format!("could not refresh domain records, {}", err))?)
}
