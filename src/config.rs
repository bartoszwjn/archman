//! Program configuration.
//!
//! The program is configured with command line arguments and a configuration file. Everything can
//! be configured through the command line. That is not the case for the configuration file: it is
//! meant for specifying things that do not change often and are too long to pass as command line
//! arguments every time the program is run.
//!
//! Some values must be specified either through the command line or the configuration file. Values
//! specified through the command line take precedence. Only values specific to the subcommand used
//! must be specified.
//!
//! # Command line arguments
//!
//! The `help` subcommand or `--help` flag can be used to get a description of all recognized
//! arguments.
//!
//! # Configuration file
//!
//! The path to the configuration file can be specified with a command line argument, otherwise it
//! defaults to `$HOME/.config/archman/archman.toml`. The file should be a TOML file, that can be
//! deserialized into a [`Config`] value. Paths specified in the configuration file can start with
//! `~`, which will be substituted for the value of the `HOME` environment variable. See the
//! documentation of [`Config`] for all values that can be configured with the configuration file.

use std::{
    env,
    ffi::OsString,
    fs::File,
    io::Read,
    path::{self, PathBuf},
};

use anyhow::{anyhow, Context};
use serde::Deserialize;
use structopt::StructOpt;

/// The programs command line arguments.
#[derive(Clone, Debug, StructOpt)]
pub struct Args {
    #[structopt(subcommand)]
    pub subcommand: Subcommand,
    /// Path to the configuration file.
    #[structopt(short = "f", long)]
    pub config: Option<PathBuf>,
}

// TODO a better about
/// The programs subcommands.
#[derive(Clone, Debug, StructOpt)]
#[structopt(about = "A configuration utility for my specific Arch Linux setup")]
pub enum Subcommand {
    Pkg(PkgArgs),
}

/// Synchronize installed packages with the package list.
#[derive(Clone, Debug, StructOpt)]
pub struct PkgArgs {
    /// Remove all unneeded packages.
    #[structopt(short = "c", long)]
    pub cleanup: bool,
    /// Do not upgrade packages.
    #[structopt(long)]
    pub no_upgrade: bool,
    /// Path to the package list file.
    #[structopt(short = "p", long)]
    pub package_list: Option<String>,
    /// Path to the xkb types file.
    #[structopt(long)]
    pub xkb_types: Option<String>,
}

/// All values that can be specified in the configuration file.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Configuration specific to the `pkg` subcommand.
    pub pkg: Option<PkgConfig>,
}

/// All values specific to the `pkg` subcommand that can be specified in the configuration file.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PkgConfig {
    /// Path to the package list.
    pub package_list: Option<String>,
    /// Path to the xkb types file.
    pub xkb_types: Option<String>,
}

/// Configuration of the `pkg` subcommand assembled from command line and configuration file.
#[derive(Clone, Debug)]
pub struct Pkg {
    /// Whether to remove all unneeded packages.
    pub cleanup: bool,
    /// Whether to skip upgrading already installed packages.
    pub no_upgrade: bool,
    /// Path to the package list.
    pub package_list: PathBuf,
    /// Path to the xkb types file.
    pub xkb_types: PathBuf,
}

/// Reads the configuration file from the given path or the default path.
///
/// If no path is given and the default path does not point to a file, an empty config is returned.
pub fn read_config_file(path: Option<PathBuf>) -> anyhow::Result<Config> {
    let path_supplied = path.is_some();
    let effective_path = match path {
        Some(path) => path,
        None => match default_config_path() {
            Some(default_path) => default_path,
            None => return Ok(Config::default()),
        },
    };

    let mut file = match File::open(&effective_path) {
        Ok(file) => file,
        Err(err) if path_supplied => {
            return Err(err).context("Failed to open the configuration file")
        }
        Err(_) => return Ok(Config::default()),
    };

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .context("Failed to read the configuration file")?;

    let toml = toml::from_str(&contents).context("Failed to parse the configuration file")?;

    Ok(toml)
}

/// Returns the default config file path, if the `HOME` environment variable is set.
fn default_config_path() -> Option<PathBuf> {
    let home = env::var_os("HOME")?;
    let mut path = PathBuf::from(home);
    path.push(".config/archman/archman.toml");
    Some(path)
}

/// Merges the command line arguments and configuration file into configuration of `pkg` subcommand.
pub fn merge_pkg_config(args: PkgArgs, config: Option<PkgConfig>) -> anyhow::Result<Pkg> {
    let config = config.unwrap_or_default();
    let package_list = Option::or(args.package_list, config.package_list)
        .ok_or_else(|| anyhow!("Package list file was not specified"))?;
    let xkb_types = Option::or(args.xkb_types, config.xkb_types)
        .ok_or_else(|| anyhow!("xkb types file was not specified"))?;

    Ok(Pkg {
        cleanup: args.cleanup,
        no_upgrade: args.no_upgrade,
        package_list: substitute_tilde(package_list).into(),
        xkb_types: substitute_tilde(xkb_types).into(),
    })
}

/// Substitutes a tilde at the start of a string for the value of the `HOME` environment variable.
///
/// The original string is returned if:
/// - the string doesn't start with a tilde
/// - the string starts with a tilde followed by a character that is not a path separator
/// - the environment variable `HOME` is not set
fn substitute_tilde(path: String) -> OsString {
    if !path.starts_with('~') {
        return path.into();
    }

    let rest = &path['~'.len_utf8()..];
    let next_char = rest.chars().next();
    match next_char {
        Some(next_char) if !path::is_separator(next_char) => path.into(),
        _ => match env::var_os("HOME") {
            None => path.into(),
            Some(home) => {
                let mut result = home;
                result.push(rest);
                result.into()
            }
        },
    }
}
