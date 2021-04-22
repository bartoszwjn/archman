//! Gathering information about declared and installed packages.

use std::collections::HashSet;

use anyhow::Context;

use crate::pacman::{self, InstallReason, QueryFilter};

/// Packages currently installed on our system.
#[derive(Clone, Debug)]
pub struct InstalledPackages {
    pub explicit: HashSet<String>,
    pub dependencies: HashSet<String>,
}

/// Packages organized by what we should do with them.
#[derive(Clone, Debug)]
pub struct OrganizedPackages<'a> {
    pub to_install: Vec<&'a str>,
    pub to_mark_as_explicit: Vec<&'a str>,
    pub to_remove: Vec<&'a str>,
}

/// Merges declared packages and package groups into a single set of packages.
pub fn get_declared_packages(
    mut declared: HashSet<String>,
    groups: Vec<String>,
) -> anyhow::Result<HashSet<String>> {
    let group_packages = pacman::groups(groups)
        .context("Failed to query for packages that belong to the declared package groups")?;
    for (package, group) in group_packages {
        if let Some(duplicate) = declared.replace(package) {
            warn!(
                "Declared package {:?} is also a member of the declared group {:?}",
                duplicate, group,
            );
        }
    }

    Ok(declared)
}

/// Queries for packages currently installed explicitly or as dependencies.
pub fn get_installed_packages() -> anyhow::Result<InstalledPackages> {
    let explicit = pacman::query(QueryFilter {
        install_reason: Some(InstallReason::Explicit),
        ..QueryFilter::default()
    })?;
    let dependencies = pacman::query(QueryFilter {
        install_reason: Some(InstallReason::Dependency),
        ..QueryFilter::default()
    })?;

    Ok(InstalledPackages {
        explicit,
        dependencies,
    })
}

/// Organizes packages based on what we should do with them.
pub fn organize_packages<'a>(
    declared: &'a HashSet<String>,
    installed: &'a InstalledPackages,
) -> OrganizedPackages<'a> {
    let mut to_remove = installed
        .explicit
        .difference(declared)
        .map(String::as_str)
        .collect::<Vec<_>>();

    let mut to_install = vec![];
    let mut to_mark_as_explicit = vec![];
    for package in declared {
        if !installed.explicit.contains(package) {
            if installed.dependencies.contains(package) {
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

/// Queries for packages installed as dependencies that are not required by other packages.
pub fn get_unneeded_packages() -> anyhow::Result<HashSet<String>> {
    pacman::query(QueryFilter {
        install_reason: Some(InstallReason::Dependency),
        unrequired: true,
        ..QueryFilter::default()
    })
    .map_err(Into::into)
}
