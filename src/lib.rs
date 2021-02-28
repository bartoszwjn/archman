//! ArchMan - a configuration utility for my specific Arch Linux setup.

mod config;
mod pacman;
mod pkg;

use config::{Args, Subcommand};

/// Runs the program, given the parsed command line arguments.
pub fn run(args: Args) -> anyhow::Result<()> {
    let config = config::read_config_file(args.config)?;

    match args.subcommand {
        Subcommand::Pkg(pkg_args) => {
            let pkg_config = config::merge_pkg_config(pkg_args, config.pkg)?;
            pkg::synchronize_packages(pkg_config)
        }
    }
}
