//! ArchMan --- a configuration utility for my specific Arch Linux setup.
//!
//! Documentation for specific modules provides more information.

#![warn(missing_debug_implementations, rust_2018_idioms)]

#[macro_use]
mod util;

pub mod config;
mod link;
mod packages;
mod pacman;
mod show;
mod sync;

use config::{Args, Config, Link, Show, Subcommand, Sync};

/// Runs the program, given the parsed command line arguments.
pub fn run(args: Args) -> anyhow::Result<()> {
    let config = Config::read_from_file(args.config)?;

    match args.subcommand {
        Subcommand::Link(link_args) => {
            let link_config = Link::new(link_args, config);
            link::create_links(link_config)
        }
        Subcommand::Show(show_args) => {
            let show_config = Show::new(show_args, config);
            show::show_packages(show_config)
        }
        Subcommand::Sync(sync_args) => {
            let sync_config = Sync::new(sync_args, config)?;
            sync::synchronize_packages(sync_config)
        }
    }
}
