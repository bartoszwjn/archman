//! Gathering information about declared and installed packages.

use std::collections::{HashMap, HashSet};

use crate::pacman::{self, InstallReason, QueryFilter};

/// Packages currently installed on our system.
#[derive(Debug)]
pub struct InstalledPackages {
    pub explicit: HashSet<String>,
    pub dependencies: HashSet<String>,
    pub unneeded: HashSet<String>,
}

/// Packages organized by what we should do with them.
#[derive(Debug)]
pub struct OrganizedPackages<'a> {
    pub to_install: Vec<&'a str>,
    pub to_mark_as_explicit: Vec<&'a str>,
    pub to_remove: Vec<&'a str>,
    pub unneeded: Vec<&'a str>,
}

#[derive(Debug)]
pub struct MergedPackages<'a> {
    pub packages: HashSet<&'a str>,
    pub duplicates: HashMap<&'a str, &'a str>,
}

/// Queries for packages currently installed explicitly or as dependencies.
pub fn query_packages() -> anyhow::Result<InstalledPackages> {
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

pub fn query_groups<'a>(
    groups: &HashSet<&'a str>,
) -> anyhow::Result<HashMap<String, &'a str>> {
    pacman::groups(groups.iter().copied()).map_err(Into::into)
}

/// Merges declared packages and package groups into a single set of packages.
pub fn merge_declared_packages<'a>(
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
pub fn organize_packages<'a>(
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
