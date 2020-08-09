mod aws;

pub use aws::Aws;

use anyhow::Result;
use std::net::Ipv4Addr;

#[async_trait::async_trait]
pub trait Provider {
    async fn get_current(&self) -> Result<Ipv4Addr>;

    async fn update_dns_record(&self, ip: &Ipv4Addr) -> Result<()>;
}
