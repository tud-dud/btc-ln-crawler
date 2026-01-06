use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
    time::{SystemTime, UNIX_EPOCH},
};

use log::{error, info};
use tonic_lnd::lnrpc::{ChannelGraph, LightningNode};

use crate::types::{Bitnode, BitnodesSnapshot, GenericNode, Graph};

pub(crate) fn find_overlapping_nodes(bitcoin: BitnodesSnapshot, lightning: ChannelGraph) -> Graph {
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
    Graph {
        timestamp,
        num_bitcoin: bitcoin_nodes.len(),
        num_lightning: lightning_nodes.len(),
        num_overlap: intersection.len(),
        addresses: intersection.into_iter().map(|a| a.addr.clone()).collect(),
    }
}

fn clean_bitnodes_snapshot(nodes: &HashMap<String, Bitnode>) -> HashSet<GenericNode> {
    let mut keep_addresses = HashSet::new();
    for addr in nodes.keys() {
        if !is_not_public_or_is_tor_address(addr) {
            keep_addresses.insert(GenericNode { addr: addr.clone() });
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
            if !is_not_public_or_is_tor_address(&addr.addr) {
                keep_addresses.insert(GenericNode {
                    addr: addr.addr.clone(),
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

// true if its a private or Tor IP address
fn is_not_public_or_is_tor_address(addr: &String) -> bool {
    let mut is_public_routable = false;
    if !addr.contains(".onion") {
        if let Ok(ip) = addr.parse::<IpAddr>() {
            match ip {
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
        } else {
            error!("Error converting  {addr} to IpAddr");
        }
    }
    is_public_routable
}
