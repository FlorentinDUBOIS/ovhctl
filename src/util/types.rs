//! # Types module
//!
//! This module provides types aliases to improve readability
use std::error::Error;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
