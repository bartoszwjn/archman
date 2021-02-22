//! `pacman -Q`

use std::{io::BufRead, process::Command};

use anyhow::{bail, ensure, Context};

use super::{Package, Packages, PacmanError};

#[derive(Debug, Default)]
pub struct QueryFilter {
    pub install_reason: InstallReason,
    pub origin: Origin,
    pub unrequired: bool,
    pub outdated: bool,
}

#[derive(Debug)]
pub enum InstallReason {
    Any,
    Explicit,
    Dependency,
}

#[derive(Debug)]
pub enum Origin {
    Any,
    Native,
    Foreign,
}

pub fn query(filter: QueryFilter) -> anyhow::Result<Packages> {
    let mut cmd = Command::new("pacman");
    cmd.arg("-Q");
    apply_filter(&mut cmd, filter);

    let output = cmd.output().context("Failed to run pacman")?;
    ensure!(
        output.status.success(),
        PacmanError::ExitFailure(output.stderr)
    );

    let packages = output
        .stdout
        .lines()
        .map(|line| parse_package(line.context("Failed to parse pacman output")?))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Packages(packages))
}

fn apply_filter(cmd: &mut Command, filter: QueryFilter) {
    match filter.install_reason {
        InstallReason::Any => {}
        InstallReason::Explicit => {
            cmd.arg("-e");
        }
        InstallReason::Dependency => {
            cmd.arg("-d");
        }
    }
    match filter.origin {
        Origin::Any => {}
        Origin::Native => {
            cmd.arg("-n");
        }
        Origin::Foreign => {
            cmd.arg("-m");
        }
    }
    if filter.unrequired {
        cmd.arg("-t");
    }
    if filter.outdated {
        cmd.arg("-u");
    }
}

fn parse_package(line: String) -> anyhow::Result<Package> {
    let mut words = line.split_whitespace();
    let name = match words.next() {
        Some(word) => word.to_owned(),
        None => bail!(PacmanError::OutputParseError(line)),
    };
    let version = match words.next() {
        Some(word) => word.to_owned(),
        None => bail!(PacmanError::OutputParseError(line)),
    };
    if let Some(_) = words.next() {
        bail!(PacmanError::OutputParseError(line))
    }
    Ok(Package { name, version })
}

impl Default for InstallReason {
    fn default() -> Self {
        Self::Any
    }
}

impl Default for Origin {
    fn default() -> Self {
        Self::Any
    }
}
