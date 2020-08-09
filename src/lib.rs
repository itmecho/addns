pub mod config;
pub mod provider;

use std::net::Ipv4Addr;

pub(crate) const BLANK_IP: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
