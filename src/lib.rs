//! ArchMan --- a configuration utility for my specific Arch Linux setup.
//!
//! Documentation for specific modules provides more information.

#[macro_use]
mod util;

pub mod config;
mod pacman;
pub mod sync;

use config::{Args, Subcommand};

/// Runs the program, given the parsed command line arguments.
#[doc(hidden)]
pub fn run(args: Args) -> anyhow::Result<()> {
    let config = config::read_config_file(args.config)?;

    match args.subcommand {
        Subcommand::Sync(sync_args) => {
            let sync_config = config::merge_sync_config(sync_args, config)?;
            sync::synchronize_packages(sync_config)
        }
    }
}
