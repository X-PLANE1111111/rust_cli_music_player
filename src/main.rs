use clap::Parser;
use cli::Cli;

pub mod cli;
mod util;

fn main() {
    let mut cli = Cli::parse();
    cli.handle();
}
