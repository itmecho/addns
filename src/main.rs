#[macro_use]
extern crate log;

use anyhow::{anyhow, Context, Result};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use structopt::{clap::crate_version, StructOpt};
use trust_dns_resolver::{
    config::{NameServerConfig, Protocol, ResolverConfig},
    AsyncResolver,
};

use ddns::config::{Config, ProviderType};
use ddns::provider::{Aws, Provider};

const DEFAULT_OPENDNS_IP: Ipv4Addr = Ipv4Addr::new(208, 67, 222, 222);

#[derive(Debug, StructOpt)]
#[structopt(name = "ddns", about = "Multi-provider dynamic DNS")]
struct Opts {
    #[structopt(
        long = "log-level",
        short = "l",
        default_value = "info",
        env = "LOG_LEVEL",
        help = "Enables different levels of log messages"
    )]
    log_level: String,

    #[structopt(
        long = "config-file",
        short = "c",
        default_value = "ddns.toml",
        env = "CONFIG_FILE",
        help = "Path to the config file"
    )]
    config_file: String,

    #[structopt(long = "once", short = "o", help = "Run ddns once and exit")]
    once: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::from_args();

    std::env::set_var("LOG_LEVEL", opts.log_level);

    env_logger::init_from_env("LOG_LEVEL");

    info!("Starting ddns {}", crate_version!());

    info!("Loading configuration from {}", opts.config_file);
    let f =
        std::fs::read_to_string(opts.config_file).context("Failed to read configuration file")?;
    debug!("Raw config: {}", f);

    let c: Config = toml::from_str(&f).context("Failed to parse configuration file")?;
    debug!("Decoded config: {:?}", c);

    info!("Fetching OpenDNS IP address");
    let r = AsyncResolver::tokio_from_system_conf().await?;
    let opendns_ip = r.ipv4_lookup("resolver1.opendns.com").await?;
    let opendns_ip = opendns_ip.into_iter().next().unwrap_or(DEFAULT_OPENDNS_IP);

    info!("Using OpenDNS IP {}", opendns_ip);

    let mut rconf = ResolverConfig::new();
    rconf.add_name_server(NameServerConfig {
        socket_addr: SocketAddr::new(IpAddr::from(opendns_ip), 53),
        protocol: Protocol::Udp,
        tls_dns_name: None,
    });
    let r =
        AsyncResolver::tokio(rconf, trust_dns_resolver::config::ResolverOpts::default()).await?;

    info!("Generating update tasks");
    let tasks: Vec<(&String, Box<dyn Provider>)> = c
        .entries
        .iter()
        .map(|e| {
            let domain = &e.domain;
            let provider: Box<dyn Provider> = match &e.provider {
                ProviderType::Aws {
                    hosted_zone_id,
                    ttl,
                } => Box::new(Aws::new(domain, &hosted_zone_id, ttl.unwrap_or(300))),
            };

            (domain, provider)
        })
        .collect();

    info!(
        "Checking IP address drift every {} seconds",
        c.global.interval_seconds
    );

    loop {
        debug!("Fetching current machine's IP address");
        let machine_ip = r
            .ipv4_lookup("myip.opendns.com")
            .await
            .context("Failed to query machine's IP address")?;
        let machine_ip = machine_ip
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Failed to extract machine's IP address"))?;

        for (domain, provider) in tasks.iter() {
            debug!("Checking IP for {}", domain);
            let domain_ip = provider
                .get_current()
                .await
                .context(format!("Failed to check IP for {}", domain))?;

            debug!(
                "{}, current::{}, machine::{}",
                domain, domain_ip, machine_ip
            );
            if machine_ip != domain_ip {
                info!("Updating {} from {} to {}", domain, domain_ip, machine_ip);
                match provider.update_dns_record(&machine_ip).await {
                    Ok(_) => info!(
                        "Successfully updated {} from {} to {}",
                        domain, domain_ip, machine_ip
                    ),
                    Err(e) => error!("Failed to update record for {}: {}", domain, e),
                }
            } else {
                info!("{} is up to date", domain);
            };
        }

        if opts.once {
            return Ok(());
        }

        debug!("Sleeping for {} seconds", c.global.interval_seconds);
        std::thread::sleep(std::time::Duration::from_secs(c.global.interval_seconds));
    }
}
