//! Generating tab-completion scripts.

use std::io;

use clap::{crate_name, CommandFactory};
use clap_complete::shells::Zsh;

use crate::args::{Args, CompletionsArgs};

/// Generates the tab-completion script for this application.
///
/// # Panics
///
/// Will panic on any kind of IO error related to writing to the provided directory. It would be
/// nicer to return an error instead, but [`clap`] doesn't let us do that.
pub fn generate_completions(args: CompletionsArgs) -> anyhow::Result<()> {
    let CompletionsArgs {} = args;
    clap_complete::generate(Zsh, &mut Args::command(), crate_name!(), &mut io::stdout());
    Ok(())
}
