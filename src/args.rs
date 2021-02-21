//! Command line arguments and subcommands.

use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(about)]
pub struct Args {
    #[structopt(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(StructOpt)]
pub enum Subcommand {
    Pkg(Pkg),
}

/// Synchronize installed packages with the package list.
#[derive(StructOpt)]
pub struct Pkg {}
