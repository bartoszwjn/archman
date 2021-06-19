//! Gathering information about declared and installed packages.

use std::collections::{HashMap, HashSet};

use crate::pacman::{self, InstallReason, QueryFilter};

/// Packages currently installed on our system.
#[derive(Debug)]
pub(crate) struct InstalledPackages {
    pub(crate) explicit: HashSet<String>,
    pub(crate) dependencies: HashSet<String>,
    pub(crate) unneeded: HashSet<String>,
}

/// Packages organized by what we should do with them.
#[derive(Debug)]
pub(crate) struct OrganizedPackages<'a> {
    pub(crate) to_install: Vec<&'a str>,
    pub(crate) to_mark_as_explicit: Vec<&'a str>,
    pub(crate) to_remove: Vec<&'a str>,
    pub(crate) unneeded: Vec<&'a str>,
}

#[derive(Debug)]
pub(crate) struct MergedPackages<'a> {
    pub(crate) packages: HashSet<&'a str>,
    pub(crate) duplicates: HashMap<&'a str, &'a str>,
}

/// Queries for packages currently installed explicitly or as dependencies.
pub(crate) fn query_packages<'a>() -> anyhow::Result<InstalledPackages> {
    let explicit = pacman::query(QueryFilter {
        install_reason: Some(InstallReason::Explicit),
        ..QueryFilter::default()
    })?;
    let dependencies = pacman::query(QueryFilter {
        install_reason: Some(InstallReason::Dependency),
        ..QueryFilter::default()
    })?;
    let unneeded = pacman::query(QueryFilter {
        install_reason: Some(InstallReason::Dependency),
        unrequired: true,
        ..QueryFilter::default()
    })?;

    Ok(InstalledPackages {
        explicit,
        dependencies,
        unneeded,
    })
}

pub(crate) fn query_groups<'a>(
    groups: &HashSet<&'a str>,
) -> anyhow::Result<HashMap<String, &'a str>> {
    pacman::groups(groups.iter().copied()).map_err(Into::into)
}

/// Merges declared packages and package groups into a single set of packages.
pub(crate) fn merge_declared_packages<'a>(
    packages: &HashSet<&'a str>,
    group_packages: &'a HashMap<String, &'a str>,
) -> MergedPackages<'a> {
    let mut merged_packages: HashSet<_> = packages.iter().copied().collect();
    let mut duplicates = HashMap::new();

    for (package, group) in group_packages {
        merged_packages.insert(package.as_str());
        // TODO this might result in some false positives, since the package might be declared
        // in the common section, while the group is declared only for a specific host
        if let Some(duplicate) = packages.get(package.as_str()) {
            duplicates.entry(*duplicate).or_insert(*group);
        }
    }

    MergedPackages {
        packages: merged_packages,
        duplicates,
    }
}

/// Organizes packages based on what we should do with them.
pub(crate) fn organize_packages<'a>(
    declared: &HashSet<&'a str>,
    installed: &'a InstalledPackages,
) -> OrganizedPackages<'a> {
    let mut to_install = Vec::new();
    let mut to_mark_as_explicit = Vec::new();
    for &package in declared {
        if !installed.explicit.contains(package) {
            if installed.dependencies.contains(package) {
                to_mark_as_explicit.push(package);
            } else {
                to_install.push(package);
            }
        }
    }

    let remove_declared = |pkgs: &'a HashSet<String>| {
        pkgs.iter()
            .filter(|pkg| !declared.contains(pkg.as_str()))
            .map(String::as_str)
            .collect::<Vec<_>>()
    };
    let mut to_remove = remove_declared(&installed.explicit);
    let mut unneeded = remove_declared(&installed.unneeded);

    // sort them so that they look nicer if we print them
    to_remove.sort_unstable();
    to_install.sort_unstable();
    to_mark_as_explicit.sort_unstable();
    unneeded.sort_unstable();

    OrganizedPackages {
        to_install,
        to_mark_as_explicit,
        to_remove,
        unneeded,
    }
}
