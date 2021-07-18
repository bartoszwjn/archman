//! Command line arguments.

use std::path::PathBuf;

use structopt::StructOpt;

/// The program's command line arguments.
#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(subcommand)]
    pub subcommand: Subcommand,
    #[structopt(flatten)]
    pub common: ArgsCommon,
}

/// Options common to all subcommands.
#[derive(Debug, StructOpt)]
pub struct ArgsCommon {
    /// Path to the configuration file.
    #[structopt(short = "f", long, parse(from_os_str))]
    pub config: Option<PathBuf>,
    /// Path to the user's home directory.
    #[structopt(short = "d", long, parse(from_os_str))]
    pub home: Option<PathBuf>,
}

// TODO a better about
/// The program's subcommands.
#[derive(Debug, StructOpt)]
#[structopt(about = "A configuration utility for my specific Arch Linux setup")]
pub enum Subcommand {
    Completions(CompletionsArgs),
    Copy(CopyArgs),
    Link(LinkArgs),
    Service(ServiceArgs),
    Show(ShowArgs),
    Sync(SyncArgs),
}

// TODO support other shells
/// Generate tab-completion script for zsh.
#[derive(Debug, StructOpt)]
pub struct CompletionsArgs {
    /// The directory to put the output in.
    #[structopt(parse(from_os_str))]
    pub out_dir: PathBuf,
}

/// Create copies of configuration files in declared locations.
#[derive(Debug, StructOpt)]
pub struct CopyArgs {
    /// Overwrite files if they already exist.
    #[structopt(short, long)]
    pub force: bool,
}

/// Create links to configuration files in declared locations.
#[derive(Debug, StructOpt)]
pub struct LinkArgs {
    /// Overwrite link targets if they already exist.
    #[structopt(short, long)]
    pub force: bool,
}

/// Enable declared systemd services.
#[derive(Debug, StructOpt)]
pub struct ServiceArgs {
    /// Reset the enabled/disabled status of all services to their defaults.
    #[structopt(long)]
    pub reset: bool,
    /// Start the services when enabling them.
    ///
    /// Only affects declared services, has no effect on services enabled by `--reset`.
    #[structopt(long)]
    pub start: bool,
}

/// Display information about declared and currently installed packages.
#[derive(Debug, StructOpt)]
pub struct ShowArgs {
    /// Equivalent to specifying '-e', '-i', '-r' and '-u'.
    #[structopt(short = "a", long)]
    pub all: bool,
    /// Display all packages that are declared and installed as dependencies.
    #[structopt(short = "e", long)]
    pub to_explicit: bool,
    /// Display all packages that are declared and not installed.
    #[structopt(short = "i", long)]
    pub to_install: bool,
    /// Display all explicitly installed packages that are not declared.
    #[structopt(short = "r", long)]
    pub to_remove: bool,
    /// Display all packages installed as dependencies that are not required by any package.
    #[structopt(short = "u", long)]
    pub unneeded: bool,
}

/// Synchronize installed packages with the package list.
#[derive(Debug, StructOpt)]
pub struct SyncArgs {
    /// Remove all unneeded packages.
    #[structopt(short = "c", long)]
    pub cleanup: bool,
    /// Do not upgrade packages.
    #[structopt(long)]
    pub no_upgrade: bool,
    /// Path to the xkb types file.
    #[structopt(long, parse(from_os_str))]
    pub xkb_types: Option<PathBuf>,
}
