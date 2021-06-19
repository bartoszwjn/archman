//! Creating links to and copies of configuration files.

use std::{fs, io::ErrorKind, os::unix, path::Path};

use anyhow::{anyhow, Context};

use crate::{
    args::{CopyArgs, LinkArgs},
    config::Config,
};

// TODO don't stop on error during the loop

/// Creates symbolic links to files specified in `cfg`.
pub(crate) fn create_links(args: LinkArgs, cfg: Config) -> anyhow::Result<()> {
    let LinkArgs {} = args;
    for (location, target) in cfg.links() {
        let parent = location
            .parent()
            // TODO indicate which entry caused the error
            .ok_or_else(|| anyhow!("The root directory is not a valid link path"))?;
        create_link(&location, &target, parent)?;
    }
    Ok(())
}

pub(crate) fn create_copies(args: CopyArgs, cfg: Config) -> anyhow::Result<()> {
    let CopyArgs {} = args;
    for (copy, original) in cfg.copies() {
        let parent = copy
            .parent()
            // TODO indicate which entry caused the error
            .ok_or_else(|| anyhow!("The root directory is not a valid copy path"))?;
        create_copy(&copy, &original, parent)?;
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
                    "{:?} already exists, but its target is {:?}, (expected {:?})",
                    location, old_target, target,
                );
            }
        }
        Ok(_) => warn!("{:?} already exists, but isn't a link", location),
        Err(err) if err.kind() == ErrorKind::NotFound => {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create the parent directory of {:?}", location)
            })?;
            unix::fs::symlink(target, location)
                .with_context(|| format!("Failed to create {:?}", location))?;
            println!("Created link {:?} -> {:?}", location, target);
        }
        Err(err) => Err(err)
            .with_context(|| format!("Failed to query for metadata of file {:?}", location))?,
    }
    Ok(())
}

fn create_copy(copy: &Path, original: &Path, parent: &Path) -> anyhow::Result<()> {
    match copy.symlink_metadata() {
        Ok(metadata) if metadata.file_type().is_file() => {
            let original_contents = fs::read(original)
                .with_context(|| format!("Failed to read the contents of {:?}", original))?;
            let dest_contents = fs::read(copy)
                .with_context(|| format!("Failed to read the contents of {:?}", copy))?;
            if original_contents == dest_contents {
                println!("{:?} already exists", copy);
            } else {
                warn!(
                    "{:?} already exists, but is different from {:?}",
                    copy, original,
                );
            }
        }
        Ok(_) => warn!("{:?} already exists, but isn't a regular file", copy),
        Err(err) if err.kind() == ErrorKind::NotFound => {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create the parent directory of {:?}", copy))?;
            fs::copy(original, copy)
                .with_context(|| format!("Failed to copy {:?} to {:?}", original, copy))?;
            println!("Copied {:?} -> {:?}", original, copy);
        }
        Err(err) => {
            Err(err).with_context(|| format!("Failed to query for metadata of file {:?}", copy))?
        }
    }
    Ok(())
}
