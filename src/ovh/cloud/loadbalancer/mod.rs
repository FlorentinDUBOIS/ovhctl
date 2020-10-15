//! # Load balancer module
//!
//! This module provide structure to interact with the loadbalancer api
use std::error::Error;

use prettytable::{Cell, Row, Table};
use serde::{Deserialize, Serialize};

use crate::cmd::fmt::{Short, Wide};
use crate::lib::types;
use crate::ovh::{Client, RestClient};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigurationState {
    #[serde(rename = "applied")]
    pub applied: i64,
    #[serde(rename = "latest")]
    pub latest: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Address {
    #[serde(rename = "ipv4")]
    pub ip_v4: String,
    #[serde(rename = "ipv6", skip_serializing_if = "Option::is_some")]
    pub ip_v6: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoadBalancer {
    #[serde(rename = "id", skip_serializing_if = "Option::is_some")]
    pub id: Option<String>,
    #[serde(rename = "name", skip_serializing_if = "Option::is_some")]
    pub name: Option<String>,
    #[serde(rename = "description", skip_serializing_if = "Option::is_some")]
    pub description: Option<String>,
    #[serde(rename = "region")]
    pub region: String,
    #[serde(rename = "status")]
    pub status: String,
    #[serde(rename = "address")]
    pub address: Address,
    #[serde(rename = "configuration")]
    pub configuration: ConfigurationState,
}

impl Short for Vec<LoadBalancer> {
    type Error = Box<dyn Error + Send + Sync>;

    fn short(&self) -> Result<String, Self::Error> {
        let mut rows = vec![Row::new(vec![
            Cell::new("Identifier"),
            Cell::new("Name"),
            Cell::new("Description"),
            Cell::new("Region"),
            Cell::new("Status"),
            Cell::new("IPv4"),
        ])];

        for loadbalancer in self {
            let id = match loadbalancer.id.to_owned() {
                Some(id) => id,
                None => "<none>".into(),
            };

            let name = match loadbalancer.name.to_owned() {
                Some(name) => name,
                None => "<none>".into(),
            };

            let description = match loadbalancer.description.to_owned() {
                Some(description) => description,
                None => "<none>".into(),
            };

            let row = Row::new(vec![
                Cell::new(&id),
                Cell::new(&name),
                Cell::new(&description),
                Cell::new(&loadbalancer.region),
                Cell::new(&loadbalancer.status),
                Cell::new(&loadbalancer.address.ip_v4),
            ]);

            rows.push(row);
        }

        Ok(format!("{}", Table::init(rows)))
    }
}

impl Wide for Vec<LoadBalancer> {
    type Error = Box<dyn Error + Send + Sync>;

    fn wide(&self) -> Result<String, Self::Error> {
        let mut rows = vec![Row::new(vec![
            Cell::new("Identifier"),
            Cell::new("Name"),
            Cell::new("Description"),
            Cell::new("Region"),
            Cell::new("Status"),
            Cell::new("IPv4"),
            Cell::new("IPv6"),
            Cell::new("Applied"),
            Cell::new("Latest"),
        ])];

        for loadbalancer in self {
            let id = match loadbalancer.id.to_owned() {
                Some(id) => id,
                None => "<none>".into(),
            };

            let name = match loadbalancer.name.to_owned() {
                Some(name) => name,
                None => "<none>".into(),
            };

            let description = match loadbalancer.description.to_owned() {
                Some(description) => description,
                None => "<none>".into(),
            };

            let ip_v6 = match loadbalancer.address.ip_v6.to_owned() {
                Some(ip_v6) => ip_v6,
                None => "<none>".into(),
            };

            let row = Row::new(vec![
                Cell::new(&id),
                Cell::new(&name),
                Cell::new(&description),
                Cell::new(&loadbalancer.region),
                Cell::new(&loadbalancer.status),
                Cell::new(&loadbalancer.address.ip_v4),
                Cell::new(&ip_v6),
                Cell::new(&format!("{}", loadbalancer.configuration.applied)),
                Cell::new(&format!("{}", loadbalancer.configuration.latest)),
            ]);

            rows.push(row);
        }

        Ok(format!("{}", Table::init(rows)))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoadBalancerCreation {
    #[serde(rename = "region")]
    pub region: String,
}

impl From<&str> for LoadBalancerCreation {
    fn from(region: &str) -> Self {
        Self {
            region: region.to_string(),
        }
    }
}

impl From<String> for LoadBalancerCreation {
    fn from(region: String) -> Self {
        Self { region }
    }
}

pub async fn list(client: &Client, tenant: &str) -> types::Result<Vec<LoadBalancer>> {
    let ids: Vec<String> = client
        .get(&format!("cloud/project/{}/loadbalancer", tenant))
        .await
        .map_err(|err| {
            format!(
                "could not list loadbalancer on tenant '{}', {}",
                tenant, err
            )
        })?;

    let mut loadbalancers = vec![];
    for id in ids {
        loadbalancers.push(
            client
                .get(&format!("cloud/project/{}/loadbalancer/{}", tenant, &id))
                .await
                .map_err(|err| {
                    format!(
                        "could not get loadbalancer '{}' on tenant '{}', {}",
                        id, tenant, err
                    )
                })?,
        );
    }

    Ok(loadbalancers)
}

pub async fn create(
    client: &Client,
    tenant: &str,
    opts: &LoadBalancerCreation,
) -> types::Result<LoadBalancer> {
    Ok(client
        .post(&format!("cloud/project/{}/loadbalancer", tenant), opts)
        .await
        .map_err(|err| format!("could not create loadbalancer, {}", err))?)
}

pub async fn delete(client: &Client, tenant: &str, id: &str) -> types::Result<()> {
    Ok(client
        .delete(&format!("cloud/project/{}/loadbalancer/{}", tenant, id))
        .await
        .map_err(|err| format!("could not delete loadbalancer '{}', {}", id, err))?)
}
