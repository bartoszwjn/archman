//! Configuration through the config file.

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    env,
    ffi::OsString,
    fs,
    hash::Hash,
    path::{Component, Path, PathBuf},
};

use anyhow::{anyhow, bail, Context};
use serde::Deserialize;

/// The configuration specified in the config file.
#[derive(Debug)]
pub(crate) struct Config {
    /// The directory where the file is located.
    dir: PathBuf,
    /// The path to the user's home directory.
    home: PathBuf,
    /// The hostname of the machine.
    hostname: OsString,
    /// The parsed contents of the file.
    data: ConfigData<OsString>,
}

/// All values that can be specified in the configuration file.
///
/// `H` is the type of a hostname: we must deserialize it as a [`String`], but it is more convenient
/// to store it as an [`OsString`].
#[derive(Debug, Deserialize)]
struct ConfigData<H> {
    /// The files that should be copied to somewhere on the filesystem.
    ///
    /// The maps map locations of the copies to the original files. For a single path to a copy, the
    /// path to the original file specified in the section for a specific host overrides the path
    /// specified in the `common` section.
    #[serde(default, bound = "H: Deserialize<'de> + Eq + Hash")]
    copies: PerHostname<H, HashMap<String, String>>,
    /// The files that should be linked from somewhere on the filesystem.
    ///
    /// The maps map locations of the links to the link targets. For a single path to a link, the
    /// path to the target specified in the section for a specific host overrides the path specified
    /// in the `common` section.
    #[serde(default, bound = "H: Deserialize<'de> + Eq + Hash")]
    links: PerHostname<H, HashMap<String, String>>,
    /// The groups of packages that should be installed on our system.
    ///
    /// The effective set of groups is a set union of groups specified in the `common` section and
    /// those specified for a specific host.
    #[serde(default, bound = "H: Deserialize<'de> + Eq + Hash")]
    package_groups: PerHostname<H, Vec<String>>,
    /// The packages that should be installed on our system.
    ///
    /// The effective set of packages is a set union of packages specified in the `common` section
    /// and those specified for a specific host.
    #[serde(default, bound = "H: Deserialize<'de> + Eq + Hash")]
    packages: PerHostname<H, NestedSet<String>>,
    /// Path to the xkb types file.
    xkb_types: Option<String>,
}

/// Value that can have different definitions depending on the hostname of the machine.
#[derive(Debug, Deserialize)]
struct PerHostname<K, T> {
    /// Values common to all hostnames.
    common: Option<T>,
    /// Values specific to some hostnames.
    #[serde(default, bound = "K: Deserialize<'de> + Eq + Hash")]
    hosts: HashMap<K, T>,
}

/// A nested set of _things_.
///
/// The _things_ in the set can be grouped into named or unnamed groups, with arbitrary nesting.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum NestedSet<T> {
    /// A single _thing_.
    Singleton(T),
    /// A group of sets of _things_, where each set has a name.
    Map(HashMap<String, NestedSet<T>>),
    /// A group of sets of _things_, where sets don't have names.
    Array(Vec<NestedSet<T>>),
}

/// A flattened [`NestedSet`].
#[derive(Debug)]
pub(crate) struct FlattenedSet<T> {
    /// The elements of the set.
    pub(crate) elements: HashSet<T>,
    /// The elements that occured more than once.
    pub(crate) duplicates: HashSet<T>,
}

impl Config {
    /// Reads the configuration file from the given path or the default path.
    pub(crate) fn read_from_file(path: Option<PathBuf>) -> anyhow::Result<Self> {
        let home = get_home_directory().context("Unable to locate the home directory")?;
        let effective_path = path.unwrap_or_else(|| Self::default_path(&home));

        let contents = fs::read_to_string(&effective_path)
            .with_context(|| format!("Failed to read the contents of file {:?}", effective_path))?;
        let raw_data: ConfigData<String> = toml::from_str(&contents).with_context(|| {
            format!(
                "Failed to parse the configuration file {:?}",
                effective_path
            )
        })?;
        let data = ConfigData {
            copies: raw_data.copies.map_keys(OsString::from),
            links: raw_data.links.map_keys(OsString::from),
            package_groups: raw_data.package_groups.map_keys(OsString::from),
            packages: raw_data.packages.map_keys(OsString::from),
            xkb_types: raw_data.xkb_types,
        };

        let absolute_path = effective_path
            .canonicalize()
            .context("Failed to determine the canonical path to the configuration file.")?;
        let dir = match absolute_path.parent() {
            Some(_) => {
                let mut parent = absolute_path;
                parent.pop();
                parent
            }
            None => bail!(
                "Path to the configuration file ({:?}) has no parent directory",
                absolute_path
            ),
        };

        Ok(Self {
            dir,
            home,
            hostname: gethostname::gethostname(),
            data,
        })
    }

    fn default_path(home: &Path) -> PathBuf {
        let mut path = PathBuf::from(home);
        path.push(".config/archman/archman.toml");
        path
    }

    pub(crate) fn xkb_types(&self) -> Option<PathBuf> {
        self.data
            .xkb_types
            .as_ref()
            .map(|p| self.resolve_path(p.as_ref()))
    }

    pub(crate) fn copies(&self) -> HashMap<PathBuf, PathBuf> {
        self.merge_links_or_copies(&self.data.copies)
    }

    pub(crate) fn links(&self) -> HashMap<PathBuf, PathBuf> {
        self.merge_links_or_copies(&self.data.links)
    }

    fn merge_links_or_copies(
        &self,
        paths: &PerHostname<OsString, HashMap<String, String>>,
    ) -> HashMap<PathBuf, PathBuf> {
        let mut ret = HashMap::new();
        let mut extend = |map: &HashMap<String, String>| {
            ret.extend(map.iter().map(|(location, target)| {
                (
                    self.resolve_path(location.as_ref()),
                    self.resolve_path(target.as_ref()),
                )
            }));
        };
        paths.common.as_ref().map(&mut extend);
        // Extending a map overrides old values, so host must go after common
        paths.hosts.get(&self.hostname).map(&mut extend);
        ret
    }

    pub(crate) fn package_groups(&self) -> FlattenedSet<&str> {
        let mut flattened = FlattenedSet::new();
        if let Some(ref common) = self.data.package_groups.common {
            flattened.extend(common.iter().map(AsRef::as_ref));
        }
        if let Some(host) = self.data.package_groups.hosts.get(&self.hostname) {
            flattened.extend(host.iter().map(AsRef::as_ref));
        }
        flattened
    }

    pub(crate) fn packages(&self) -> FlattenedSet<&str> {
        let mut flattened = FlattenedSet::new();
        if let Some(ref common) = self.data.packages.common {
            common.flatten_into(&mut flattened);
        }
        if let Some(host) = self.data.packages.hosts.get(&self.hostname) {
            host.flatten_into(&mut flattened);
        }
        flattened
    }

    fn resolve_path(&self, path: &Path) -> PathBuf {
        let mut components = path.components();
        let substituted_tilde = match components.next() {
            Some(Component::Normal(first_component)) if first_component == "~" => {
                let mut substituted = self.home.clone();
                substituted.push(components.as_path());
                Cow::Owned(substituted)
            }
            _ => Cow::Borrowed(path),
        };
        if substituted_tilde.is_absolute() {
            substituted_tilde.into_owned()
        } else {
            let mut ret = self.dir.clone();
            ret.push(&substituted_tilde);
            ret
        }
    }
}

// TODO detect sudo
/// Returns the path to the user's home directory.
fn get_home_directory() -> anyhow::Result<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("The environment variable HOME is not set"))
}

impl<K1, T> PerHostname<K1, T> {
    fn map_keys<K2: Eq + Hash, F: FnMut(K1) -> K2>(self, mut f: F) -> PerHostname<K2, T> {
        PerHostname {
            common: self.common,
            hosts: self.hosts.into_iter().map(|(k, v)| (f(k), v)).collect(),
        }
    }
}

impl<T> NestedSet<T> {
    /// Converts a `NestedSet` into a [`FlattenedSet`].
    fn flatten_into<'a, E>(&'a self, flattened: &mut FlattenedSet<&'a E>)
    where
        E: Eq + Hash + ?Sized,
        T: AsRef<E>,
    {
        self.for_each(|element| {
            if let Some(duplicate) = flattened.elements.replace(element.as_ref()) {
                flattened.duplicates.insert(duplicate);
            }
        });
    }

    /// Applies a function to each element in the set, passing each element by value.
    fn for_each<'a, F: FnMut(&'a T)>(&'a self, mut f: F) {
        fn do_for_each<'a, T, F: FnMut(&'a T)>(this: &'a NestedSet<T>, f: &mut F) {
            match this {
                NestedSet::Singleton(x) => f(x),
                NestedSet::Map(map) => {
                    for (_, subset) in map {
                        do_for_each(subset, f);
                    }
                }
                NestedSet::Array(array) => {
                    for subset in array {
                        do_for_each(subset, f);
                    }
                }
            }
        }

        do_for_each(self, &mut f)
    }
}

impl<T> FlattenedSet<T> {
    fn new() -> Self {
        Self {
            elements: Default::default(),
            duplicates: Default::default(),
        }
    }
}

impl<T: Eq + Hash> Extend<T> for FlattenedSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for element in iter {
            if let Some(duplicate) = self.elements.replace(element) {
                self.duplicates.insert(duplicate);
            }
        }
    }
}

impl<K, T> Default for PerHostname<K, T> {
    fn default() -> Self {
        Self {
            common: Default::default(),
            hosts: Default::default(),
        }
    }
}

impl<T> Default for NestedSet<T> {
    /// Returns an empty `Array` of _things_.
    fn default() -> Self {
        Self::Array(Default::default())
    }
}
