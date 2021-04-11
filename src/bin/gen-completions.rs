//! Generates the shell completions file for our application.

use std::path::PathBuf;

use structopt::{
    clap::{crate_name, Shell},
    StructOpt,
};

#[derive(Debug, StructOpt)]
#[structopt(about = concat!("Generate the shell completions file for '", crate_name!(), "'."))]
struct Args {
    /// The directory to put the output in.
    #[structopt(parse(from_os_str))]
    out_dir: PathBuf,
}

fn main() {
    let args = Args::from_args();
    let mut app = archman::config::Args::clap();
    app.gen_completions(crate_name!(), Shell::Zsh, args.out_dir);
}
