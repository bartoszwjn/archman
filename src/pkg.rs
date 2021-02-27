//! Managing installed packages.
//!
//! The end goal is that packages declared in the package file are all installed, and their install
//! reason is `explicitly installed`. All packages that are not explicitly installed and are not
//! dependencies of other packages should be removed. Sometimes that might not be what we want, e.g.
//! for packages that are build dependencies of some AUR packages. One day this might be addressed.
//!
//! Right now we do not concern ourselves with AUR packages.
//!
//! For now this is what we do:
//! - read the list of packages from the package file
//! - get the list of explicitly installed packages
//! - get the list of packages installed as dependencies
//! - mark declared packages that are installed as dependencies as explicitly installed
//! - mark explicitly installed packages that are not declared as installed as dependencies
//! - update packages and install declared packages that are not installed
//! - if doing a cleanup, get the list of packages installed as dependencies that are not needed
//!   and remove them recursively
//! - if not doing a cleanup, recursively remove the packages that were explicitly installed before,
//!   are no longer in the list of packages and are not dependencies of other installed packages
//! - check if the xkb_types file needs to be patched (TODO: run arbitrary scripts at this point?)
//!
//! # Packages file
//!
//! The file should be a UTF-8 encoded text file. In the file any occurrence of the `#` character
//! starts a comment that lasts until the end of the line. After comments are ignored, the file is
//! read as a whitespace separated list of package names.

use std::{
    collections::HashSet,
    env,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
    process::Command,
};

use ansi_term::Colour;
use anyhow::{anyhow, ensure, Context};
use regex::Regex;

use crate::{
    args::Pkg,
    pacman::{self, InstallReason, PacmanError, QueryFilter},
};

/// Packages organized by what we should do with them.
#[derive(Clone, Debug)]
struct OrganizedPackages<'a> {
    to_install: Vec<&'a str>,
    to_mark_as_explicit: Vec<&'a str>,
    to_remove: Vec<&'a str>,
}

/// Synchronizes installed packages with the package list.
///
/// See module documentation for the details.
pub fn synchronize_packages(cfg: Pkg) -> anyhow::Result<()> {
    let declared =
        parse_package_file(&cfg.package_list).context("Failed to read the package list")?;
    let (installed_explicitly, installed_as_deps) =
        query_installed_packages().context("Failed to query for installed packages")?;
    println!(
        "Packages: {} declared, {} explicitly installed, {} installed as dependencies",
        declared.len(),
        installed_explicitly.len(),
        installed_as_deps.len(),
    );
    let organized = organize_packages(&declared, &installed_explicitly, &installed_as_deps);

    update_database(&organized).context("Failed to update package database")?;
    update_and_install_packages(!cfg.no_upgrade, &organized.to_install)
        .context("Failed to update and install new packages")?;

    if cfg.cleanup {
        let unneeded =
            query_unneeded_packages().context("Failed to query for unneeded packages")?;
        let mut unneeded = unneeded.iter().map(String::as_str).collect::<Vec<_>>();
        unneeded.sort();
        remove_packages(&unneeded).context("Failed to remove unneeded packages")?;
    } else {
        remove_packages(&organized.to_remove).context("Failed to remove unneeded packages")?;
    }

    patch_xkb_types(&cfg.xkb_types).context("Failed to patch the xkb types file")?;

    Ok(())
}

/// Reads the set declared packages from the given file.
///
/// See the module documentation for the description of the file format.
fn parse_package_file(path: &Path) -> anyhow::Result<HashSet<String>> {
    let file = BufReader::new(File::open(path).context("Failed to open packages file")?);

    let mut packages = HashSet::new();
    for line in file.lines() {
        let line = line.context("Failed to parse packages file")?;
        let without_comment = match line.find('#') {
            Some(ix) => &line[..ix],
            None => &line,
        };
        packages.extend(without_comment.split_whitespace().map(ToOwned::to_owned))
    }

    Ok(packages)
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
    to_remove.sort();
    to_install.sort();
    to_mark_as_explicit.sort();

    OrganizedPackages {
        to_install,
        to_mark_as_explicit,
        to_remove,
    }
}

/// Updates the install reason of already installed packages.
fn update_database(organized: &OrganizedPackages) -> anyhow::Result<()> {
    if !organized.to_mark_as_explicit.is_empty() {
        println!(
            "Marking {} packages as explicitly installed",
            organized.to_mark_as_explicit.len(),
        );
        pacman::database(InstallReason::Explicit, &organized.to_mark_as_explicit)?;
    }

    if !organized.to_remove.is_empty() {
        println!(
            "Marking {} packages as installed as dependencies",
            organized.to_remove.len(),
        );
        pacman::database(InstallReason::Dependency, &organized.to_remove)?;
    }

    Ok(())
}

/// Updates installed packages and installs new ones.
fn update_and_install_packages(upgrade: bool, to_install: &[&str]) -> anyhow::Result<()> {
    println!(
        "Updating package databases and installing {} new packages ({} system upgrade)",
        to_install.len(),
        if upgrade { "with" } else { "without" },
    );
    match pacman::sync(upgrade, to_install) {
        Ok(()) => Ok(()),
        Err(PacmanError::ExitFailure) => {
            println!(
                "{}",
                Colour::Yellow.paint("pacman did not exit successfully, continuing..."),
            );
            Ok(())
        }
        Err(err) => Err(err.into()),
    }
}

/// Queries for packages installed as dependencies that are not required by other packages.
fn query_unneeded_packages() -> anyhow::Result<HashSet<String>> {
    let unneeded = pacman::query(QueryFilter {
        install_reason: Some(InstallReason::Dependency),
        unrequired: true,
        ..QueryFilter::default()
    })?;
    println!("Found {} unneeded packages", unneeded.len());
    Ok(unneeded)
}

/// Recursively removes given packages, if they are not needed by other packages.
fn remove_packages(to_remove: &[&str]) -> anyhow::Result<()> {
    if to_remove.is_empty() {
        return Ok(());
    }
    println!("Removing {} packages", to_remove.len());
    match pacman::remove(to_remove) {
        Ok(()) => Ok(()),
        Err(PacmanError::ExitFailure) => {
            println!(
                "{}",
                Colour::Yellow.paint("pacman did not exit successfully, continuing..."),
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
        let last_line_start = contents.find("}").unwrap();
        contents.insert_str(last_line_start, "    include \"ed\"\n");
        write_as_root(path, &contents)?;
    }

    Ok(())
}

/// Creates a temporary file with the given contents, then moves it to the given path with `sudo`.
fn write_as_root(path: &Path, contents: &str) -> anyhow::Result<()> {
    let mut tmp_path = env::temp_dir();
    tmp_path.push("archman_xkb_types");

    BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)
            .context("Failed to create temporary file")?,
    )
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
