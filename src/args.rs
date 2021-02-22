//! Command line arguments and subcommands.

use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about)]
pub struct Args {
    #[structopt(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(Debug, StructOpt)]
pub enum Subcommand {
    Pkg(Pkg),
}

/// Synchronize installed packages with the package list.
#[derive(Debug, StructOpt)]
pub struct Pkg {
    /// path to the package list file
    #[structopt(short = "p", long, parse(from_os_str))]
    pub package_list: PathBuf,
}
