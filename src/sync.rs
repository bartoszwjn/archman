//! Synchronizing installed packages.
//!
//! The end goal is that packages declared in the configuration file are all installed, and their
//! install reason is `explicitly installed`. All packages that are not explicitly installed and are
//! not dependencies of other packages should be removed. Sometimes that might not be what we want,
//! e.g. for packages that are build dependencies of some AUR packages. One day this might be
//! addressed.
//!
//! Right now we do not concern ourselves with AUR packages.
//!
//! For now this is what we do:
//! - get the list of explicitly installed packages
//! - get the list of packages installed as dependencies
//! - mark declared packages that are installed as dependencies as explicitly installed
//! - mark explicitly installed packages that are not declared as installed as dependencies
//! - update packages and install declared packages that are not installed
//! - if doing a cleanup, get the list of packages installed as dependencies that are not needed and
//!   remove them recursively
//! - if not doing a cleanup, recursively remove the packages that were explicitly installed before,
//!   are no longer in the list of packages and are not dependencies of other installed packages
//! - check if the xkb_types file needs to be patched (TODO: run arbitrary scripts at this point?)

use std::{
    collections::HashSet,
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::Command,
};

use ansi_term::{Colour, Style};
use anyhow::{anyhow, ensure, Context};
use regex::Regex;

use crate::{
    config::Sync,
    pacman::{self, InstallReason, PacmanError, QueryFilter},
};

/// Packages organized by what we should do with them.
#[derive(Clone, Debug)]
struct OrganizedPackages<'a> {
    to_install: Vec<&'a str>,
    to_mark_as_explicit: Vec<&'a str>,
    to_remove: Vec<&'a str>,
}

/// This is the only way to create a style in a const context.
const DEFAULT_STYLE: Style = Style {
    foreground: None,
    background: None,
    is_bold: false,
    is_dimmed: false,
    is_italic: false,
    is_underline: false,
    is_blink: false,
    is_reverse: false,
    is_hidden: false,
    is_strikethrough: false,
};

/// The colour of the printed info text.
const INFO_STYLE: Style = Style {
    foreground: Some(Colour::Blue),
    is_bold: true,
    ..DEFAULT_STYLE
};

/// The colour of the printed warnings.
const WARNING_STYLE: Style = Style {
    foreground: Some(Colour::Yellow),
    is_bold: true,
    ..DEFAULT_STYLE
};

/// Synchronizes installed packages with the package list.
///
/// See module documentation for the details.
pub(crate) fn synchronize_packages(cfg: Sync) -> anyhow::Result<()> {
    let (installed_explicitly, installed_as_deps) =
        query_installed_packages().context("Failed to query for installed packages")?;
    println!(
        "Packages: {} declared, {} explicitly installed, {} installed as dependencies\n",
        cfg.packages.len(),
        installed_explicitly.len(),
        installed_as_deps.len(),
    );
    let organized = organize_packages(&cfg.packages, &installed_explicitly, &installed_as_deps);

    update_database(&organized).context("Failed to update package database")?;
    update_and_install_packages(!cfg.no_upgrade, &organized.to_install)
        .context("Failed to update and install new packages")?;

    if cfg.cleanup {
        let unneeded =
            query_unneeded_packages().context("Failed to query for unneeded packages")?;
        let mut unneeded = unneeded.iter().map(String::as_str).collect::<Vec<_>>();
        unneeded.sort_unstable();
        remove_packages(&unneeded).context("Failed to remove unneeded packages")?;
    } else {
        remove_packages(&organized.to_remove).context("Failed to remove unneeded packages")?;
    }

    patch_xkb_types(&cfg.xkb_types).context("Failed to patch the xkb types file")?;

    Ok(())
}

/// Queries for packages currently installed explicitly or as dependencies.
fn query_installed_packages() -> anyhow::Result<(HashSet<String>, HashSet<String>)> {
    let installed_explicitly = pacman::query(QueryFilter {
        install_reason: Some(InstallReason::Explicit),
        ..QueryFilter::default()
    })?;
    let installed_as_dependencies = pacman::query(QueryFilter {
        install_reason: Some(InstallReason::Dependency),
        ..QueryFilter::default()
    })?;

    Ok((installed_explicitly, installed_as_dependencies))
}

/// Organizes packages based on what we should do with them.
fn organize_packages<'a>(
    declared: &'a HashSet<String>,
    installed_explicitly: &'a HashSet<String>,
    installed_as_dependencies: &'a HashSet<String>,
) -> OrganizedPackages<'a> {
    let mut to_remove = installed_explicitly
        .difference(declared)
        .map(String::as_str)
        .collect::<Vec<_>>();

    let mut to_install = vec![];
    let mut to_mark_as_explicit = vec![];
    for package in declared {
        if !installed_explicitly.contains(package) {
            if installed_as_dependencies.contains(package) {
                to_mark_as_explicit.push(package.as_str());
            } else {
                to_install.push(package.as_str());
            }
        }
    }

    // sort them so that they look nicer if we print them
    to_remove.sort_unstable();
    to_install.sort_unstable();
    to_mark_as_explicit.sort_unstable();

    OrganizedPackages {
        to_install,
        to_mark_as_explicit,
        to_remove,
    }
}

/// Updates the install reason of already installed packages.
fn update_database(organized: &OrganizedPackages<'_>) -> anyhow::Result<()> {
    if !organized.to_mark_as_explicit.is_empty() {
        println_styled!(
            INFO_STYLE,
            "Marking {} packages as explicitly installed",
            organized.to_mark_as_explicit.len(),
        );
        pacman::database(InstallReason::Explicit, &organized.to_mark_as_explicit)?;
    }

    if !organized.to_remove.is_empty() {
        println_styled!(
            INFO_STYLE,
            "Marking {} packages as installed as dependencies",
            organized.to_remove.len(),
        );
        pacman::database(InstallReason::Dependency, &organized.to_remove)?;
    }

    Ok(())
}

/// Updates installed packages and installs new ones.
fn update_and_install_packages(upgrade: bool, to_install: &[&str]) -> anyhow::Result<()> {
    println_styled!(
        INFO_STYLE,
        "{}{}",
        if upgrade {
            "Upgrading installed packages"
        } else {
            "Updating package databases"
        },
        if !to_install.is_empty() {
            format!(" and installing {} new packages", to_install.len())
        } else {
            "".into()
        },
    );

    match pacman::sync(upgrade, to_install) {
        Ok(()) => Ok(()),
        Err(PacmanError::ExitFailure) => {
            eprintln_styled!(
                WARNING_STYLE,
                "pacman did not exit successfully, continuing...",
            );
            Ok(())
        }
        Err(err) => Err(err.into()),
    }
}

/// Queries for packages installed as dependencies that are not required by other packages.
fn query_unneeded_packages() -> anyhow::Result<HashSet<String>> {
    pacman::query(QueryFilter {
        install_reason: Some(InstallReason::Dependency),
        unrequired: true,
        ..QueryFilter::default()
    })
    .map_err(Into::into)
}

/// Recursively removes given packages, if they are not needed by other packages.
fn remove_packages(to_remove: &[&str]) -> anyhow::Result<()> {
    if to_remove.is_empty() {
        return Ok(());
    }

    println_styled!(INFO_STYLE, "Removing {} packages", to_remove.len(),);
    match pacman::remove(to_remove) {
        Ok(()) => Ok(()),
        Err(PacmanError::ExitFailure) => {
            eprintln_styled!(
                WARNING_STYLE,
                "pacman did not exit successfully, continuing...",
            );
            Ok(())
        }
        Err(err) => Err(err.into()),
    }
}

/// Includes my own xkb types in the types file, in case it was overwritten during the update.
fn patch_xkb_types(path: &Path) -> anyhow::Result<()> {
    let mut contents = fs::read_to_string(path).context("Failed to read from file")?;

    const XKB_TYPES_REGEX_STR: &str =
        r#"^default xkb_types "complete" \{\n(?:    include "[[:alnum:]]+"\n)*\};\n$"#;
    let contents_regex =
        Regex::new(XKB_TYPES_REGEX_STR).context("Failed to build a regular expression")?;
    ensure!(
        contents_regex.is_match(&contents),
        "Did not recognize the contents of the xkb types file",
    );

    if !contents.contains("include \"ed\"") {
        println!("Patching up {:?}", path);
        // regex match ensures the string contains '}'
        let last_line_start = contents.find('}').unwrap();
        contents.insert_str(last_line_start, "    include \"ed\"\n");
        write_as_root(path, &contents)?;
    }

    Ok(())
}

/// Creates a temporary file with the given contents, then moves it to the given path with `sudo`.
fn write_as_root(path: &Path, contents: &str) -> anyhow::Result<()> {
    let mut tmp_path = env::temp_dir();
    tmp_path.push("archman_xkb_types");

    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp_path)
        .context("Failed to create temporary file")?
        .write_all(contents.as_bytes())
        .context("Failed to write to temporary file")?;

    let status = Command::new("sudo")
        .arg("mv")
        .arg(&tmp_path)
        .arg(path)
        .status()
        .context("Failed to run 'mv' command")?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "'sudo mv {:?} {:?}' did not exit successfully",
            &tmp_path,
            path,
        ))
    }
}
