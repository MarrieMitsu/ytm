use std::path::PathBuf;

use clap::Parser;

#[derive(Clone, Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// JSON History File
    #[arg(short, long)]
    pub file: PathBuf,

    /// Which port is server running on
    #[arg(short, long, default_value_t = 8000)]
    pub port: u16,
}

impl Config {}
