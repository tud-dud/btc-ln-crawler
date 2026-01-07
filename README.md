# btc-ln-crawler

![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

The crawler identifies public IP addresses that are present in both Bitcoin and
Lightning mainnets.
It queries [Bitnodes' HTTP API](bitnodes.io) for the latest snapshot of the
Bitcoin network.
It connects to an `lnd` node and uses the `lncli describegraph` to get a
current snapshot.
A report of the crawl including the identified addresses is output to a JSON
file.

## Requirements

1. [rustup](https://rustup.rs/)
    - assumes a C linker is already installed, e.g., `sudo apt install
      build-essential` on Ubuntu or `xcode-select --install` on MacOS.

## Usage

1. Compile
      ```bash
        cargo build --release
      ```
2. Execute

    ```bash
         btc-ln-crawler [OPTIONS] [VERBOSE]

         Arguments:
           [VERBOSE]

         Options:
           -l, --log <LOG_LEVEL>   [default: info]
           -o, --out <OUTPUT_DIR>  Path to directory where the results will be stored
           -c, --config <CONFIG>   Config file with credentials for the LND
                                   node's RPC interface [default: ./lnd.toml]
           -h, --help              Print help
           -V, --version           Print version
    ```
