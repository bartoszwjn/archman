//! Running `pacman` --- the Arch Linux package manager.
//!
//! The functions in this module run the respective `pacman` subcommands. Additional flags are given
//! based on the function arguments. Subcommands that require root privileges are run with `sudo`.

use std::{collections::HashSet, ffi::OsStr, io, process::Command};

use ansi_term::Style;
use thiserror::Error;

/// The return type of all `pacman` calls.
type Result<T, E = PacmanError> = std::result::Result<T, E>;

/// Errors that can occur when trying to run `pacman`.
#[derive(Debug, Error)]
pub enum PacmanError {
    /// `pacman` did not exit successfully.
    #[error("pacman did not exit successfully")]
    ExitFailure,
    /// `pacman` output was not valid UTF-8.
    #[error("pacman output was not valid UTF-8:\n{}", String::from_utf8_lossy(.0))]
    NonUtf8Output(Vec<u8>),
    /// A IO error occurred.
    #[error("Failed to run pacman: {0}")]
    IO(#[from] io::Error),
}

/// Filter for packages returned from a query.
#[derive(Clone, Debug, Default)]
pub struct QueryFilter {
    /// Constrain the install reason.
    pub install_reason: Option<InstallReason>,
    /// Only packages not (optionally) required by any other package.
    // TODO: ignore optional dependencies?
    pub unrequired: bool,
    /// Only outdated packages.
    pub outdated: bool,
}

/// Install reason of a package.
#[derive(Clone, Copy, Debug)]
pub enum InstallReason {
    /// Explicitly installed.
    Explicit,
    /// Installed as a dependency of another package.
    Dependency,
}

/// `pacman -D`
///
/// # Arguments
/// - `install_reason`: the install reason to be set for the specified `packages`.
/// - `packages`: packages that should have their database entries modified.
pub fn database<P, S>(install_reason: InstallReason, packages: P) -> Result<()>
where
    P: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new("sudo");
    cmd.args(&["pacman", "--color=auto", "-D"]);
    match install_reason {
        InstallReason::Explicit => cmd.arg("--asexplicit"),
        InstallReason::Dependency => cmd.arg("--asdeps"),
    };
    cmd.args(packages);

    run_for_status(cmd)
}

/// `pacman -S`
///
/// The `--refresh` (`-y`) flag is always used.
///
/// # Arguments
/// - `system_upgrade`: update outdated packages (`-u` flag).
/// - `packages`: additional packages to be installed.
pub fn sync<P, S>(system_upgrade: bool, packages: P) -> Result<()>
where
    P: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new("sudo");
    cmd.args(&["pacman", "--color=auto", "-S", "-y"]);
    if system_upgrade {
        cmd.arg("-u");
    }
    cmd.args(packages);

    run_for_status(cmd)
}

/// `pacman -R`
///
/// The `--recursive` (`-s`) and `--unneeded` (`-u`) flags are always used.
///
/// # Arguments
/// - `packages`: packages that should be removed.
pub fn remove<P, S>(packages: P) -> Result<()>
where
    P: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new("sudo");
    cmd.args(&["pacman", "--color=auto", "-R", "-s", "-u"]);
    cmd.args(packages);

    run_for_status(cmd)
}

/// Runs the given command and maps its return status to a variant of [`Result`].
///
/// The input and output streams of the command are inherited from the current process. Emits output
/// to mark the start and end of the command output.
fn run_for_status(mut cmd: Command) -> Result<()> {
    let style = Style::new().bold();
    println_styled!(style, "======== RUNNING PACMAN ========");
    let status = cmd.status();
    println_styled!(style, "===== END OF PACMAN OUTPUT =====\n");
    match status {
        Ok(exit_status) if exit_status.success() => Ok(()),
        Ok(_) => Err(PacmanError::ExitFailure),
        Err(io_err) => Err(io_err.into()),
    }
}

/// `pacman -Q`
///
/// The `--native` (`-n`) flag is always used. `stdout` is captured and parsed, `stderr` is
/// inherited from the current process.
pub fn query(filter: QueryFilter) -> Result<HashSet<String>> {
    let mut cmd = Command::new("pacman");
    cmd.args(&["-Q", "-q", "-n"]);
    if let Some(install_reason) = filter.install_reason {
        match install_reason {
            InstallReason::Explicit => cmd.arg("-e"),
            InstallReason::Dependency => cmd.arg("-d"),
        };
    }
    if filter.unrequired {
        cmd.arg("-t");
    }
    if filter.outdated {
        cmd.arg("-u");
    };

    let output = cmd.output()?;

    if output.status.success() {
        match std::str::from_utf8(&output.stdout) {
            Ok(s) => Ok(s.lines().map(String::from).collect()),
            Err(_) => Err(PacmanError::NonUtf8Output(output.stdout)),
        }
    } else {
        // `pacman` returns with an error when the query has no results. We do not want to treat it
        // as an error, so we check if there was any output. No output means that there was no real
        // error.
        if output.stdout.is_empty() && output.stderr.is_empty() {
            Ok(HashSet::new())
        } else {
            Err(PacmanError::ExitFailure)
        }
    }
}
