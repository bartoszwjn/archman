//! Command line arguments and subcommands.

use std::path::PathBuf;

use structopt::StructOpt;

/// The programs command line arguments.
#[derive(Clone, Debug, StructOpt)]
pub struct Args {
    #[structopt(subcommand)]
    pub subcommand: Subcommand,
}

// TODO a better about
/// The programs subcommands.
#[derive(Clone, Debug, StructOpt)]
#[structopt(about = "A configuration utility for my specific Arch Linux setup")]
pub enum Subcommand {
    Pkg(Pkg),
}

/// Synchronize installed packages with the package list.
#[derive(Clone, Debug, StructOpt)]
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
