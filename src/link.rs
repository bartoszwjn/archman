//! Creating links to and copies of configuration files.

use std::{fs, io::ErrorKind, path::Path};

use anyhow::{bail, Context};

use crate::config::Link;

/// Creates symbolic links and copies of files specified in `cfg`.
pub fn create_links(cfg: Link) -> anyhow::Result<()> {
    for (location, target) in cfg.links {
        let parent = match location.parent() {
            Some(parent) => parent,
            None => bail!("The root directory is not a valid link path"),
        };
        create_link(&location, &target, &parent)?;
    }

    if !cfg.copies.is_empty() {
        warn!("copying files as root, not yet implemented");
    }

    Ok(())
}

fn create_link(location: &Path, target: &Path, parent: &Path) -> anyhow::Result<()> {
    match location.symlink_metadata() {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            let old_target = location
                .read_link()
                .with_context(|| format!("Failed to read the target of link {:?}", location))?;
            if old_target == target {
                println!("{:?} already exists", location);
            } else {
                warn!(
                    "{:?} already exists, but its target is {:?}",
                    location, old_target
                );
            }
        }
        Ok(_) => warn!("{:?} already exists, but isn't a link", location),
        Err(err) if err.kind() == ErrorKind::NotFound => {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create the parent directory of {:?}", location)
            })?;
            std::os::unix::fs::symlink(target, location)
                .with_context(|| format!("Failed to create {:?}", location))?;
            println!("Created link {:?} -> {:?}", location, target);
        }
        Err(err) => Err(err)
            .with_context(|| format!("Failed to query for metadata of file {:?}", location))?,
    }
    Ok(())
}
