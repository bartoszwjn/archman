//! Synchronizing installed packages.

use std::{
    cmp::Ordering,
    env,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
    process::Command,
};

use anyhow::{anyhow, ensure, Context};
use regex::Regex;

use crate::{
    args::Pkg,
    pacman::{self, InstallReason, Origin, Package, Packages, PacmanError, QueryFilter},
};

pub fn synchronize_packages(args: Pkg) -> anyhow::Result<()> {
    update_packages(!args.no_upgrade).context("Failed to update installed packages")?;

    let declared = parse_packages_file(&args.package_list)?;
    let installed = query_installed_packages().context("Failed to query installed packages")?;
    let (to_install, to_remove) = packages_diff(&declared, &installed);

    remove_packages(&to_remove).context("Failed to remove undeclared packages")?;
    install_packages(&to_install).context("Failed to install new packages")?;

    patch_xkb_types(&args.xkb_types).context("Failed to patch the xkb types file")?;

    Ok(())
}

fn update_packages(upgrade: bool) -> anyhow::Result<()> {
    println!(
        "Updating packages ({} system upgrade)...",
        if upgrade { "with" } else { "without" }
    );
    match pacman::update(upgrade) {
        Ok(()) => Ok(()),
        Err(PacmanError::ExitFailure) => {
            println!("pacman did not exit successfully, continuing...");
            Ok(())
        }
        Err(err) => Err(err.into()),
    }
}

fn parse_packages_file(path: &Path) -> anyhow::Result<Vec<String>> {
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
    println!("Found {} declared packages", packages.len());

    Ok(packages)
}

fn query_installed_packages() -> anyhow::Result<Packages> {
    let packages = pacman::query(QueryFilter {
        install_reason: InstallReason::Explicit,
        origin: Origin::Native,
        ..QueryFilter::default()
    })?;
    println!(
        "Found {} explicitly installed, native packages",
        packages.len()
    );

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

fn remove_packages(to_remove: &[&str]) -> anyhow::Result<()> {
    if to_remove.is_empty() {
        println!("No packages need to be removed");
        return Ok(());
    }

    println!("Packages to be removed: {}", to_remove.len());
    for package in to_remove {
        println!("  {}", package);
    }

    match pacman::remove(to_remove, true) {
        Ok(()) => Ok(()),
        Err(PacmanError::ExitFailure) => {
            println!("pacman did not exit successfully, continuing...");
            Ok(())
        }
        Err(err) => Err(err.into()),
    }
}

fn install_packages(to_install: &[&str]) -> anyhow::Result<()> {
    if to_install.is_empty() {
        println!("No packages need to be installed");
        return Ok(());
    }

    println!("Packages to be installed: {}", to_install.len());
    for package in to_install {
        println!("  {}", package);
    }

    match pacman::install(to_install) {
        Ok(()) => Ok(()),
        Err(PacmanError::ExitFailure) => {
            println!("pacman did not exit successfully, continuing...");
            Ok(())
        }
        Err(err) => Err(err.into()),
    }
}

fn patch_xkb_types(path: &Path) -> anyhow::Result<()> {
    let mut contents = fs::read_to_string(path).context("Failed to read from file")?;

    const XKB_TYPES_REGEX_STR: &str =
        r#"^default xkb_types "complete" \{\n(?:    include "[[:alnum:]]+"\n)*\};\n$"#;
    let contents_regex =
        Regex::new(XKB_TYPES_REGEX_STR).context("Failed to build a regular expression")?;
    ensure!(
        contents_regex.is_match(&contents),
        "Did not recognize the contents of the xkb types file"
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
            path
        ))
    }
}
