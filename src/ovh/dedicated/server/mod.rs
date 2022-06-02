//! # Server module
//!
//! This module provide structure to interact with the server api
use std::error::Error;

use prettytable::{Cell, Row, Table};
use serde::{Deserialize, Serialize};

use crate::cmd::fmt::{Short, Wide};
use crate::lib::types;
use crate::ovh::{Client, RestClient};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Server {
    #[serde(rename = "reverse")]
    pub reverse: String,
    #[serde(rename = "state")]
    pub state: String,
    #[serde(rename = "monitoring")]
    pub monitoring: bool,
    #[serde(rename = "datacenter")]
    pub data_center: String,
    #[serde(rename = "rack")]
    pub rack: String,
    #[serde(rename = "os")]
    pub os: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "linkSpeed")]
    pub link_speed: i64,
    #[serde(rename = "ip")]
    pub ip: String,
    #[serde(rename = "serverId")]
    pub server_id: i64,
}

impl Short for Vec<Server> {
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    fn short(&self) -> Result<String, Self::Error> {
        let mut rows = vec![Row::new(vec![
            Cell::new("Identifier"),
            Cell::new("Name"),
            Cell::new("Ip"),
            Cell::new("State"),
            Cell::new("Reverse"),
        ])];

        for server in self {
            rows.push(Row::new(vec![
                Cell::new(&format!("{}", server.server_id)),
                Cell::new(&server.name),
                Cell::new(&server.ip),
                Cell::new(&server.state),
                Cell::new(&server.reverse),
            ]));
        }

        Ok(format!("{}", Table::init(rows)))
    }
}

impl Wide for Vec<Server> {
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    fn wide(&self) -> Result<String, Self::Error> {
        let mut rows = vec![Row::new(vec![
            Cell::new("Identifier"),
            Cell::new("Name"),
            Cell::new("Ip"),
            Cell::new("State"),
            Cell::new("Reverse"),
            Cell::new("Monitoring"),
            Cell::new("OS"),
            Cell::new("Data center"),
            Cell::new("Rack"),
            Cell::new("Link speed"),
        ])];

        for server in self {
            rows.push(Row::new(vec![
                Cell::new(&format!("{}", server.server_id)),
                Cell::new(&server.name),
                Cell::new(&server.ip),
                Cell::new(&server.state),
                Cell::new(&server.reverse),
                Cell::new(&format!("{}", server.monitoring)),
                Cell::new(&server.os),
                Cell::new(&server.data_center),
                Cell::new(&server.rack),
                Cell::new(&format!("{}", server.link_speed)),
            ]));
        }

        Ok(format!("{}", Table::init(rows)))
    }
}

#[tracing::instrument(skip(client))]
pub async fn list_servers(client: &Client) -> types::Result<Vec<Server>> {
    let ids: Vec<String> = client
        .get("dedicated/server")
        .await
        .map_err(|err| format!("could not retrieve list of server, {}", err))?;

    let mut servers = vec![];
    for id in ids {
        servers.push(
            client
                .get(&format!("dedicated/server/{}", id))
                .await
                .map_err(|err| format!("could not retrieve server '{}', {}", id, err))?,
        );
    }

    Ok(servers)
}
