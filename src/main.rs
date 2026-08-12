#![deny(unsafe_code)]

use clap::Parser;
use resense::{app, cli::Cli};

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    if resense::update::version_requested(&args) {
        if let Err(error) = resense::update::print_version() {
            eprintln!("error: {error:#}");
            std::process::exit(1);
        }
        return;
    }

    if let Err(error) = app::run(Cli::parse()) {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}
