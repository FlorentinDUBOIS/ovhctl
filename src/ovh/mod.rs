//! # OVHcloud module
//!
//! This module provide all necessary stuffs to communicate with https://api.ovh.com
use std::convert::TryFrom;
use std::error::Error;
use std::io::Read;
use std::str;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Buf;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use hyper::header::USER_AGENT;
use hyper::{body::aggregate, body::Body, client::HttpConnector, Method, Request};
use hyper_tls::HttpsConnector;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::cfg::{Configuration, Ovh};

pub mod auth;
pub mod cloud;
pub mod dedicated;
pub mod domain;

pub const X_OVH_APPLICATION: &str = "X-Ovh-Application";
pub const X_OVH_TIMESTAMP: &str = "X-Ovh-Timestamp";
pub const X_OVH_SIGNATURE: &str = "X-Ovh-Signature";
pub const X_OVH_CONSUMER: &str = "X-Ovh-Consumer";

#[derive(Clone, Debug)]
pub struct ClientConfiguration {
    pub endpoint: String,
    pub application_key: String,
    pub application_secret: String,
    pub consumer_key: String,
}

impl TryFrom<Ovh> for ClientConfiguration {
    type Error = Box<dyn Error + Send + Sync>;

    fn try_from(config: Ovh) -> Result<Self, Self::Error> {
        Ok(Self {
            endpoint: config.endpoint,
            application_key: config.application_key,
            application_secret: config.application_secret,
            consumer_key: config
                .consumer_key
                .ok_or_else(|| "could not retrieve consumer key".to_string())?,
        })
    }
}

impl TryFrom<Arc<Configuration>> for ClientConfiguration {
    type Error = Box<dyn Error + Send + Sync>;

    fn try_from(config: Arc<Configuration>) -> Result<Self, Self::Error> {
        Ok(Self::try_from(config.ovh.to_owned())?)
    }
}

pub struct Client {
    inner: hyper::Client<HttpsConnector<HttpConnector>, Body>,
    config: ClientConfiguration,
}

impl From<ClientConfiguration> for Client {
    fn from(config: ClientConfiguration) -> Self {
        let client = hyper::Client::builder().build(HttpsConnector::new());

        Self {
            inner: client,
            config,
        }
    }
}

#[async_trait]
pub trait RestClient {
    type Error;

    async fn get<T>(&self, path: &str) -> Result<T, Self::Error>
    where
        T: Sized + DeserializeOwned + Send + Sync;

    async fn post<T, U>(&self, path: &str, obj: &T) -> Result<U, Self::Error>
    where
        T: Sized + Serialize + Send + Sync,
        U: Sized + DeserializeOwned + Send + Sync;

    async fn put<T, U>(&self, path: &str, obj: &T) -> Result<U, Self::Error>
    where
        T: Sized + Serialize + Send + Sync,
        U: Sized + DeserializeOwned + Send + Sync;

    async fn delete(&self, path: &str) -> Result<(), Self::Error>;
}

#[async_trait]
impl RestClient for Client {
    type Error = Box<dyn Error + Send + Sync>;

    async fn get<T>(&self, path: &str) -> Result<T, Self::Error>
    where
        T: Sized + DeserializeOwned + Send + Sync,
    {
        let timestamp = chrono::offset::Utc::now().timestamp();
        let uri = format!("{}/{}", self.config.endpoint.to_owned(), path);

        let request = Request::builder()
            .header(X_OVH_APPLICATION, self.config.application_key.to_owned())
            .header(X_OVH_TIMESTAMP, format!("{}", timestamp))
            .header(X_OVH_CONSUMER, self.config.consumer_key.to_owned())
            .header(X_OVH_SIGNATURE, self.hash("GET", &uri, "", timestamp))
            .header(
                USER_AGENT,
                format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            )
            .method(Method::GET)
            .uri(&uri)
            .body(Body::empty())
            .map_err(|err| format!("could not create request, {}", err))?;

        let response = self
            .inner
            .request(request)
            .await
            .map_err(|err| format!("could not execute request, {}", err))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!(
                "could not execute the request '{}', got '{}'",
                uri,
                status.as_u16()
            )
            .into());
        }

        let body = aggregate(response)
            .await
            .map_err(|err| format!("could not aggregate payload, {}", err))?;

        Ok(serde_json::from_reader(body.reader())
            .map_err(|err| format!("could not deserialize the payload, {}", err))?)
    }

    async fn post<T, U>(&self, path: &str, obj: &T) -> Result<U, Self::Error>
    where
        T: Sized + Serialize + Send + Sync,
        U: Sized + DeserializeOwned + Send + Sync,
    {
        let timestamp = chrono::offset::Utc::now().timestamp();
        let uri = format!("{}/{}", self.config.endpoint.to_owned(), path);

        let mut body = serde_json::to_string(obj)
            .map_err(|err| format!("could not serialize given object, {}", err))?;

        let mut request_builder = Request::builder();
        if "\"\"" != &body {
            request_builder = request_builder.header("Content-Type", "application/json");
        } else {
            body = String::new();
        }

        let request = request_builder
            .header(X_OVH_APPLICATION, self.config.application_key.to_owned())
            .header(X_OVH_TIMESTAMP, format!("{}", timestamp))
            .header(X_OVH_CONSUMER, self.config.consumer_key.to_owned())
            .header(X_OVH_SIGNATURE, self.hash("POST", &uri, &body, timestamp))
            .header(
                USER_AGENT,
                format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            )
            .method(Method::POST)
            .uri(&uri)
            .body(Body::from(body))
            .map_err(|err| format!("could not create request, {}", err))?;

        let response = self
            .inner
            .request(request)
            .await
            .map_err(|err| format!("could not execute request, {}", err))?;

        let status = response.status();
        if !status.is_success() {
            let mut buf = vec![];
            aggregate(response).await?.reader().read_to_end(&mut buf)?;

            return Err(format!(
                "could not execute the request '{}', got '{}', {}",
                uri,
                status.as_u16(),
                str::from_utf8(&buf)?,
            )
            .into());
        }

        let body = aggregate(response)
            .await
            .map_err(|err| format!("could not aggregate payload, {}", err))?;

        Ok(serde_json::from_reader(body.reader())
            .map_err(|err| format!("could not deserialize the payload, {}", err))?)
    }

    async fn put<T, U>(&self, path: &str, obj: &T) -> Result<U, Self::Error>
    where
        T: Sized + Serialize + Send + Sync,
        U: Sized + DeserializeOwned + Send + Sync,
    {
        let timestamp = chrono::offset::Utc::now().timestamp();
        let uri = format!("{}/{}", self.config.endpoint.to_owned(), path);

        let mut body = serde_json::to_string(obj)
            .map_err(|err| format!("could not serialize given object, {}", err))?;

        let mut request_builder = Request::builder();
        if "\"\"" != &body {
            request_builder = request_builder.header("Content-Type", "application/json");
        } else {
            body = String::new();
        }

        let request = request_builder
            .header("Content-Type", "application/json")
            .header(X_OVH_APPLICATION, self.config.application_key.to_owned())
            .header(X_OVH_APPLICATION, self.config.application_key.to_owned())
            .header(X_OVH_TIMESTAMP, format!("{}", timestamp))
            .header(X_OVH_CONSUMER, self.config.consumer_key.to_owned())
            .header(X_OVH_SIGNATURE, self.hash("PUT", &uri, &body, timestamp))
            .header(
                USER_AGENT,
                format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            )
            .method(Method::PUT)
            .uri(&uri)
            .body(Body::from(body))
            .map_err(|err| format!("could not create request, {}", err))?;

        let response = self
            .inner
            .request(request)
            .await
            .map_err(|err| format!("could not execute request, {}", err))?;

        let status = response.status();
        if !status.is_success() {
            let mut buf = vec![];
            aggregate(response).await?.reader().read_to_end(&mut buf)?;

            return Err(format!(
                "could not execute the request '{}', got '{}', {}",
                uri,
                status.as_u16(),
                str::from_utf8(&buf)?,
            )
            .into());
        }

        let body = aggregate(response)
            .await
            .map_err(|err| format!("could not aggregate payload, {}", err))?;

        Ok(serde_json::from_reader(body.reader())
            .map_err(|err| format!("could not deserialize the payload, {}", err))?)
    }

    async fn delete(&self, path: &str) -> Result<(), Self::Error> {
        let timestamp = chrono::offset::Utc::now().timestamp();
        let uri = format!("{}/{}", self.config.endpoint.to_owned(), path);

        let request = Request::builder()
            .header(X_OVH_APPLICATION, self.config.application_key.to_owned())
            .header(X_OVH_TIMESTAMP, format!("{}", timestamp))
            .header(X_OVH_CONSUMER, self.config.consumer_key.to_owned())
            .header(X_OVH_SIGNATURE, self.hash("DELETE", &uri, "", timestamp))
            .header(
                USER_AGENT,
                format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            )
            .method(Method::DELETE)
            .uri(&uri)
            .body(Body::empty())
            .map_err(|err| format!("could not create request, {}", err))?;

        let response = self
            .inner
            .request(request)
            .await
            .map_err(|err| format!("could not execute request, {}", err))?;

        let status = response.status();
        if !status.is_success() && 404 != status.as_u16() {
            let mut buf = vec![];
            aggregate(response).await?.reader().read_to_end(&mut buf)?;

            return Err(format!(
                "could not execute the request '{}', got '{}', {}",
                uri,
                status.as_u16(),
                str::from_utf8(&buf)?
            )
            .into());
        }

        Ok(())
    }
}

#[async_trait]
pub trait UnauthenticatedRestClient {
    type Error;

    async fn get_unauthenticated<T>(&self, path: &str) -> Result<T, Self::Error>
    where
        T: Sized + DeserializeOwned + Send + Sync;

    async fn post_unauthenticated<T, U>(&self, path: &str, obj: &T) -> Result<U, Self::Error>
    where
        T: Sized + Serialize + Send + Sync,
        U: Sized + DeserializeOwned + Send + Sync;
}

#[async_trait]
impl UnauthenticatedRestClient for Client {
    type Error = Box<dyn Error + Send + Sync>;

    async fn get_unauthenticated<T>(&self, path: &str) -> Result<T, Self::Error>
    where
        T: Sized + DeserializeOwned + Send + Sync,
    {
        let uri = format!("{}/{}", self.config.endpoint.to_owned(), path);
        let request = Request::builder()
            .header(X_OVH_APPLICATION, self.config.application_key.to_owned())
            .header(
                USER_AGENT,
                format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            )
            .method(Method::GET)
            .uri(&uri)
            .body(Body::empty())
            .map_err(|err| format!("could not create request, {}", err))?;

        let response = self
            .inner
            .request(request)
            .await
            .map_err(|err| format!("could not execute request, {}", err))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!(
                "could not execute the request '{}', got '{}'",
                uri,
                status.as_u16()
            )
            .into());
        }

        let body = aggregate(response)
            .await
            .map_err(|err| format!("could not aggregate payload, {}", err))?;

        Ok(serde_json::from_reader(body.reader())
            .map_err(|err| format!("could not deserialize the payload, {}", err))?)
    }

    async fn post_unauthenticated<T, U>(&self, path: &str, obj: &T) -> Result<U, Self::Error>
    where
        T: Sized + Serialize + Send + Sync,
        U: Sized + DeserializeOwned + Send + Sync,
    {
        let uri = format!("{}/{}", self.config.endpoint.to_owned(), path);
        let body = serde_json::to_string(obj)
            .map_err(|err| format!("could not serialize given object, {}", err))?;

        let request = Request::builder()
            .header("Content-Type", "application/json")
            .header(X_OVH_APPLICATION, self.config.application_key.to_owned())
            .header(
                USER_AGENT,
                format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            )
            .method(Method::POST)
            .uri(&uri)
            .body(Body::from(body))
            .map_err(|err| format!("could not create request, {}", err))?;

        let response = self
            .inner
            .request(request)
            .await
            .map_err(|err| format!("could not execute request, {}", err))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!(
                "could not execute the request '{}', got '{}'",
                uri,
                status.as_u16()
            )
            .into());
        }

        let body = aggregate(response)
            .await
            .map_err(|err| format!("could not aggregate payload, {}", err))?;

        Ok(serde_json::from_reader(body.reader())
            .map_err(|err| format!("could not deserialize the payload, {}", err))?)
    }
}

impl Client {
    fn hash(&self, method: &str, path: &str, body: &str, timestamp: i64) -> String {
        let mut hasher = Sha1::new();

        hasher.input_str(&self.config.application_secret);
        hasher.input_str("+");
        hasher.input_str(&self.config.consumer_key);
        hasher.input_str("+");
        hasher.input_str(method);
        hasher.input_str("+");
        hasher.input_str(path);
        hasher.input_str("+");
        hasher.input_str(body);
        hasher.input_str("+");
        hasher.input_str(&format!("{}", timestamp));

        format!("$1${}", hasher.result_str())
    }
}
