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

    /// Input file
    #[structopt(short = "c", long = "config", parse(from_os_str))]
    pub config_file: PathBuf,

    // TODO: split these for node a node b, can be shared
    /// The primary node host address, without protocol information
    #[structopt(short = "h", long = "host", default_value = "127.0.0.1:9944")]
    pub node_host: String,

    /// The primary node host address, without protocol information
    #[structopt(short = "p", long = "protocol", default_value = "ws")]
    pub node_protocol: String,
}
