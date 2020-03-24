use rusoto_core::Region;
use rusoto_route53::{Route53, Route53Client};
use serde::Deserialize;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use trust_dns_resolver::{
    config::{NameServerConfig, Protocol, ResolverConfig},
    Resolver,
};

const DEFAULT_OPENDNS_IP: Ipv4Addr = Ipv4Addr::new(208, 67, 222, 222);

#[derive(Clone, Copy, Debug, Deserialize)]
struct Config<'a> {
    domain: &'a str,
    hosted_zone: &'a str,
    interval_seconds: Option<u64>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let f = std::fs::read_to_string("addns.toml")?;
    let c: Config = toml::from_str(&f)?;

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

    let client = Route53Client::new(Region::default());

    loop {
        let machine_ip = r.ipv4_lookup("myip.opendns.com")?;
        let machine_ip = machine_ip.iter().next().unwrap();

        if machine_ip != domain_ip {
            println!("Updating {} from {} to {}", c.domain, domain_ip, machine_ip);
            // TODO load the current resource record set and use that
            let res = client
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
                                name: String::from(c.domain),
                                region: None,
                                resource_records: Some(vec![rusoto_route53::ResourceRecord {
                                    value: format!("{}", machine_ip),
                                }]),
                                set_identifier: None,
                                ttl: Some(300),
                                traffic_policy_instance_id: None,
                                type_: String::from("A"),
                                weight: None,
                            },
                        }],
                        // TODO add a comment
                        comment: None,
                    },
                    hosted_zone_id: c.hosted_zone.to_string(),
                })
                .sync();

            match res {
                Ok(_) => println!("Updated successfully"),
                Err(e) => eprintln!("Failed to update: {}", e),
            };
            println!("Need to update");
        } else {
            println!("Nothing to do");
        }

        std::thread::sleep(std::time::Duration::from_secs(interval_seconds));
    }
}
