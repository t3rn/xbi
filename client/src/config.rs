use crate::{NodeConfig, SubscriberConfig};
use clap::Parser;
use config::{Environment, File};
use serde::Deserialize;
use serde::Serialize;
use std::env;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Activate debug mode
    pub debug: bool,

    pub nodes: Vec<NodeConfig>,

    pub subscribers: Vec<SubscriberConfig>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliConfig {
    /// Activate debug mode
    #[clap(short, long)]
    pub debug: Option<bool>,

    #[clap(short, long)]
    pub nodes: Option<Nodes>,

    #[clap(short, long)]
    pub subscribers: Option<Subscribers>,
}

impl Config {
    pub fn new() -> Self {
        let run_mode = env::var("XBI_CLIENT_RUN_MODE").unwrap_or_else(|_| "local".into());

        let s = config::Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("default"))
            // Add in the current environment file
            // Default to 'development' env
            // Note that this file is _optional_
            .add_source(File::with_name(&format!("{run_mode}")).required(false))
            // Add the testnet in
            .add_source(File::with_name(&format!("test")).required(false))
            // Add the mainnet in
            .add_source(File::with_name(&format!("main")).required(false))
            // Add in a local configuration file
            // This file shouldn't be checked in to git
            .add_source(File::with_name("patch").required(false))
            // Add in settings from the environment
            .add_source(Environment::with_prefix("XBI_CLIENT"))
            .build()
            .unwrap();

        s.try_deserialize().unwrap()
    }

    pub fn apply_cli_args(mut self) -> Self {
        let cli = CliConfig::parse();
        if let Some(debug) = cli.debug {
            self.debug = debug;
        }
        if let Some(nodes) = cli.nodes {
            self.nodes = nodes.0;
        }
        if let Some(subscribers) = cli.subscribers {
            self.subscribers = subscribers.0;
        }
        if self.debug {
            env::set_var(
                "RUST_LOG",
                // "substrate_api_client=none,xbi_client_channel_example=debug",
                "error,xbi_client=info,xbi_client::http=debug",
            );
        }
        self
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Subscribers(pub Vec<SubscriberConfig>);

impl FromStr for Subscribers {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Nodes(pub Vec<NodeConfig>);

impl FromStr for Nodes {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
