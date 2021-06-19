//! Displaying information about declared and currently installed packages.

use std::{collections::HashSet, fmt::Display};

use anyhow::Context;

use crate::{
    args::ShowArgs,
    config::Config,
    packages::{self, InstalledPackages, OrganizedPackages},
};

/// Prints out information about declared and installed packages.
pub(crate) fn show_packages(args: ShowArgs, cfg: Config) -> anyhow::Result<()> {
    let declared_packages = cfg.packages();
    let declared_groups = cfg.package_groups();

    let installed = packages::query_packages().context("Failed to query for installed packages")?;
    let group_packages = packages::query_groups(&declared_groups.elements)
        .context("Failed to query for packages that belong to the declared package groups")?;

    let declared = packages::merge_declared_packages(&declared_packages.elements, &group_packages);
    let organized = packages::organize_packages(&declared.packages, &installed);

    // TODO print warnings

    print_summary(&declared.packages, &installed, &organized);
    if args.all || args.to_install {
        print_packages("Packages to install", &organized.to_install);
    }
    if args.all || args.to_explicit {
        print_packages(
            "Packages to mark as explicitly installed",
            &organized.to_mark_as_explicit,
        );
    }
    if args.all || args.to_remove {
        print_packages("Packages to remove", &organized.to_remove);
    }
    if args.all || args.unneeded {
        print_packages("Unneeded packages", &organized.unneeded);
    }

    Ok(())
}

fn print_summary(
    declared: &HashSet<&str>,
    installed: &InstalledPackages,
    organized: &OrganizedPackages<'_>,
) {
    let summary = [
        ("declared", declared.len()),
        (
            "installed",
            installed.explicit.len() + installed.dependencies.len(),
        ),
        ("  explicitly", installed.explicit.len()),
        ("  as dependencies", installed.dependencies.len()),
        ("to install", organized.to_install.len()),
        (
            "to mark as explicitly installed",
            organized.to_mark_as_explicit.len(),
        ),
        ("to remove", organized.to_remove.len()),
        ("unneeded", organized.unneeded.len()),
    ];

    let what_width = summary.iter().map(|&(what, _)| what.len()).max().unwrap();
    let n_width = summary
        .iter()
        .map(|&(_, n)| n.to_string().len())
        .max()
        .unwrap();

    info!("Packages:");
    for &(what, n) in summary.iter() {
        println!(
            "  {what:what_width$} : {n:n_width$}",
            what = what,
            what_width = what_width,
            n = n,
            n_width = n_width
        );
    }
}

fn print_packages<I, P>(what: &str, packages: I)
where
    I: IntoIterator<Item = P>,
    P: Display,
{
    let mut packages = packages.into_iter().peekable();
    if packages.peek().is_none() {
        info!("No {}", what.to_lowercase())
    } else {
        info!("{}:", what);
        for package in packages {
            println!("  {}", package);
        }
    }
}
