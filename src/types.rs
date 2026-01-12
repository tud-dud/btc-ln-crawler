use log::error;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

use crate::net::Asn;

#[derive(Debug, Default, Serialize)]
pub(crate) struct Graph {
    pub(crate) timestamp: u64,
    /// public nodes
    pub(crate) num_bitcoin: usize,
    /// public addresses (not nodes since LN nodes can announce multiple)
    pub(crate) num_lightning: usize,
    pub(crate) num_overlap: usize,
    pub(crate) nodes: Vec<Node>,
}

#[derive(Debug, Default, Serialize)]
pub(crate) struct Node {
    pub(crate) address: String,
    pub(crate) alias: String,
    pub(crate) asn: Asn,
    pub(crate) channels: Vec<Channel>,
}

#[derive(Debug, Default, Serialize)]
pub(crate) struct Channel {
    pub(crate) id: u64,
    /// in satoshis
    pub(crate) capacity: i64,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub(crate) struct BitnodesSnapshot {
    timestamp: usize,
    total_nodes: usize,
    latest_height: usize,
    pub(crate) nodes: HashMap<String, Bitnode>,
}

#[derive(Debug, Default)]
#[allow(unused)]
pub(crate) struct Bitnode {
    protocol_version: u32,
    user_agent: String,
    last_seen: u64,
    services: u64,
    best_height: u32,
}

#[derive(Debug, Serialize, Hash, PartialEq, Eq)]
pub(crate) struct GenericNode {
    pub(crate) addr: String,
}

#[derive(Debug, Deserialize)]
struct NodeTuple(u32, String, u64, u64, u32);

impl From<NodeTuple> for Bitnode {
    fn from(t: NodeTuple) -> Self {
        Self {
            protocol_version: t.0,
            user_agent: t.1,
            last_seen: t.2,
            services: t.3,
            best_height: t.4,
        }
    }
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct LndConfig {
    pub(crate) address: String,
    pub(crate) certificate: String,
    pub(crate) macaroon: String,
}

impl LndConfig {
    pub(crate) fn from_toml_file(path: &PathBuf) -> Option<Self> {
        if let Ok(config_str) = fs::read_to_string(path) {
            if let Ok(config) = toml::from_str::<LndConfig>(&config_str) {
                Some(config)
            } else {
                error!("Error parsing {} config file.", path.display());
                None
            }
        } else {
            error!("Error reading {}.", path.display());
            None
        }
    }
}

impl<'de> Deserialize<'de> for Bitnode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let tuple = NodeTuple::deserialize(deserializer)?;
        Ok(tuple.into())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_config() {
        let conf = r#"
          address = 'https://localhost:10009'
          certificate = 'tls.cert'
          macaroon = 'admin.macaroon'
        "#;
        let actual: LndConfig = toml::from_str(&conf).unwrap();
        let expected = LndConfig {
            address: "https://localhost:10009".to_string(),
            certificate: "tls.cert".to_string(),
            macaroon: "admin.macaroon".to_string(),
        };
        assert_eq!(actual, expected);
    }
}
