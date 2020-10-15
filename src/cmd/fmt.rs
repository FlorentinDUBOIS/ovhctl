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
    JSON,
    YAML,
}

impl FromStr for Kind {
    type Err = Box<dyn Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "short" => Ok(Self::Short),
            "wide" => Ok(Self::Wide),
            "json" => Ok(Self::JSON),
            "yaml" => Ok(Self::YAML),
            _ => Err(format!(
                "'{}' is not allowed, only 'short', 'wide', 'json' or 'yaml",
                s
            ).into()),
        }
    }
}

pub trait JSON {
    type Error;

    fn json(&self) -> Result<String, Self::Error>;
}

pub trait YAML {
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
    fn from(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> JSON for Formatter<T>
where
    T: Sized + Serialize + Short + Wide,
{
    type Error = Box<dyn Error + Send + Sync>;

    fn json(&self) -> Result<String, Self::Error> {
        Ok(serde_json::to_string_pretty(&self.inner)
            .map_err(|err| format!("could not serialize in json, {}", err))?)
    }
}

impl<T> YAML for Formatter<T>
where
    T: Sized + Serialize + Short + Wide,
{
    type Error = Box<dyn Error + Send + Sync>;

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

    fn wide(&self) -> Result<String, Self::Error> {
        Ok(self
            .inner
            .wide()
            .map_err(|err| format!("could not serialize in wide format, {}", err))?)
    }
}
