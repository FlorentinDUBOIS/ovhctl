//! # Format module
//!
//! This module provide utilities to format command line output
use std::error::Error;
use std::str::FromStr;

use serde::Serialize;

#[derive(Clone, Debug)]
pub enum Kind {
    Short,
    Wide,
    Json,
    Yaml,
}

impl FromStr for Kind {
    type Err = Box<dyn Error + Send + Sync>;

    #[tracing::instrument]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "short" => Ok(Self::Short),
            "wide" => Ok(Self::Wide),
            "json" => Ok(Self::Json),
            "yaml" => Ok(Self::Yaml),
            _ => Err(format!(
                "'{}' is not allowed, only 'short', 'wide', 'json' or 'yaml",
                s
            )
            .into()),
        }
    }
}

pub trait Json {
    type Error;

    fn json(&self) -> Result<String, Self::Error>;
}

pub trait Yaml {
    type Error;

    fn yaml(&self) -> Result<String, Self::Error>;
}

pub trait Short {
    type Error;

    fn short(&self) -> Result<String, Self::Error>;
}

pub trait Wide {
    type Error;

    fn wide(&self) -> Result<String, Self::Error>;
}

pub struct Formatter<T>
where
    T: Sized + Serialize + Short + Wide,
{
    inner: T,
}

impl<T> From<T> for Formatter<T>
where
    T: Sized + Serialize + Short + Wide,
{
    #[tracing::instrument(skip(inner))]
    fn from(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> Json for Formatter<T>
where
    T: Sized + Serialize + Short + Wide,
{
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument(skip(self))]
    fn json(&self) -> Result<String, Self::Error> {
        Ok(serde_json::to_string_pretty(&self.inner)
            .map_err(|err| format!("could not serialize in json, {}", err))?)
    }
}

impl<T> Yaml for Formatter<T>
where
    T: Sized + Serialize + Short + Wide,
{
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument(skip(self))]
    fn yaml(&self) -> Result<String, Self::Error> {
        Ok(serde_yaml::to_string(&self.inner)
            .map_err(|err| format!("could not serialize in yaml, {}", err))?)
    }
}

impl<T> Short for Formatter<T>
where
    T: Sized + Serialize + Short + Wide,
    <T as Short>::Error: Error + Send + Sync,
{
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument(skip(self))]
    fn short(&self) -> Result<String, Self::Error> {
        Ok(self
            .inner
            .short()
            .map_err(|err| format!("could not serialize in short format, {}", err))?)
    }
}

impl<T> Wide for Formatter<T>
where
    T: Sized + Serialize + Short + Wide,
    <T as Wide>::Error: Error + Send + Sync,
{
    type Error = Box<dyn Error + Send + Sync>;

    #[tracing::instrument(skip(self))]
    fn wide(&self) -> Result<String, Self::Error> {
        Ok(self
            .inner
            .wide()
            .map_err(|err| format!("could not serialize in wide format, {}", err))?)
    }
}
