//! Displaying information about declared and currently installed packages.

use std::collections::HashSet;

use anyhow::Context;

use crate::{
    config::Show,
    packages::{self, InstalledPackages, OrganizedPackages},
};

/// Prints out information about declared and installed packages.
pub fn show_packages(cfg: Show) -> anyhow::Result<()> {
    let declared = packages::get_declared_packages(cfg.packages, cfg.package_groups)
        .context("Failed to determine the set of declared packages")?;
    let installed =
        packages::get_installed_packages().context("Failed to query for installed packages")?;
    let organized = packages::organize_packages(&declared, &installed);
    let unneeded =
        packages::get_unneeded_packages().context("Failed to query for unneeded packages")?;

    print_summary(&declared, &installed, &organized, &unneeded);

    Ok(())
}

fn print_summary(
    declared: &HashSet<String>,
    installed: &InstalledPackages,
    organized: &OrganizedPackages<'_>,
    unneeded: &HashSet<String>,
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
        ("unneeded", unneeded.len()),
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
