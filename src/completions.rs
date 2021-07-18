//! Generating tab-completion scripts.

use anyhow::Context;
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
pub fn generate_completions(args: CompletionsArgs) -> anyhow::Result<()> {
    let mut app = Args::clap();
    std::fs::create_dir_all(&args.out_dir)
        .with_context(|| format!("Failed to create {:?}", args.out_dir))?;
    app.gen_completions(crate_name!(), Shell::Zsh, &args.out_dir);
    Ok(())
}
