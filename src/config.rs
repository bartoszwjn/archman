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
    path::{self, Path, PathBuf},
};

use anyhow::{anyhow, bail, Context};
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
    Link(LinkArgs),
    Show(ShowArgs),
    Sync(SyncArgs),
}

/// Create links to and copies of configuration files in declared locations.
#[derive(Clone, Debug, StructOpt)]
pub struct LinkArgs {
    /// The absolute path from which all relative paths to link targets are resolved.
    #[structopt(short = "r", long, validator = validate_absolute_path)]
    pub link_root: Option<String>,
}

/// Display information about declared and currently installed packages.
#[derive(Clone, Debug, StructOpt)]
pub struct ShowArgs {
    /// Equivalent to specifying '-e', '-i', '-r' and '-u'.
    #[structopt(short = "a", long)]
    pub all: bool,
    /// Display all packages that are declared and installed as dependencies.
    #[structopt(short = "e", long)]
    pub to_explicit: bool,
    /// Display all packages that are declared and not installed.
    #[structopt(short = "i", long)]
    pub to_install: bool,
    /// Display all explicitly installed packages that are not declared.
    #[structopt(short = "r", long)]
    pub to_remove: bool,
    /// Display all packages installed as dependencies that are not declared.
    #[structopt(short = "u", long)]
    pub unneeded: bool,
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
    #[structopt(long, validator = validate_absolute_path)]
    pub xkb_types: Option<String>,
}

/// All values that can be specified in the configuration file.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The absolute path from which all relative paths to link targets are resolved.
    link_root: Option<String>,
    /// The files that should be linked from somewhere on the filesystem.
    #[serde(default)]
    links: NestedSet<LinkEntry>,
    /// The groups of packages that should be installed on our system.
    #[serde(default)]
    package_groups: Vec<String>,
    /// The packages that should be installed on our system.
    #[serde(default)]
    packages: NestedSet<String>,
    /// Path to the xkb types file.
    xkb_types: Option<String>,
}

/// A nested set of _things_.
///
/// The _things_ in the set can be grouped into named or unnamed groups, with arbitrary nesting.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum NestedSet<T> {
    /// A single _thing_.
    Singleton(T),
    /// A group of sets of _things_, where each set has a name.
    Map(HashMap<String, Self>),
    /// A group of sets of _things_, where sets don't have names.
    Array(Vec<Self>),
}

impl<T> Default for NestedSet<T> {
    /// Returns an empty `Array` of _things_.
    fn default() -> Self {
        Self::Array(vec![])
    }
}

/// Description of a file that should be linked or copied.
#[derive(Clone, Debug, Deserialize)]
struct LinkEntry {
    /// Should the file be copied instead of linking.
    #[serde(default)]
    copy: bool,
    /// Absolute path to where the link or copy should be located.
    location: String,
    /// What file should be pointed to by the link or copied.
    ///
    /// Relative paths are resolved relative to `link_root`.
    target: String,
}

/// Configuration of the `link` subcommand assembled from command line and configuration file.
#[derive(Clone, Debug)]
pub struct Link {
    /// Files that should be linked or copied.
    ///
    /// Keys must be absolute paths.
    pub links: HashMap<PathBuf, LinkTarget>,
}

/// Description of a target file of a link or copy.
#[derive(Clone, Debug)]
pub struct LinkTarget {
    /// Should the file be copied instead of linking.
    pub copy: bool,
    /// Absolute path to the link target.
    pub path: PathBuf,
}

/// Configuration of the `show` subcommand assembled from command line and configuration file.
#[derive(Clone, Debug)]
pub struct Show {
    /// The declared package groups.
    pub package_groups: Vec<String>,
    /// The declared packages.
    pub packages: HashSet<String>,
    /// Whether to display all packages that are declared and installed as dependencies.
    pub to_explicit: bool,
    /// Whether to display all packages that are declared and not installed.
    pub to_install: bool,
    /// Whether to display all explicitly installed packages that are not declared.
    pub to_remove: bool,
    /// Whether to display all packages installed as dependencies that are not declared.
    pub unneeded: bool,
}

/// Configuration of the `sync` subcommand assembled from command line and configuration file.
#[derive(Clone, Debug)]
pub struct Sync {
    /// Whether to remove all unneeded packages.
    pub cleanup: bool,
    /// Whether to skip upgrading already installed packages.
    pub no_upgrade: bool,
    /// The declared package groups.
    pub package_groups: Vec<String>,
    /// The declared packages.
    pub packages: HashSet<String>,
    /// Path to the xkb types file.
    pub xkb_types: Option<PathBuf>,
}

/// Reading the configuration file.
impl Config {
    /// Reads the configuration file from the given path or the default path.
    ///
    /// If no path is given and the default path does not point to a file, an empty config is
    /// returned.
    ///
    /// # Errors
    ///
    /// Returns `Err` if:
    /// - `path` is `Some`, but the given path does not exist
    /// - contents of the file are invalid
    /// - an IO error occurs
    pub fn read_from_file(path: Option<PathBuf>) -> anyhow::Result<Self> {
        let path_supplied = path.is_some();
        let effective_path = match path {
            Some(path) => path,
            None => match Self::default_path() {
                Some(default_path) => default_path,
                None => return Ok(Self::default()),
            },
        };

        let mut file = match File::open(&effective_path) {
            Ok(file) => file,
            Err(err) if !path_supplied && err.kind() == io::ErrorKind::NotFound => {
                // If the default path doesn't exist, use the empty configuration
                return Ok(Self::default());
            }
            Err(err) => return Err(err).context("Failed to open the configuration file"),
        };

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .context("Failed to read the configuration file")?;

        let this: Self =
            toml::from_str(&contents).context("Failed to parse the configuration file")?;

        this.validate()
            .context("Contents of the configuration file are invalid")?;

        Ok(this)
    }

    /// Returns the default config file path, if the `HOME` environment variable is set.
    fn default_path() -> Option<PathBuf> {
        let home = env::var_os("HOME")?;
        let mut path = PathBuf::from(home);
        path.push(".config/archman/archman.toml");
        Some(path)
    }

    fn validate(&self) -> anyhow::Result<()> {
        if let Some(ref xkb_types) = self.xkb_types {
            validate_absolute_path(xkb_types)
                .map_err(|s| anyhow!(s))
                .context("Invalid value for the 'xkb_types' field")?;
        }

        if let Some(ref link_root) = self.link_root {
            validate_absolute_path(link_root)
                .map_err(|s| anyhow!(s))
                .context("Invalid value for the 'link_root' field")?;
        }

        let mut locations = HashSet::new();
        self.links
            .try_for_each_ref(|link_entry| {
                validate_absolute_path(&link_entry.location).map_err(|s| anyhow!(s))?;
                if let Some(duplicate) = locations.replace(&link_entry.location) {
                    bail!("Multiple links with location {:?}", duplicate);
                }
                Ok(())
            })
            .context("Invalid value of a link location")?;

        Ok(())
    }
}

impl Link {
    /// Builds configuration of the `link` subcommand from command line arguments and config file.
    pub fn new(args: LinkArgs, config: Config) -> anyhow::Result<Link> {
        // We already ensured that this is an absolute path.
        let link_root = Option::or(args.link_root, config.link_root).map(PathBuf::from);

        let mut links = HashMap::new();
        config.links.try_for_each(|link_entry| {
            let location = PathBuf::from(link_entry.location);
            let target = PathBuf::from(link_entry.target);
            let resolved_target = if target.is_absolute() {
                target
            } else {
                match link_root {
                    Some(ref root) => {
                        let mut resolved_target = root.clone();
                        resolved_target.push(&target);
                        resolved_target
                    }
                    None => bail!(
                        "Found a relative link target {:?} and 'link_root' was not specified",
                        target
                    ),
                }
            };

            links.insert(
                location,
                LinkTarget {
                    copy: link_entry.copy,
                    path: resolved_target,
                },
            );

            Ok(())
        })?;

        Ok(Self { links })
    }
}

impl Show {
    /// Builds configuration of the `show` subcommand from command line arguments and config file.
    pub fn new(args: ShowArgs, config: Config) -> Self {
        Self {
            package_groups: config.package_groups,
            packages: config.packages.flatten_packages(),
            to_explicit: args.to_explicit || args.all,
            to_install: args.to_install || args.all,
            to_remove: args.to_remove || args.all,
            unneeded: args.unneeded || args.all,
        }
    }
}

impl Sync {
    /// Builds configuration of the `sync` subcommand from command line arguments and config file.
    pub fn new(args: SyncArgs, config: Config) -> anyhow::Result<Self> {
        let xkb_types = Option::or(args.xkb_types, config.xkb_types)
            .map(|xkb_types| substitute_tilde(xkb_types).into());

        Ok(Self {
            cleanup: args.cleanup,
            no_upgrade: args.no_upgrade,
            package_groups: config.package_groups,
            packages: config.packages.flatten_packages(),
            xkb_types,
        })
    }
}

impl<T> NestedSet<T> {
    /// Applies a function to each element in the set, passing each element by value.
    fn for_each(self, mut f: impl FnMut(T)) {
        fn inner<T>(this: NestedSet<T>, f: &mut impl FnMut(T)) {
            match this {
                NestedSet::Singleton(x) => f(x),
                NestedSet::Map(map) => {
                    for (_, subset) in map {
                        inner(subset, f);
                    }
                }
                NestedSet::Array(array) => {
                    for subset in array {
                        inner(subset, f);
                    }
                }
            }
        }

        inner(self, &mut f)
    }

    /// Applies a function to each element in the set, passing each element by value and returning
    /// early in case of an error.
    fn try_for_each<E>(self, mut f: impl FnMut(T) -> Result<(), E>) -> Result<(), E> {
        fn inner<T, E>(
            this: NestedSet<T>,
            f: &mut impl FnMut(T) -> Result<(), E>,
        ) -> Result<(), E> {
            match this {
                NestedSet::Singleton(x) => f(x),
                NestedSet::Map(map) => {
                    for (_, subset) in map {
                        inner(subset, f)?;
                    }
                    Ok(())
                }
                NestedSet::Array(array) => {
                    for subset in array {
                        inner(subset, f)?;
                    }
                    Ok(())
                }
            }
        }

        inner(self, &mut f)
    }

    /// Applies a function to each element in the set, passing each element by reference and
    /// returning early in case of an error.
    fn try_for_each_ref<'a, E>(
        &'a self,
        mut f: impl FnMut(&'a T) -> Result<(), E>,
    ) -> Result<(), E> {
        fn inner<'a, T, E>(
            this: &'a NestedSet<T>,
            f: &mut impl FnMut(&'a T) -> Result<(), E>,
        ) -> Result<(), E> {
            match *this {
                NestedSet::Singleton(ref x) => f(x),
                NestedSet::Map(ref map) => {
                    for subset in map.values() {
                        inner(subset, f)?;
                    }
                    Ok(())
                }
                NestedSet::Array(ref array) => {
                    for subset in array {
                        inner(subset, f)?;
                    }
                    Ok(())
                }
            }
        }

        inner(self, &mut f)
    }
}

impl NestedSet<String> {
    /// Converts a nested set of packages into a [`HashSet`] of packages, printing warnings if the
    /// nested set contains duplicates.
    fn flatten_packages(self) -> HashSet<String> {
        let mut set = HashSet::new();
        self.for_each(|package| {
            if let Some(duplicate) = set.replace(package) {
                warn!("Package {:?} is declared multiple times", duplicate);
            }
        });
        set
    }
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

fn validate_absolute_path<P: AsRef<Path>>(path: P) -> Result<(), String> {
    if path.as_ref().is_absolute() {
        Ok(())
    } else {
        Err("Expected an absolute path".to_owned())
    }
}
