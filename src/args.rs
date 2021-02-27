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
    /// remove all unneeded packages
    #[structopt(short = "c", long)]
    pub cleanup: bool,
    /// do not upgrade packages
    #[structopt(long)]
    pub no_upgrade: bool,
    /// path to the package list file
    #[structopt(short = "p", long, parse(from_os_str))]
    pub package_list: PathBuf,
    /// path to the xkb types file
    #[structopt(
        long,
        parse(from_os_str),
        default_value = "/usr/share/X11/xkb/types/complete"
    )]
    pub xkb_types: PathBuf,
}
