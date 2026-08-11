#![deny(unsafe_code)]

use clap::Parser;
use resense::{app, cli::Cli};

fn main() {
    if let Err(error) = app::run(Cli::parse()) {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}
