use log::error;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Default, Serialize)]
pub(crate) struct Graph {
    pub(crate) timestamp: u64,
    /// public nodes
    pub(crate) num_bitcoin: usize,
    /// public addresses (not nodes since LN nodes can announce multiple)
    pub(crate) num_lightning: usize,
    pub(crate) num_overlap: usize,
    pub(crate) addresses: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BitnodesSnapshot {
    pub(crate) timestamp: usize,
    pub(crate) total_nodes: usize,
    pub(crate) latest_height: usize,
    pub(crate) nodes: HashMap<String, Bitnode>,
}

#[derive(Debug)]
pub(crate) struct Bitnode {
    pub(crate) protocol_version: u32,
    pub(crate) user_agent: String,
    pub(crate) last_seen: u64,
    pub(crate) services: u64,
    pub(crate) best_height: u32,
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
    use std::{fs::File, io::Write, str::FromStr};
    use tempfile::tempdir;

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
