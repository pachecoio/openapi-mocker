use std::path::PathBuf;

use clap::Parser;

pub mod server;
pub mod spec;

#[derive(Parser)]
#[clap(version = "0.1.0", author = "Thiago Pacheco")]
pub struct Args {
    #[clap(index = 1)]
    pub spec: PathBuf,
    #[clap(short, long, default_value = "8080")]
    pub port: Option<u16>,
}
