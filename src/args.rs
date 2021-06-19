//! Command line arguments.

use std::{ffi::OsString, path::PathBuf};

use structopt::StructOpt;

/// The programs command line arguments.
#[derive(Clone, Debug, StructOpt)]
pub struct Args {
    #[structopt(subcommand)]
    pub(crate) subcommand: Subcommand,
    /// Path to the configuration file.
    #[structopt(short = "f", long)]
    pub(crate) config: Option<PathBuf>,
    // TODO override home directory
}

// TODO a better about
/// The programs subcommands.
#[derive(Clone, Debug, StructOpt)]
#[structopt(about = "A configuration utility for my specific Arch Linux setup")]
pub(crate) enum Subcommand {
    Copy(CopyArgs),
    Link(LinkArgs),
    Show(ShowArgs),
    Sync(SyncArgs),
}

/// Create copies of configuration files in declared locations.
#[derive(Clone, Debug, StructOpt)]
pub(crate) struct CopyArgs {}

/// Create links to configuration files in declared locations.
#[derive(Clone, Debug, StructOpt)]
pub(crate) struct LinkArgs {}

/// Display information about declared and currently installed packages.
#[derive(Clone, Debug, StructOpt)]
pub(crate) struct ShowArgs {
    /// Equivalent to specifying '-e', '-i', '-r' and '-u'.
    #[structopt(short = "a", long)]
    pub(crate) all: bool,
    /// Display all packages that are declared and installed as dependencies.
    #[structopt(short = "e", long)]
    pub(crate) to_explicit: bool,
    /// Display all packages that are declared and not installed.
    #[structopt(short = "i", long)]
    pub(crate) to_install: bool,
    /// Display all explicitly installed packages that are not declared.
    #[structopt(short = "r", long)]
    pub(crate) to_remove: bool,
    /// Display all packages installed as dependencies that are not required by any package.
    #[structopt(short = "u", long)]
    pub(crate) unneeded: bool,
}

/// Synchronize installed packages with the package list.
#[derive(Clone, Debug, StructOpt)]
pub(crate) struct SyncArgs {
    /// Remove all unneeded packages.
    #[structopt(short = "c", long)]
    pub(crate) cleanup: bool,
    /// Do not upgrade packages.
    #[structopt(long)]
    pub(crate) no_upgrade: bool,
    /// Path to the xkb types file.
    #[structopt(long, parse(from_os_str))]
    pub(crate) xkb_types: Option<OsString>,
}
