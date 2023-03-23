//! Command line arguments.

use std::path::PathBuf;

use clap::Parser;

/// Trying to declaratively configure Arch Linux
#[derive(Debug, Parser)]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: Subcommand,
    #[command(flatten)]
    pub common: ArgsCommon,
}

/// Options common to all subcommands.
#[derive(Debug, Parser)]
pub struct ArgsCommon {
    /// Path to the configuration file.
    #[arg(short = 'f', long)]
    pub config: Option<PathBuf>,
    /// Path to the user's home directory.
    #[arg(short = 'd', long)]
    pub home: Option<PathBuf>,
}

#[derive(Debug, Parser)]
pub enum Subcommand {
    Completions(CompletionsArgs),
    Copy(CopyArgs),
    Link(LinkArgs),
    Service(ServiceArgs),
    Show(ShowArgs),
    Sync(SyncArgs),
}

// TODO support other shells
/// Output tab-completion script for zsh to stdout
#[derive(Debug, Parser)]
pub struct CompletionsArgs {}

/// Create copies of configuration files in declared locations.
#[derive(Debug, Parser)]
pub struct CopyArgs {
    /// Overwrite files if they already exist.
    #[arg(short, long)]
    pub force: bool,
}

/// Create links to configuration files in declared locations.
#[derive(Debug, Parser)]
pub struct LinkArgs {
    /// Overwrite link targets if they already exist.
    #[arg(short, long)]
    pub force: bool,
}

/// Enable declared systemd services.
#[derive(Debug, Parser)]
pub struct ServiceArgs {
    /// Reset the enabled/disabled status of all services to their defaults.
    #[arg(long)]
    pub reset: bool,
    /// Start the services when enabling them.
    ///
    /// Only affects declared services, has no effect on services enabled by `--reset`.
    #[arg(long)]
    pub start: bool,
}

/// Display information about declared and currently installed packages.
#[derive(Debug, Parser)]
pub struct ShowArgs {
    /// Equivalent to specifying '-e', '-i', '-r' and '-u'.
    #[arg(short = 'a', long)]
    pub all: bool,
    /// Display all packages that are declared and installed as dependencies.
    #[arg(short = 'e', long)]
    pub to_explicit: bool,
    /// Display all packages that are declared and not installed.
    #[arg(short = 'i', long)]
    pub to_install: bool,
    /// Display all explicitly installed packages that are not declared.
    #[arg(short = 'r', long)]
    pub to_remove: bool,
    /// Display all packages installed as dependencies that are not required by any package.
    #[arg(short = 'u', long)]
    pub unneeded: bool,
}

/// Synchronize installed packages with the package list.
#[derive(Debug, Parser)]
pub struct SyncArgs {
    /// Remove all unneeded packages.
    #[arg(short = 'c', long)]
    pub cleanup: bool,
    /// Do not upgrade packages.
    #[arg(long)]
    pub no_upgrade: bool,
    /// Path to the xkb types file.
    #[arg(long)]
    pub xkb_types: Option<PathBuf>,
}
