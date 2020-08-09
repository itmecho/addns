use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub global: Global,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
pub struct Global {
    pub interval_seconds: u64,
}

impl Default for Global {
    fn default() -> Self {
        Self {
            interval_seconds: 300,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    pub domain: String,
    pub interval_seconds: Option<u64>,
    pub provider: ProviderType,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ProviderType {
    Aws {
        hosted_zone_id: String,
        ttl: Option<i64>,
    },
}
