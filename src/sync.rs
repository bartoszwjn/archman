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
//! - mark declared packages that are installed as dependencies as explicitly installed
//! - mark explicitly installed packages that are not declared as installed as dependencies
//! - update packages and install declared packages that are not installed
//! - if doing a cleanup, recursively remove unneeded packages installed as dependencies
//! - if not doing a cleanup, recursively remove the packages that were explicitly installed before,
//!   are no longer in the list of packages and are not dependencies of other installed packages
//!
//! Bonus step:
//! - check if the xkb_types file needs to be patched (TODO: run arbitrary scripts at this point?)

use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::Command,
};

use anyhow::{anyhow, ensure, Context};
use regex::Regex;

use crate::{
    config::Sync,
    packages::{self, OrganizedPackages},
    pacman::{self, InstallReason, PacmanError},
};

/// Synchronizes installed packages with the package list.
///
/// See module documentation for the details.
pub fn synchronize_packages(cfg: Sync) -> anyhow::Result<()> {
    let declared = packages::get_declared_packages(cfg.packages, cfg.package_groups)
        .context("Failed to determine the set of declared packages")?;
    let installed =
        packages::get_installed_packages().context("Failed to query for installed packages")?;
    let organized = packages::organize_packages(&declared, &installed);

    update_database(&organized).context("Failed to update package database")?;
    update_and_install_packages(!cfg.no_upgrade, &organized.to_install)
        .context("Failed to update and install new packages")?;

    if cfg.cleanup {
        let unneeded =
            packages::get_unneeded_packages().context("Failed to query for unneeded packages")?;
        let mut unneeded = unneeded.iter().map(String::as_str).collect::<Vec<_>>();
        unneeded.sort_unstable();
        remove_packages(&unneeded).context("Failed to remove unneeded packages")?;
    } else {
        remove_packages(&organized.to_remove).context("Failed to remove unneeded packages")?;
    }

    if let Some(ref xkb_types) = cfg.xkb_types {
        patch_xkb_types(xkb_types).context("Failed to patch the xkb types file")?;
    }

    Ok(())
}

/// Updates the install reason of already installed packages.
fn update_database(organized: &OrganizedPackages<'_>) -> anyhow::Result<()> {
    if !organized.to_mark_as_explicit.is_empty() {
        info!(
            "Marking {} packages as explicitly installed",
            organized.to_mark_as_explicit.len(),
        );
        pacman::database(InstallReason::Explicit, &organized.to_mark_as_explicit)?;
    }

    if !organized.to_remove.is_empty() {
        info!(
            "Marking {} packages as installed as dependencies",
            organized.to_remove.len(),
        );
        pacman::database(InstallReason::Dependency, &organized.to_remove)?;
    }

    Ok(())
}

/// Updates installed packages and installs new ones.
fn update_and_install_packages(upgrade: bool, to_install: &[&str]) -> anyhow::Result<()> {
    info!(
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
            warn!("pacman did not exit successfully, continuing...");
            Ok(())
        }
        Err(err) => Err(err.into()),
    }
}

/// Recursively removes given packages, if they are not needed by other packages.
fn remove_packages(to_remove: &[&str]) -> anyhow::Result<()> {
    if to_remove.is_empty() {
        return Ok(());
    }

    info!("Removing {} packages", to_remove.len());
    match pacman::remove(to_remove) {
        Ok(()) => Ok(()),
        Err(PacmanError::ExitFailure) => {
            warn!("pacman did not exit successfully, continuing...");
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
