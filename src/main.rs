use rusoto_core::Region;
use rusoto_route53::{Route53, Route53Client};
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use trust_dns_resolver::{
    config::{NameServerConfig, Protocol, ResolverConfig},
    Resolver,
};

const DEFAULT_OPENDNS_IP: Ipv4Addr = Ipv4Addr::new(208, 67, 222, 222);

mod config {
    use serde::Deserialize;

    #[derive(Clone, Copy, Debug, Deserialize)]
    pub struct Config<'a> {
        pub domain: &'a str,
        pub interval_seconds: Option<u64>,
        pub provider: &'a str,
        pub aws: Option<Aws<'a>>,
    }

    #[derive(Clone, Copy, Debug, Deserialize)]
    pub struct Aws<'a> {
        pub hosted_zone_id: &'a str,
        pub ttl: Option<i64>,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let f = std::fs::read_to_string("addns.toml")?;
    let c: config::Config = toml::from_str(&f)?;

    let r = Resolver::default()?;
    let opendns_ip = r.ipv4_lookup("resolver1.opendns.com")?;
    let opendns_ip = opendns_ip.iter().next().unwrap_or(&DEFAULT_OPENDNS_IP);

    let mut rconf = ResolverConfig::new();
    rconf.add_name_server(NameServerConfig {
        socket_addr: SocketAddr::new(IpAddr::from(opendns_ip.clone()), 53),
        protocol: Protocol::Udp,
        tls_dns_name: None,
    });
    let r = Resolver::new(rconf, trust_dns_resolver::config::ResolverOpts::default())?;

    let domain_ip = r.ipv4_lookup(c.domain)?;
    let domain_ip = domain_ip.iter().next().unwrap();
    let interval_seconds = c.interval_seconds.unwrap_or(3600);

    println!("Checking IP address every {} seconds", &interval_seconds);

    let updater: Box<dyn UpdateRecord> = match c.provider {
        "aws" => {
            let provider_config = match c.aws {
                Some(c) => c,
                None => {
                    eprintln!("Missing provider specific configuration");
                    std::process::exit(1);
                }
            };
            Box::new(Aws::new(
                c.domain,
                provider_config.hosted_zone_id,
                provider_config.ttl.unwrap_or(300),
            ))
        }
        _ => {
            eprintln!("Unsupported DNS provider: {}", c.provider);
            std::process::exit(1);
        }
    };

    loop {
        let machine_ip = r.ipv4_lookup("myip.opendns.com")?;
        let machine_ip = machine_ip.iter().next().unwrap();

        if machine_ip != domain_ip {
            println!("Updating {} from {} to {}", c.domain, domain_ip, machine_ip);
            updater.update_dns_record(machine_ip)?
        } else {
            println!("Nothing to do");
        }

        std::thread::sleep(std::time::Duration::from_secs(interval_seconds));
    }
}

struct Aws {
    client: Route53Client,
    domain: String,
    hosted_zone_id: String,
    ttl: i64,
}

impl Aws {
    pub fn new(domain: impl ToString, hosted_zone_id: impl ToString, ttl: i64) -> Self {
        Self {
            client: Route53Client::new(Region::default()),
            domain: domain.to_string(),
            hosted_zone_id: hosted_zone_id.to_string(),
            ttl,
        }
    }
}

impl UpdateRecord for Aws {
    fn update_dns_record(&self, ip: &Ipv4Addr) -> Result<(), Box<dyn Error>> {
        let res = self
            .client
            .change_resource_record_sets(rusoto_route53::ChangeResourceRecordSetsRequest {
                change_batch: rusoto_route53::ChangeBatch {
                    changes: vec![rusoto_route53::Change {
                        action: String::from("UPSERT"),
                        resource_record_set: rusoto_route53::ResourceRecordSet {
                            alias_target: None,
                            failover: None,
                            geo_location: None,
                            health_check_id: None,
                            multi_value_answer: None,
                            name: self.domain.clone(),
                            region: None,
                            resource_records: Some(vec![rusoto_route53::ResourceRecord {
                                value: format!("{}", ip),
                            }]),
                            set_identifier: None,
                            ttl: Some(self.ttl),
                            traffic_policy_instance_id: None,
                            type_: String::from("A"),
                            weight: None,
                        },
                    }],
                    // TODO add a comment
                    comment: None,
                },
                hosted_zone_id: self.hosted_zone_id.clone(),
            })
            .sync();

        match res {
            Ok(_) => println!("Updated successfully"),
            Err(e) => eprintln!("Failed to update: {}", e),
        };

        Ok(())
    }
}

trait UpdateRecord {
    fn update_dns_record(&self, ip: &Ipv4Addr) -> Result<(), Box<dyn Error>>;
}
