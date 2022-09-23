use serde::Deserialize;
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

    /// The primary node host address, without protocol information
    #[structopt(short = "i", long = "id", default_value = "1000")]
    pub primary_node_id: u64,

    /// The primary node host address, without protocol information
    #[structopt(short = "h", long = "host", default_value = "ws://127.0.0.1:9944")]
    pub primary_node_host: String,

    /// A json array of additional subscriber configs
    #[structopt(short = "s", long = "subscribe", default_value = "[]")]
    pub subscribers: String,
}
