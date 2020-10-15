//! # Authentication module
//!
//! This module provide structure to interact with the authentication api
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Rule {
    #[serde(rename = "method")]
    pub method: String,
    #[serde(rename = "path")]
    pub path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Credential {
    #[serde(rename = "accessRules")]
    pub access_rules: Vec<Rule>,
    #[serde(rename = "redirection")]
    pub redirection: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CredentialValidation {
    #[serde(rename = "validationUrl")]
    pub validation_url: String,
    #[serde(rename = "consumerKey")]
    pub consumer_key: String,
    #[serde(rename = "state")]
    pub state: String,
}
