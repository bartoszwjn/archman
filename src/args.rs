//! Command line arguments.

use std::path::PathBuf;

use structopt::StructOpt;

/// The program's command line arguments.
#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(subcommand)]
    pub(crate) subcommand: Subcommand,
    #[structopt(flatten)]
    pub(crate) common: ArgsCommon,
}

/// Options common to all subcommands.
#[derive(Debug, StructOpt)]
pub(crate) struct ArgsCommon {
    /// Path to the configuration file.
    #[structopt(short = "f", long, parse(from_os_str))]
    pub(crate) config: Option<PathBuf>,
    /// Path to the user's home directory.
    #[structopt(short = "d", long, parse(from_os_str))]
    pub(crate) home: Option<PathBuf>,
}

// TODO a better about
/// The program's subcommands.
#[derive(Debug, StructOpt)]
#[structopt(about = "A configuration utility for my specific Arch Linux setup")]
pub(crate) enum Subcommand {
    Completions(CompletionsArgs),
    Copy(CopyArgs),
    Link(LinkArgs),
    Show(ShowArgs),
    Sync(SyncArgs),
}

// TODO support other shells
/// Generate tab-completion script for zsh.
#[derive(Debug, StructOpt)]
pub(crate) struct CompletionsArgs {
    /// The directory to put the output in.
    #[structopt(parse(from_os_str))]
    pub(crate) out_dir: PathBuf,
}

/// Create copies of configuration files in declared locations.
#[derive(Debug, StructOpt)]
pub(crate) struct CopyArgs {}

/// Create links to configuration files in declared locations.
#[derive(Debug, StructOpt)]
pub(crate) struct LinkArgs {}

/// Display information about declared and currently installed packages.
#[derive(Debug, StructOpt)]
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
#[derive(Debug, StructOpt)]
pub(crate) struct SyncArgs {
    /// Remove all unneeded packages.
    #[structopt(short = "c", long)]
    pub(crate) cleanup: bool,
    /// Do not upgrade packages.
    #[structopt(long)]
    pub(crate) no_upgrade: bool,
    /// Path to the xkb types file.
    #[structopt(long, parse(from_os_str))]
    pub(crate) xkb_types: Option<PathBuf>,
}
