use serde::Deserialize;
use std::path::PathBuf;
use structopt::StructOpt;
use structopt_toml::StructOptToml;

#[derive(Debug, StructOpt, StructOptToml, Deserialize)]
#[structopt(
    name = "XBI Client - channel example",
    about = "Connect the XBI client to two parachains and send some messages"
)]
pub struct Config {
    /// Activate debug mode
    // short and long flags (-d, --debug) will be deduced from the field's name
    #[structopt(short, long)]
    pub debug: bool,

    #[structopt(long = "primary_parachain_id", default_value = "2000")]
    pub primary_parachain_id: u32,

    /// The primary node host address, without protocol information
    #[structopt(long = "primary_node_host", default_value = "ws://127.0.0.1:9944")]
    pub primary_node_host: String,

    #[structopt(long = "primary_node_key")]
    pub primary_node_seed: Option<PathBuf>,

    #[structopt(long = "secondary_parachain_id", default_value = "3000")]
    pub secondary_parachain_id: u32,

    #[structopt(long = "secondary_node_host")]
    pub secondary_node_host: Option<String>,

    #[structopt(long = "secondary_node_key")]
    pub secondary_node_seed: Option<PathBuf>,

    /// A json array of additional subscriber configs
    #[structopt(short = "s", long = "subscribe", default_value = "[]")]
    pub subscribers: String,
}
