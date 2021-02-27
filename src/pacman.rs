//! Interacting with `pacman` - the Arch Linux package manager.

use std::{
    ffi::OsStr,
    io::{self, BufRead},
    ops::Deref,
    process::{Command, ExitStatus, Stdio},
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PacmanError {
    #[error("pacman did not exit successfully")]
    ExitFailure,
    #[error("Failed to parse pacman output:\n{0}")]
    OutputParseError(String),
    #[error("Failed to run pacman: {0}")]
    IO(#[from] io::Error),
}

pub fn install<'a, P, S>(packages: P) -> Result<(), PacmanError>
where
    P: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new("sudo");
    cmd.arg("pacman").arg("-S").args(packages);
    let status = run_for_status(cmd)?;
    if status.success() {
        Ok(())
    } else {
        Err(PacmanError::ExitFailure)
    }
}

pub fn remove<'a, P, S>(packages: P, recursive: bool) -> Result<(), PacmanError>
where
    P: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new("sudo");
    cmd.arg("pacman").arg("-R");
    if recursive {
        cmd.arg("-s");
    }
    cmd.args(packages);
    let status = run_for_status(cmd)?;
    if status.success() {
        Ok(())
    } else {
        Err(PacmanError::ExitFailure)
    }
}

pub fn update(upgrade: bool) -> Result<(), PacmanError> {
    let mut cmd = Command::new("sudo");
    cmd.arg("pacman").arg("-S").arg("-y");
    if upgrade {
        cmd.arg("-u");
    }
    let status = run_for_status(cmd)?;
    if status.success() {
        Ok(())
    } else {
        Err(PacmanError::ExitFailure)
    }
}

fn run_for_status(mut cmd: Command) -> Result<ExitStatus, io::Error> {
    println!("\n===== RUNNING PACMAN =====");
    let status = cmd.status();
    println!("===== END OF PACMAN OUTPUT =====\n");
    status
}

#[derive(Debug)]
pub struct Packages(Vec<Package>);

#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub version: String,
}

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

pub fn query(filter: QueryFilter) -> Result<Packages, PacmanError> {
    let mut cmd = Command::new("pacman");
    cmd.arg("-Q");
    apply_filter(&mut cmd, filter);
    cmd.stderr(Stdio::inherit());

    let output = cmd.output()?;
    if !output.status.success() {
        return Err(PacmanError::ExitFailure);
    }

    let packages = output
        .stdout
        .lines()
        .map(|line| parse_package(line?))
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

fn parse_package(line: String) -> Result<Package, PacmanError> {
    let mut words = line.split_whitespace();
    let name = match words.next() {
        Some(word) => word.to_owned(),
        None => return Err(PacmanError::OutputParseError(line)),
    };
    let version = match words.next() {
        Some(word) => word.to_owned(),
        None => return Err(PacmanError::OutputParseError(line)),
    };
    if let Some(_) = words.next() {
        return Err(PacmanError::OutputParseError(line));
    }
    Ok(Package { name, version })
}

impl Deref for Packages {
    type Target = [Package];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
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
