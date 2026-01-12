use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, SocketAddr},
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use log::{debug, error, info};
use tonic_lnd::lnrpc::{ChannelEdge, ChannelGraph, LightningNode};

use crate::{
    net::DbReader,
    rpc::nodeinfo,
    types::{Bitnode, BitnodesSnapshot, Channel, GenericNode, Graph, LndConfig, Node},
};

pub(crate) async fn find_overlapping_nodes(
    bitcoin: BitnodesSnapshot,
    lightning: ChannelGraph,
    lnd_config: LndConfig,
) -> Graph {
    let bitcoin_nodes = clean_bitnodes_snapshot(&bitcoin.nodes);
    let lightning_nodes = clean_lightning_snapshot(&lightning.nodes);
    let intersection: HashSet<_> = bitcoin_nodes.intersection(&lightning_nodes).collect();
    info!(
        "Got intersection of {} addresses in both networks",
        intersection.len()
    );
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let db_reader = DbReader::new();
    let mut nodes: Vec<Node> = vec![];
    for i in &intersection {
        let mut node = Node::default();
        if let Some(ln_node) = find_node_by_ip(&lightning.nodes, &i.addr) {
            node.address = i.addr.clone();
            node.alias = ln_node.alias.clone();
            node.asn = if let Ok(ip) = IpAddr::from_str(&node.address) {
                db_reader.lookup_asn(ip).unwrap_or(u32::MAX)
            } else {
                u32::MAX
            };
            if nodes.iter().any(|n| node.alias == n.alias) {
                debug!("Skipping {} as we have already visited it.", node.alias);
                continue;
            }
            if let Some(channels) = get_channels_by_pubkey(&ln_node.pub_key, &lnd_config).await {
                // only want real nodes
                if !channels.is_empty() {
                    for c in channels {
                        node.channels.push(Channel {
                            capacity: c.capacity,
                            id: c.channel_id,
                        });
                    }
                    nodes.push(node);
                }
            }
        }
    }
    Graph {
        timestamp,
        num_bitcoin: bitcoin_nodes.len(),
        num_lightning: lightning_nodes.len(),
        num_overlap: nodes.len(),
        nodes,
    }
}

fn find_node_by_ip(nodes: &[LightningNode], ip: &String) -> Option<LightningNode> {
    let mut ln_node = None;
    for node in nodes {
        for a in &node.addresses {
            if a.addr.contains(ip) {
                ln_node = Some(node.clone());
                break;
            }
        }
    }
    ln_node
}

async fn get_channels_by_pubkey(pubkey: &str, lnd_config: &LndConfig) -> Option<Vec<ChannelEdge>> {
    let mut channels = None;
    if let Some(nodeinfo) = nodeinfo(lnd_config.clone(), pubkey).await {
        channels = Some(nodeinfo.channels);
    }
    channels
}

fn clean_bitnodes_snapshot(nodes: &HashMap<String, Bitnode>) -> HashSet<GenericNode> {
    let mut keep_addresses = HashSet::new();
    for addr in nodes.keys() {
        if let Some(ip) = from_str_to_ip(addr)
            && !is_not_public_or_is_tor_address(ip)
        {
            keep_addresses.insert(GenericNode {
                addr: ip.to_string(),
            });
        }
    }
    info!(
        "Keeping {} out of {} Bitcoin addresses",
        keep_addresses.len(),
        nodes.len()
    );
    keep_addresses
}

fn clean_lightning_snapshot(nodes: &Vec<LightningNode>) -> HashSet<GenericNode> {
    let mut keep_addresses = HashSet::new();
    for node in nodes {
        for addr in &node.addresses {
            if let Some(ip) = from_str_to_ip(&addr.addr)
                && !is_not_public_or_is_tor_address(ip)
            {
                keep_addresses.insert(GenericNode {
                    addr: ip.to_string(),
                });
            }
        }
    }
    info!(
        "Keeping {} out of {} Lightning addresses",
        keep_addresses.len(),
        nodes
            .iter()
            .map(|a| a.addresses.iter().len())
            .sum::<usize>()
    );
    keep_addresses
}

fn from_str_to_ip(addr: &String) -> Option<IpAddr> {
    if let Ok(sock_addr) = addr.parse::<SocketAddr>() {
        Some(sock_addr.ip())
    } else {
        error!("Error converting {addr} to IpAddr");
        None
    }
}

// true if its a private or Tor IP address
fn is_not_public_or_is_tor_address(addr: IpAddr) -> bool {
    let mut is_public_routable = false;
    match addr {
        IpAddr::V4(ipv4) => {
            if ipv4.is_unspecified()
                || ipv4.is_private()
                || ipv4.is_loopback()
                || ipv4.is_documentation()
                || ipv4.is_broadcast()
                || ipv4.is_link_local()
            {
                is_public_routable = true;
            }
        }
        IpAddr::V6(ipv6) => {
            if ipv6.is_unspecified()
                || ipv6.is_unique_local()
                || ipv6.is_loopback()
                || ipv6.is_unicast_link_local()
            {
                is_public_routable = true;
            }
        }
    }
    is_public_routable
}

#[cfg(test)]
mod tests {

    use std::net::Ipv4Addr;

    use tonic_lnd::lnrpc::NodeAddress;

    use super::*;

    #[test]
    fn parse_ip_str() {
        let tor = "fufxobnxbep2szdexwk2lxyv7qtl3ycddv7dspwjbgrsnfmzrceivnid.onion:8333".to_string();
        let actual = from_str_to_ip(&tor);
        assert!(actual.is_none());

        let ipv6 = "[2a12:8e40:5668:e40c::1]:8333".to_string();
        let actual = from_str_to_ip(&ipv6);
        assert!(actual.is_some());
        let ipv4 = "176.123.166.122:8333".to_string();
        let actual = from_str_to_ip(&ipv4);
        assert!(actual.is_some());
    }

    #[test]
    fn is_public_ip() {
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        assert!(is_not_public_or_is_tor_address(ip));
    }

    #[test]
    fn clean_btc_nodes() {
        let nodes = HashMap::from([
            ("194.230.239.52:8333".to_string(), Bitnode::default()),
            (
                "lp2cmaeso5o2ffj2xuuu7a7rcwq6dshialkx6zo3w2wx2xbz37lkg5qd.onion:8333".to_string(),
                Bitnode::default(),
            ),
            ("[2a01:4f8:231:285::2]:8333".to_string(), Bitnode::default()),
        ]);
        let expected = HashSet::from([
            GenericNode {
                addr: "194.230.239.52".to_string(),
            },
            GenericNode {
                addr: "2a01:4f8:231:285::2".to_string(),
            },
        ]);
        let actual = clean_bitnodes_snapshot(&nodes);
        assert_eq!(actual.len(), expected.len());
    }
    #[test]
    fn clean_ln_nodes() {
        let nodes = vec![
            LightningNode {
                addresses: vec![
                    NodeAddress {
                        addr: "194.230.239.52:9735".to_string(),
                        network: "tcp".to_string(),
                    },
                    NodeAddress {
                        addr: "br4uj734xva77u7yt6oevyp2ropqjl7nw2jyzeejwmd7dzlouenkfmid.onion:9735"
                            .to_string(),
                        network: "tcp".to_string(),
                    },
                ],
                ..Default::default()
            },
            LightningNode {
                addresses: vec![NodeAddress {
                    addr: "lp2cmaeso5o2ffj2xuuu7a7rcwq6dshialkx6zo3w2wx2xbz37lkg5qd.onion:8333"
                        .to_string(),
                    network: "tcp".to_string(),
                }],
                ..Default::default()
            },
            LightningNode {
                addresses: vec![NodeAddress {
                    addr: "[2a01:4f8:231:285::2]:9735".to_string(),
                    network: "tcp".to_string(),
                }],
                ..Default::default()
            },
        ];
        let expected = HashSet::from([
            GenericNode {
                addr: "194.230.239.52".to_string(),
            },
            GenericNode {
                addr: "2a01:4f8:231:285::2".to_string(),
            },
        ]);
        let actual = clean_lightning_snapshot(&nodes);
        assert_eq!(actual.len(), expected.len());
    }
}
