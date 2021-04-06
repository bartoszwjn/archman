//! Program configuration.
//!
//! The program is configured with command line arguments and a configuration file. Some things can
//! only be configured with command line arguments while others only with the configuration file.
//! Some values can be specified either through the command line or the configuration file. Values
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
    collections::{HashMap, HashSet},
    env,
    ffi::OsString,
    fs::File,
    io::{self, Read},
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
    Sync(SyncArgs),
}

/// Synchronize installed packages with the package list.
#[derive(Clone, Debug, StructOpt)]
pub struct SyncArgs {
    /// Remove all unneeded packages.
    #[structopt(short = "c", long)]
    pub cleanup: bool,
    /// Do not upgrade packages.
    #[structopt(long)]
    pub no_upgrade: bool,
    /// Path to the xkb types file.
    #[structopt(long)]
    pub xkb_types: Option<String>,
}

/// All values that can be specified in the configuration file.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The groups of packages that should be installed on our system.
    #[serde(default)]
    pub package_groups: Vec<String>,
    /// The packages that should be installed on our system.
    pub packages: Option<Packages>,
    /// Path to the xkb types file.
    pub xkb_types: Option<String>,
}

/// The list of packages that should be installed.
///
/// The package list can be grouped into named or unnamed groups, with arbitrary nesting.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Packages {
    /// A name of a single package that should be installed.
    Package(String),
    /// A group of package lists, where each list has a name.
    Map(HashMap<String, Packages>),
    /// A group of package lists, where lists don't have names.
    Array(Vec<Packages>),
}

/// Configuration of the `sync` subcommand assembled from command line and configuration file.
#[derive(Clone, Debug)]
pub(crate) struct Sync {
    /// Whether to remove all unneeded packages.
    pub cleanup: bool,
    /// Whether to skip upgrading already installed packages.
    pub no_upgrade: bool,
    /// The list of declared package groups.
    pub package_groups: Vec<String>,
    /// The list of declared packages.
    pub packages: HashSet<String>,
    /// Path to the xkb types file.
    pub xkb_types: PathBuf,
}

/// Reads the configuration file from the given path or the default path.
///
/// If no path is given and the default path does not point to a file, an empty config is returned.
pub(crate) fn read_config_file(path: Option<PathBuf>) -> anyhow::Result<Config> {
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
        Err(err) if !path_supplied && err.kind() == io::ErrorKind::NotFound => {
            // If the default path doesn't exist, use the empty configuration
            return Ok(Config::default());
        }
        Err(err) => return Err(err).context("Failed to open the configuration file"),
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

/// Merges the command line arguments and configuration file into configuration of `sync`
/// subcommand.
pub(crate) fn merge_sync_config(args: SyncArgs, config: Config) -> anyhow::Result<Sync> {
    let packages = config
        .packages
        .ok_or_else(|| anyhow!("Package list file was not specified"))?
        .into_set();
    let xkb_types = Option::or(args.xkb_types, config.xkb_types)
        .ok_or_else(|| anyhow!("xkb types file was not specified"))?;

    Ok(Sync {
        cleanup: args.cleanup,
        no_upgrade: args.no_upgrade,
        package_groups: config.package_groups,
        packages,
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
                result
            }
        },
    }
}

impl Packages {
    /// Flattens the nested list of packages into a `HashSet`.
    fn into_set(self) -> HashSet<String> {
        let mut set = HashSet::new();
        self.add_to_set(&mut set);
        set
    }

    /// Adds all packages from `self` to the given `HashSet`.
    fn add_to_set(self, set: &mut HashSet<String>) {
        match self {
            Packages::Package(package) => {
                if let Some(duplicate) = set.replace(package) {
                    warn!("Package {:?} is declared multiple times", duplicate);
                }
            }
            Packages::Map(map) => {
                for (_, packages) in map {
                    packages.add_to_set(set);
                }
            }
            Packages::Array(array) => {
                for packages in array {
                    packages.add_to_set(set);
                }
            }
        }
    }
}
