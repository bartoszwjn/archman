//! ArchMan - a configuration utility for my specific Arch Linux setup.

pub mod args;
mod pacman;
mod pkg;

use args::{Args, Subcommand};

pub fn run(args: Args) -> anyhow::Result<()> {
    match args.subcommand {
        Subcommand::Pkg(pkg_subcommand) => pkg::synchronize_packages(pkg_subcommand)?,
    }

    Ok(())
}
