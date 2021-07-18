//! Generating tab-completion scripts.

use structopt::{
    clap::{crate_name, Shell},
    StructOpt,
};

use crate::args::{Args, CompletionsArgs};

/// Generates the tab-completion script for this application.
///
/// # Panics
///
/// Will panic on any kind of IO error related to writing to the provided directory. It would be
/// nicer to return an error instead, but [`clap`] doesn't let us do that.
pub(crate) fn generate_completions(args: CompletionsArgs) {
    let mut app = Args::clap();
    // TODO at least try to detect some errors, the panic messages are awful
    app.gen_completions(crate_name!(), Shell::Zsh, args.out_dir);
}
