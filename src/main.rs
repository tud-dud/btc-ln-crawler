use analysis::find_overlapping_nodes;
use clap::Parser;
use log::{LevelFilter, error, info};
use rpc::{get_btc_snapshot, get_ln_snapshot};
use std::{fs::File, path::PathBuf};
use types::LndConfig;

mod analysis;
mod rpc;
mod types;

#[derive(clap::Parser)]
#[command(version, about)]
/// Simulate occupying a node's outgoing connection slots at a given rate of disconnections with a
/// given number of IP prefixes.
struct Cli {
    #[arg(long = "log", short = 'l', default_value = "info")]
    log_level: LevelFilter,
    /// Path to directory where the results will be stored
    #[arg(long = "out", short = 'o')]
    output_dir: Option<PathBuf>,
    #[arg(long = "config", short = 'c', default_value = "./lnd.toml")]
    config: PathBuf,

    verbose: bool,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let log_level = args.log_level;
    env_logger::builder().filter_level(log_level).init();

    if let Some(config) = LndConfig::from_toml_file(&args.config) {
        info!("Successfully read config.");

        let output_dir = if let Some(output_dir) = args.output_dir {
            output_dir
        } else {
            PathBuf::from("crawl-results")
        };
        if let Err(e) = std::fs::create_dir_all(&output_dir) {
            error!("Error creating output directory {e}");
        } else {
            info!("Crawl results will be written to {output_dir:#?}/ directory.");

            if let Some(ln_snapshot) = get_ln_snapshot(config).await
                && let Some(bitcoin_snapshot) = get_btc_snapshot().await
            {
                let intersection_graph = find_overlapping_nodes(bitcoin_snapshot, ln_snapshot);
                let mut path = output_dir.clone();
                path.push(format!("crawl-{}.json", intersection_graph.timestamp));
                if let Ok(f) = File::create(&path) {
                    match serde_json::to_writer_pretty(f, &intersection_graph) {
                        Ok(_) => {
                            info!("Crawl results written to {} successfully", path.display())
                        }
                        Err(e) => error!("Error {e} writing graph to {} as JSON", path.display()),
                    }
                }
            }
        }
    }
}
