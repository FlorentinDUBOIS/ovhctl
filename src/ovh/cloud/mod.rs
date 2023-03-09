//! # Cloud module
//!
//! This module provide structure to interact with the cloud api
use std::error::Error;
use std::net::IpAddr;

use prettytable::{Cell, Row, Table};
use serde::{Deserialize, Serialize};

use crate::cmd::fmt::{Short, Wide};
use crate::ovh::{Client, RestClient};
use crate::util::types;

pub mod loadbalancer;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tenant {
    #[serde(rename = "project_id")]
    pub project_id: String,
    #[serde(rename = "description")]
    pub description: String,
    #[serde(rename = "planCode")]
    pub plan_code: String,
    #[serde(rename = "unleash")]
    pub unleash: bool,
    #[serde(rename = "status")]
    pub status: String,
    #[serde(rename = "access")]
    pub access: String,
}

impl Short for Vec<Tenant> {
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    fn short(&self) -> Result<String, Self::Error> {
        let mut rows = vec![Row::new(vec![
            Cell::new("Tenant"),
            Cell::new("Status"),
            Cell::new("Description"),
            Cell::new("Plan code"),
            Cell::new("Unleash"),
            Cell::new("Access"),
        ])];

        for tenant in self {
            let row = Row::new(vec![
                Cell::new(&tenant.project_id),
                Cell::new(&tenant.status),
                Cell::new(&tenant.description),
                Cell::new(&tenant.plan_code),
                Cell::new(&format!("{}", tenant.unleash)),
                Cell::new(&tenant.access),
            ]);

            rows.push(row);
        }

        Ok(format!("{}", Table::init(rows)))
    }
}

impl Wide for Vec<Tenant> {
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    fn wide(&self) -> Result<String, Self::Error> {
        let mut rows = vec![Row::new(vec![
            Cell::new("Tenant"),
            Cell::new("Status"),
            Cell::new("Description"),
            Cell::new("Plan code"),
            Cell::new("Unleash"),
            Cell::new("Access"),
        ])];

        for tenant in self {
            let row = Row::new(vec![
                Cell::new(&tenant.project_id),
                Cell::new(&tenant.status),
                Cell::new(&tenant.description),
                Cell::new(&tenant.plan_code),
                Cell::new(&format!("{}", tenant.unleash)),
                Cell::new(&tenant.access),
            ]);

            rows.push(row);
        }

        Ok(format!("{}", Table::init(rows)))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IpAddress {
    #[serde(rename = "ip")]
    pub ip: IpAddr,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(rename = "version")]
    pub version: i32,
    #[serde(rename = "networkId")]
    pub network_id: String,
    #[serde(rename = "gatewayIp")]
    pub gateway_ip: Option<String>,
}

impl Short for Vec<IpAddress> {
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    fn short(&self) -> Result<String, Self::Error> {
        let mut rows = vec![Row::new(vec![
            Cell::new("Identifier"),
            Cell::new("IP"),
            Cell::new("Type"),
            Cell::new("Version"),
            Cell::new("Gateway IP"),
        ])];

        for address in self {
            let gateway_ip = match &address.gateway_ip {
                Some(gateway_ip) => gateway_ip.to_owned(),
                None => String::from("<none>"),
            };

            let row = Row::new(vec![
                Cell::new(&address.network_id),
                Cell::new(&address.ip.to_string()),
                Cell::new(&address.kind),
                Cell::new(&format!("{}", address.version)),
                Cell::new(&gateway_ip),
            ]);

            rows.push(row);
        }

        Ok(format!("{}", Table::init(rows)))
    }
}

impl Wide for Vec<IpAddress> {
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    fn wide(&self) -> Result<String, Self::Error> {
        let mut rows = vec![Row::new(vec![
            Cell::new("Identifier"),
            Cell::new("IP"),
            Cell::new("Type"),
            Cell::new("Version"),
            Cell::new("Gateway IP"),
        ])];

        for address in self {
            let gateway_ip = match &address.gateway_ip {
                Some(gateway_ip) => gateway_ip.to_owned(),
                None => String::from("<none>"),
            };

            let row = Row::new(vec![
                Cell::new(&address.network_id),
                Cell::new(&address.ip.to_string()),
                Cell::new(&address.kind),
                Cell::new(&format!("{}", address.version)),
                Cell::new(&gateway_ip),
            ]);

            rows.push(row);
        }

        Ok(format!("{}", Table::init(rows)))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Instance {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "ipAddresses")]
    pub ip_addresses: Vec<IpAddress>,
    #[serde(rename = "flavorId")]
    pub flavor_id: String,
    #[serde(rename = "imageId")]
    pub image_id: String,
    #[serde(rename = "region")]
    pub region: String,
    #[serde(rename = "status")]
    pub status: String,
    #[serde(rename = "planCode")]
    pub plan_code: String,
}

impl Short for Vec<Instance> {
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    fn short(&self) -> Result<String, Self::Error> {
        let mut rows = vec![Row::new(vec![
            Cell::new("Identifier"),
            Cell::new("Name"),
            Cell::new("Region"),
            Cell::new("Status"),
            Cell::new("Plan code"),
        ])];

        for instance in self {
            let row = Row::new(vec![
                Cell::new(&instance.id),
                Cell::new(&instance.name),
                Cell::new(&instance.region),
                Cell::new(&instance.status),
                Cell::new(instance.plan_code.trim_end_matches(".consumption")),
            ]);

            rows.push(row);
        }

        Ok(format!("{}", Table::init(rows)))
    }
}

impl Wide for Vec<Instance> {
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    fn wide(&self) -> Result<String, Self::Error> {
        let mut rows = vec![Row::new(vec![
            Cell::new("Identifier"),
            Cell::new("Name"),
            Cell::new("Region"),
            Cell::new("Status"),
            Cell::new("Flavor"),
            Cell::new("Image"),
            Cell::new("Plan code"),
        ])];

        for instance in self {
            let row = Row::new(vec![
                Cell::new(&instance.id),
                Cell::new(&instance.name),
                Cell::new(&instance.region),
                Cell::new(&instance.status),
                Cell::new(&instance.flavor_id),
                Cell::new(&instance.image_id),
                Cell::new(&instance.plan_code),
            ]);

            rows.push(row);
        }

        Ok(format!("{}", Table::init(rows)))
    }
}

#[tracing::instrument(skip(client))]
pub async fn list_tenants(client: &Client) -> types::Result<Vec<Tenant>> {
    let ids: Vec<String> = client
        .get("cloud/project")
        .await
        .map_err(|err| format!("could not retrieve tenants, {}", err))?;

    let mut tenants = vec![];
    for id in ids {
        tenants.push(
            client
                .get(&format!("cloud/project/{}", id))
                .await
                .map_err(|err| format!("could not retrieve tenant '{}', {}", id, err))?,
        );
    }

    Ok(tenants)
}

#[tracing::instrument(skip(client))]
pub async fn list_instances(client: &Client, tenant: &str) -> types::Result<Vec<Instance>> {
    Ok(client
        .get(&format!("cloud/project/{}/instance", tenant))
        .await
        .map_err(|err| {
            format!(
                "could not retrieve instance for tenant '{}', {}",
                tenant, err
            )
        })?)
}
