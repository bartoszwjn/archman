//! ArchMan --- a configuration utility for my specific Arch Linux setup.
//!
//! Documentation for specific modules provides more information.

#![warn(missing_debug_implementations, rust_2018_idioms)]

#[macro_use]
mod util;

mod args;
mod completions;
mod config;
mod link;
mod packages;
mod pacman;
mod service;
mod show;
mod sync;

pub use args::Args;

use args::Subcommand;
use config::Config;

/// Runs the program, given the parsed command line arguments.
pub fn run(args: Args) -> anyhow::Result<()> {
    let config = Config::read_from_file(args.common)?;

    match args.subcommand {
        Subcommand::Completions(completions_args) => {
            completions::generate_completions(completions_args)
        }
        Subcommand::Copy(copy_args) => {
            link::create_copies(copy_args, config);
            Ok(())
        }
        Subcommand::Link(link_args) => {
            link::create_links(link_args, config);
            Ok(())
        }
        Subcommand::Service(service_args) => service::synchronize_services(service_args, config),
        Subcommand::Show(show_args) => show::show_packages(show_args, config),
        Subcommand::Sync(sync_args) => sync::synchronize_packages(sync_args, config),
    }
}
