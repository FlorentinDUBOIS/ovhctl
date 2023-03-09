//! # Network module
//!
//! This module export all stuff that you could need to handle network operations
use std::net::IpAddr;

use ipnetwork::IpNetwork;

pub fn contains(cidrs: &[IpNetwork], ip: IpAddr) -> Option<IpNetwork> {
    for cidr in cidrs {
        if cidr.contains(ip) {
            return Some(cidr.to_owned());
        }
    }

    None
}
