use crate::types::{BitnodesSnapshot, LndConfig};

use log::{error, info};
use tonic_lnd::{
    Client,
    lnrpc::{ChannelGraph, ChannelGraphRequest},
};

pub(crate) async fn get_btc_snapshot() -> Option<BitnodesSnapshot> {
    let mut json = None;
    if let Ok(body) = reqwest::get("https://bitnodes.io/api/v1/snapshots/latest/").await
        && let Ok(text) = body.text().await
    {
        json = serde_json::from_str(&text).ok()
    }
    json
}

pub(crate) async fn get_ln_snapshot(config: LndConfig) -> Option<ChannelGraph> {
    match tonic_lnd::connect(config.address, config.certificate, config.macaroon).await {
        Ok(client) => describegraph(client).await,
        Err(e) => {
            error!("Error connecting to LND: {e}");
            None
        }
    }
}

async fn describegraph(mut client: Client) -> Option<ChannelGraph> {
    match client
        .lightning()
        .describe_graph(ChannelGraphRequest {
            include_unannounced: true,
        })
        .await
    {
        Ok(response) => {
            info!("Got channel graph from LND");
            Some(response.into_inner())
        }
        Err(e) => {
            error!("Error getting ChannelGraph: {e}");
            None
        }
    }
}
