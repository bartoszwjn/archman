//! Synchronizing installed packages.

use std::{
    cmp::Ordering,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::Context;

use crate::{
    args::Pkg,
    pacman::{self, InstallReason, Origin, Package, PacmanError, QueryFilter},
};

pub fn synchronize_packages(args: Pkg) -> anyhow::Result<()> {
    let declared = parse_packages_file(&args.package_list)?;
    println!("Found {} declared packages", declared.len());

    let filter = QueryFilter {
        install_reason: InstallReason::Explicit,
        origin: Origin::Native,
        ..QueryFilter::default()
    };
    let installed = pacman::query(filter)?;
    println!(
        "Found {} explicitly installed, native packages",
        installed.len()
    );

    let (to_install, to_remove) = packages_diff(&declared, &installed);

    if to_remove.is_empty() {
        println!("No packages need to be removed");
    } else {
        println!("Packages to be removed: {}", to_remove.len());
        for package in &to_remove {
            println!("  {}", package);
        }
        match pacman::remove(&to_remove, true) {
            Ok(()) => {}
            Err(PacmanError::ExitFailure) => {
                println!("pacman did not exit successfully, continuing...");
            }
            Err(err) => Err(err).context("Failed to remove undeclared packages")?,
        }
    }

    if to_install.is_empty() {
        println!("No packages need to be installed");
    } else {
        println!("Packages to be installed: {}", to_install.len());
        for package in &to_install {
            println!("  {}", package);
        }
        match pacman::install(&to_install) {
            Ok(()) => {}
            Err(PacmanError::ExitFailure) => {
                println!("pacman did not exit successfully, continuing...");
            }
            Err(err) => Err(err).context("Failed to install new packages")?,
        }
    }

    Ok(())
}

fn parse_packages_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<String>> {
    let file = BufReader::new(File::open(path).context("Failed to open packages file")?);

    let mut packages = vec![];
    for line in file.lines() {
        let line = line.context("Failed to parse packages file")?;
        let without_comment = match line.find('#') {
            Some(ix) => &line[..ix],
            None => &line,
        };
        packages.extend(without_comment.split_whitespace().map(ToOwned::to_owned))
    }
    packages.sort();

    Ok(packages)
}

fn packages_diff<'a>(
    declared: &'a [String],
    installed: &'a [Package],
) -> (Vec<&'a str>, Vec<&'a str>) {
    let mut declared = declared.into_iter();
    let mut installed = installed.into_iter();
    let mut next_declared = declared.next();
    let mut next_installed = installed.next();
    let mut to_install = vec![];
    let mut to_remove = vec![];

    loop {
        match (next_declared, next_installed) {
            (None, None) => break,
            (None, Some(inst)) => {
                to_remove.push(inst.name.as_str());
                next_installed = installed.next();
            }
            (Some(decl), None) => {
                to_install.push(decl.as_str());
                next_declared = declared.next();
            }
            (Some(decl), Some(inst)) => {
                let decl = decl.as_str();
                let inst = inst.name.as_str();
                match decl.cmp(inst) {
                    Ordering::Less => {
                        to_install.push(decl);
                        next_declared = declared.next();
                    }
                    Ordering::Equal => {
                        next_declared = declared.next();
                        next_installed = installed.next();
                    }
                    Ordering::Greater => {
                        to_remove.push(inst);
                        next_installed = installed.next();
                    }
                }
            }
        }
    }

    (to_install, to_remove)
}
