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
//! - remove explicitly installed packages that are not declared
//! - if doing cleanup, also remove packages installed as dependencies that are not declared and
//!   not required by other packages
//!
//! Bonus step:
//! - check if the xkb_types file needs to be patched

use std::{fs, path::Path};

use anyhow::{ensure, Context};
use regex::Regex;

use crate::{
    args::SyncArgs,
    config::Config,
    packages::{self, OrganizedPackages},
    pacman::{self, InstallReason, PacmanError},
};

/// Synchronizes installed packages with the package list.
///
/// See module documentation for the details.
pub(crate) fn synchronize_packages(args: SyncArgs, cfg: Config) -> anyhow::Result<()> {
    let declared_packages = cfg.packages();
    let declared_groups = cfg.package_groups();

    let installed = packages::query_packages().context("Failed to query for installed packages")?;
    let group_packages = packages::query_groups(&declared_groups.elements)
        .context("Failed to query for packages that belong to the declared package groups")?;

    let declared = packages::merge_declared_packages(&declared_packages.elements, &group_packages);
    let organized = packages::organize_packages(&declared.packages, &installed);

    // TODO warn about duplicate packages

    update_database(&organized).context("Failed to update package database")?;
    update_and_install_packages(args.no_upgrade, &organized.to_install)
        .context("Failed to update and install new packages")?;

    if args.cleanup {
        let mut unneeded = organized.to_remove.clone();
        unneeded.extend(&organized.unneeded);
        remove_packages(&unneeded).context("Failed to remove packages")?;
    } else {
        remove_packages(&organized.to_remove).context("Failed to remove packages")?;
    }

    if let Some(xkb_types) = args.xkb_types.or_else(|| cfg.xkb_types()) {
        patch_xkb_types(&xkb_types).context("Failed to patch the xkb types file")?;
    }

    Ok(())
}

/// Updates the install reason of already installed packages.
fn update_database(organized: &OrganizedPackages<'_>) -> anyhow::Result<()> {
    if !organized.to_mark_as_explicit.is_empty() {
        colour!(
            "Marking {} {} as explicitly installed",
            organized.to_mark_as_explicit.len(),
            packages_str(organized.to_mark_as_explicit.len()),
        );
        pacman::database(InstallReason::Explicit, &organized.to_mark_as_explicit)?;
    }

    if !organized.to_remove.is_empty() {
        colour!(
            "Marking {} {} as installed as {}",
            organized.to_remove.len(),
            packages_str(organized.to_remove.len()),
            if organized.to_remove.len() == 1 {
                "dependency"
            } else {
                "dependencies"
            },
        );
        pacman::database(InstallReason::Dependency, &organized.to_remove)?;
    }

    Ok(())
}

/// Updates installed packages and installs new ones.
fn update_and_install_packages(no_upgrade: bool, to_install: &[&str]) -> anyhow::Result<()> {
    let update_str = if no_upgrade {
        "Updating package databases"
    } else {
        "Upgrading installed packages"
    };

    if !to_install.is_empty() {
        colour!(
            "{} and installing {} new {}",
            update_str,
            to_install.len(),
            packages_str(to_install.len()),
        );
    } else {
        colour!("{}", update_str);
    }

    match pacman::sync(!no_upgrade, to_install) {
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

    colour!(
        "Removing {} {}",
        to_remove.len(),
        packages_str(to_remove.len())
    );
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
        fs::write(path, &contents).with_context(|| format!("Failed to modify {:?}", path))?;
    }

    Ok(())
}

fn packages_str(count: usize) -> &'static str {
    if count == 1 {
        "package"
    } else {
        "packages"
    }
}
