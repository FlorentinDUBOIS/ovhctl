//! # Command interface
//!
//! This module provide all stuffs to interact with the command line
use std::{convert::TryFrom, error::Error, path::PathBuf, sync::Arc};

use async_trait::async_trait;
use clap::{ArgAction, Parser, Subcommand};
use ipnetwork::IpNetwork;

use crate::cfg::Configuration;
use crate::cmd::dedicated::server;
use crate::cmd::fmt::Kind;
use crate::ovh::{auth, Client, ClientConfiguration, UnauthenticatedRestClient};

pub mod cloud;
pub mod dedicated;
pub mod domain;
pub mod fmt;
pub mod loadbalancer;

/// Manage domain zone
#[derive(Subcommand, Clone, Debug)]
pub enum DomainZone {
    /// List domain zone
    #[clap(name = "list", alias = "l")]
    List {
        /// Choose the output format
        #[clap(short = 'o', long = "output", default_value = "short")]
        output: Kind,
    },
}

#[async_trait]
impl Execute for DomainZone {
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    async fn execute(&self, config: Arc<Configuration>) -> Result<(), Self::Error> {
        match self {
            Self::List { output } => domain::list_zones(config, output).await,
        }
    }
}

/// Manage domain records
#[derive(Subcommand, Clone, Debug)]
pub enum DomainRecord {
    /// List domain records
    #[clap(name = "list", alias = "l")]
    List {
        /// Zone that contains domain records
        #[clap(name = "zone")]
        zone: String,

        /// Choose the output format
        #[clap(short = 'o', long = "output", default_value = "short")]
        output: Kind,
    },

    /// Synchronise domain records
    #[clap(name = "sync", alias = "s")]
    Sync {
        /// Zone that contains domain records
        #[clap(name = "zone")]
        zone: String,

        /// Choose the output format
        #[clap(short = 'o', long = "output", default_value = "short")]
        output: Kind,

        /// List of cidr to discard from the sync
        #[clap(short = 'n', long = "not-in-cidrs")]
        not_in_cidrs: Vec<IpNetwork>,
    },

    /// Delete domain record
    #[clap(name = "delete", alias = "d")]
    Delete {
        /// Zone that contains domain records
        #[clap(name = "zone")]
        zone: String,

        /// Zone that contains domain records
        #[clap(name = "record")]
        id: i64,
    },

    /// Refresh domain records
    #[clap(name = "refresh", alias = "r")]
    Refresh {
        /// Zone that contains domain records
        #[clap(name = "zone")]
        zone: String,
    },
}

#[async_trait]
impl Execute for DomainRecord {
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    async fn execute(&self, config: Arc<Configuration>) -> Result<(), Self::Error> {
        match self {
            Self::List { zone, output } => domain::list_records(config, zone, output).await,
            Self::Sync {
                zone,
                output,
                not_in_cidrs,
            } => domain::sync_records(config, zone, output, not_in_cidrs).await,
            Self::Refresh { zone } => domain::refresh_records(config, zone).await,
            Self::Delete { zone, id } => domain::delete_record(config, zone, id).await,
        }
    }
}

/// Manage domain across the ovh api
#[derive(Subcommand, Clone, Debug)]
pub enum Domain {
    /// Manage domain zone
    #[clap(name = "zone", alias = "z", subcommand)]
    Zone(DomainZone),

    /// Manage domain records
    #[clap(name = "record", alias = "r", subcommand)]
    Record(DomainRecord),
}

#[async_trait]
impl Execute for Domain {
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    async fn execute(&self, config: Arc<Configuration>) -> Result<(), Self::Error> {
        match self {
            Self::Zone(cmd) => cmd.execute(config).await,
            Self::Record(cmd) => cmd.execute(config).await,
        }
    }
}

#[async_trait]
pub trait Execute {
    type Error;

    async fn execute(&self, config: Arc<Configuration>) -> Result<(), Self::Error>;
}

/// Manage cloud loadbalancer
#[derive(Subcommand, Clone, Debug)]
pub enum LoadBalancer {
    /// List load balancer in tenant
    #[clap(name = "list", alias = "l")]
    List {
        /// Choose the output format
        #[clap(short = 'o', long = "output", default_value = "short")]
        output: Kind,

        /// Tenant on which we scope the search
        #[clap(name = "tenant")]
        tenant: String,
    },

    /// List load balancer in tenant
    #[clap(name = "create", alias = "c")]
    Create {
        /// Choose the output format
        #[clap(short = 'o', long = "output", default_value = "short")]
        output: Kind,

        /// Tenant on which we scope the search
        #[clap(name = "tenant")]
        tenant: String,

        /// Tenant on which we scope the search
        #[clap(name = "region")]
        region: String,
    },

    /// List load balancer in tenant
    #[clap(name = "delete", alias = "d")]
    Delete {
        /// Choose the output format
        #[clap(short = 'o', long = "output", default_value = "short")]
        output: Kind,

        /// Tenant on which we scope the search
        #[clap(name = "tenant")]
        tenant: String,

        /// Tenant on which we scope the search
        #[clap(name = "id")]
        id: String,
    },
}

#[async_trait]
impl Execute for LoadBalancer {
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    async fn execute(&self, config: Arc<Configuration>) -> Result<(), Self::Error> {
        match self {
            Self::List { output, tenant } => loadbalancer::list(config, output, tenant).await,
            Self::Create {
                output,
                tenant,
                region,
            } => loadbalancer::create(config, output, tenant, region).await,
            Self::Delete { output, tenant, id } => {
                loadbalancer::delete(config, output, tenant, id).await
            }
        }
    }
}

/// Manage tenants
#[derive(Subcommand, Clone, Debug)]
pub enum Tenant {
    /// List tenants
    #[clap(name = "list", alias = "l")]
    List {
        /// Choose the output format
        #[clap(short = 'o', long = "output", default_value = "short")]
        output: Kind,
    },
}

#[async_trait]
impl Execute for Tenant {
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    async fn execute(&self, config: Arc<Configuration>) -> Result<(), Self::Error> {
        match self {
            Self::List { output } => cloud::list_tenants(config, output).await,
        }
    }
}

/// Manage instances
#[derive(Subcommand, Clone, Debug)]
pub enum Instance {
    /// List instances
    #[clap(name = "list", alias = "l")]
    List {
        /// Tenant to use
        #[clap(name = "tenant")]
        tenant: String,

        /// Choose the output format
        #[clap(short = 'o', long = "output", default_value = "short")]
        output: Kind,
    },
}

#[async_trait]
impl Execute for Instance {
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    async fn execute(&self, config: Arc<Configuration>) -> Result<(), Self::Error> {
        match self {
            Self::List { tenant, output } => cloud::list_instances(config, tenant, output).await,
        }
    }
}

/// Manage cloud resources across the ovh api
#[derive(Subcommand, Clone, Debug)]
pub enum Cloud {
    /// Manage tenants
    #[clap(name = "tenant", alias = "t", subcommand)]
    Tenant(Tenant),

    /// Manage instances
    #[clap(name = "instance", alias = "i", subcommand)]
    Instance(Instance),

    /// Manage load balancer
    #[clap(name = "loadbalancer", alias = "l", subcommand)]
    LoadBalancer(LoadBalancer),
}

#[async_trait]
impl Execute for Cloud {
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    async fn execute(&self, config: Arc<Configuration>) -> Result<(), Self::Error> {
        match self {
            Self::Tenant(cmd) => cmd.execute(config).await,
            Self::Instance(cmd) => cmd.execute(config).await,
            Self::LoadBalancer(cmd) => cmd.execute(config).await,
        }
    }
}

/// Manage bare-metal servers
#[derive(Subcommand, Clone, Debug)]
pub enum Server {
    /// List servers
    #[clap(name = "list", alias = "l")]
    List {
        /// Choose the output format
        #[clap(short = 'o', long = "output", default_value = "short")]
        output: Kind,
    },
}

#[async_trait]
impl Execute for Server {
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    async fn execute(&self, config: Arc<Configuration>) -> Result<(), Self::Error> {
        match self {
            Self::List { output } => server::list_servers(config, output).await,
        }
    }
}

/// Manage dedicated infrastructure
#[derive(Subcommand, Clone, Debug)]
pub enum Dedicated {
    /// Manage bare-metal servers
    #[clap(name = "server", alias = "s", subcommand)]
    Server(Server),
}

#[async_trait]
impl Execute for Dedicated {
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    async fn execute(&self, config: Arc<Configuration>) -> Result<(), Self::Error> {
        match self {
            Self::Server(cmd) => cmd.execute(config).await,
        }
    }
}

/// Commands parsed from the command line
#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    /// Manage dedicated infrastructure
    #[clap(name = "dedicated", alias = "de", subcommand)]
    Dedicated(Dedicated),

    /// Manage domain across the ovh api
    #[clap(name = "domain", alias = "do", subcommand)]
    Domain(Domain),

    /// Manage cloud resources across the ovh api
    #[clap(name = "cloud", alias = "c", subcommand)]
    Cloud(Cloud),

    /// Login to the ovh api
    #[clap(name = "connect")]
    Connect,
}

#[async_trait]
impl Execute for Command {
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    async fn execute(&self, config: Arc<Configuration>) -> Result<(), Self::Error> {
        match self {
            Self::Dedicated(cmd) => cmd.execute(config).await,
            Self::Domain(cmd) => cmd.execute(config).await,
            Self::Cloud(cmd) => cmd.execute(config).await,
            Self::Connect => connect(config).await,
        }
    }
}

#[tracing::instrument]
async fn connect(config: Arc<Configuration>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = Client::from(ClientConfiguration::try_from(config).map_err(|err| {
        format!(
            "could not create ovh client configuration from the current configuration, {}",
            err
        )
    })?);

    let credentials: auth::CredentialValidation = client.post_unauthenticated(
        "auth/credential",
        &auth::Credential {
            access_rules: vec![
                auth::Rule {
                    method: "GET".into(),
                    path: "/*".into(),
                },
                auth::Rule {
                    method: "POST".into(),
                    path: "/*".into(),
                },
                auth::Rule {
                    method: "PUT".into(),
                    path: "/*".into(),
                },
                auth::Rule {
                    method: "DELETE".into(),
                    path: "/*".into(),
                },
            ],
            redirection: "https://upload.wikimedia.org/wikipedia/commons/thumb/f/f3/Emojione_1F4AA.svg/768px-Emojione_1F4AA.svg.png".into(),
        },
    ).await?;

    println!(
        "Please login on this url '{}' before going further",
        credentials.validation_url
    );
    println!(
        "Then, please add the following credentials '{}' as consumer key in configuration",
        credentials.consumer_key
    );

    Ok(())
}

/// Arguments parsed from the command line
#[derive(Parser, Clone, Debug)]
#[clap(author, version, about)]
pub struct Args {
    /// Increase log verbosity
    #[clap(short = 'v', global = true, action = ArgAction::Count)]
    pub verbose: usize,

    /// Validate the configuration
    #[clap(short = 't')]
    pub check: bool,

    /// Path to the configuration file
    #[clap(short = 'c', global = true, long = "config")]
    pub config: Option<PathBuf>,

    #[clap(subcommand)]
    pub cmd: Option<Command>,
}

impl paw::ParseArgs for Args {
    type Error = std::io::Error;

    fn parse_args() -> Result<Self, Self::Error> {
        Ok(Self::parse())
    }
}
