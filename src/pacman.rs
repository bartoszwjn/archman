//! Interacting with `pacman` - the Arch Linux package manager.

use std::ops::Deref;

use thiserror::Error;

mod query;

pub use query::{query, InstallReason, Origin, QueryFilter};

#[derive(Debug)]
pub struct Packages(Vec<Package>);

#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub version: String,
}

impl Deref for Packages {
    type Target = [Package];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum PacmanError {
    #[error("pacman did not exit successfully:\n{}", String::from_utf8_lossy(&.0))]
    ExitFailure(Vec<u8>),
    #[error("Failed to parse pacman output:\n{0}")]
    OutputParseError(String),
}
