//! ArchMan --- a configuration utility for my specific Arch Linux setup.
//!
//! Documentation for specific modules provides more information.

#![warn(missing_debug_implementations, rust_2018_idioms)]

#[macro_use]
mod util;

pub mod config;
mod packages;
mod pacman;
mod show;
mod sync;

use config::{Args, Subcommand};

/// Runs the program, given the parsed command line arguments.
pub fn run(args: Args) -> anyhow::Result<()> {
    let config = config::read_config_file(args.config)?;

    match args.subcommand {
        Subcommand::Show(show_args) => {
            let show_config = config::merge_show_config(show_args, config);
            show::show_packages(show_config)
        }
        Subcommand::Sync(sync_args) => {
            let sync_config = config::merge_sync_config(sync_args, config)?;
            sync::synchronize_packages(sync_config)
        }
    }
}
