use anyhow::{anyhow, Result};
use rusoto_core::Region;
use rusoto_route53::{Route53, Route53Client, TestDNSAnswerRequest};
use std::net::Ipv4Addr;

use crate::provider::Provider;
use crate::BLANK_IP;

pub struct Aws {
    client: Route53Client,
    domain: String,
    hosted_zone_id: String,
    ttl: i64,
}

impl Aws {
    pub fn new(domain: &'_ str, hosted_zone_id: &'_ str, ttl: i64) -> Self {
        Self {
            client: Route53Client::new(Region::default()),
            domain: domain.to_string(),
            hosted_zone_id: hosted_zone_id.to_string(),
            ttl,
        }
    }
}

#[async_trait::async_trait]
impl Provider for Aws {
    async fn get_current(&self) -> Result<Ipv4Addr> {
        let mut req = TestDNSAnswerRequest::default();
        req.hosted_zone_id = self.hosted_zone_id.clone();
        req.record_name = self.domain.clone();
        req.record_type = String::from("A");
        let res = self.client.test_dns_answer(req).await;

        match res {
            Ok(response) => {
                match response.record_data.len() {
                    0 => Ok(BLANK_IP.clone()),
                    1 => Ok(response.record_data[0].parse()?),
                    // TODO possibly just return the blank IP to force an update
                    _ => Err(anyhow!("record set has multiple values")),
                }
            }
            Err(e) => Err(anyhow!(e)),
        }
    }

    async fn update_dns_record(&self, ip: &Ipv4Addr) -> Result<()> {
        self.client
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
            .await
            .map(|_| ())
            .map_err(|e| anyhow!(e))
    }
}

// TODO Add proper tests
#[cfg(test)]
mod test {
    use super::*;
    use rusoto_mock::{
        MockCredentialsProvider, MockRequestDispatcher, MockResponseReader, ReadMockResponse,
    };

    #[tokio::test]
    async fn get_success() {
        let mock_client = rusoto_route53::Route53Client::new_with(
            MockRequestDispatcher::default().with_body(&MockResponseReader::read_response(
                "testdata/awsroute53",
                "get_success.xml",
            )),
            MockCredentialsProvider,
            Default::default(),
        );

        let aws = Aws {
            client: mock_client,
            domain: "test.test.net".to_string(),
            hosted_zone_id: "ABCDEFG1234567".to_string(),
            ttl: 300,
        };

        let res = aws.get_current().await;
        dbg!(&res);

        assert!(res.is_ok());
        assert_eq!(Ipv4Addr::new(1, 2, 3, 4), res.unwrap());
    }
}
