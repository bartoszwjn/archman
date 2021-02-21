//! ArchMan - a configuration utility for my specific ArchLinux setup.

mod args;

pub use args::Args;

pub fn run(_args: Args) -> anyhow::Result<()> {
    println!("Hi");

    Ok(())
}
