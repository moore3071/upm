//! The universal package manager library (upm-lib) provides an abstraction to perform simple
//! commands with any package manager. If you want to do something with a particular package
//! manager then this probably isn't the crate for you. There is a distinction between installation
//! and removal of packages on a system-wide level and a local level (think of language level
//! managers like gem, pip, and Cargo).

#[macro_use]
extern crate failure;
extern crate regex;
extern crate toml;

use std::process::{Command,Child};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::fs::{File,read_dir};
use std::io::prelude::*;
use std::cmp::Ordering;
use std::path::PathBuf;
use failure::Error;
use regex::Regex;
use toml::Value;

/// The representation of a package manager. Includes the name of the package manager, a path to
/// reference scripts from, and commands in string form (or scripts to call package manager
/// commands and properly format the output)
#[derive(Eq)]
pub struct PackageManager {
    pub name: String,
    pub version: String,
    pub config_dir: PathBuf,
    pub install: Option<String>,
    pub install_local: Option<String>,
    pub remove: Option<String>,
    pub remove_local: Option<String>,
    pub search: Option<String>,
}

impl PackageManager {
    /// Create a blank PackageManager with no commands and the name and version being empty Strings.
    pub fn new() -> PackageManager {
        PackageManager {
            name: String::from(""),
            version: String::from(""),
            config_dir: PathBuf::new(),
            install: None,
            install_local: None,
            remove: None,
            remove_local: None,
            search: None,
        }
    }

    /// Check if the PackageManager is installed by seeing if the version command exits with a
    /// status code of 0.
    pub fn exists(self) -> bool {
        let mut version_command = self.make_command("version");
        match version_command.status() {
            Ok(s) => s.success(),
            Err(_) => false
        }
    }

    /// Check if the specified command field of the struct is some
    pub fn has_command(self, name: &str) -> bool {
        match name {
            "version" => true,
            "install" => self.install.is_some(),
            "install_local" => self.install_local.is_some(),
            "remove" => self.remove.is_some(),
            "remove_local" => self.remove_local.is_some(),
            &_ => false,
        }
    }

    /// Attempt to run the PackageManager command specified by name. Arguments can be supplied with
    /// the args parameter.
    pub fn run_command(self, name: &str, args: &str) -> Result<Child,Error> {
        let mut command = self.make_command(name);
        command.args(args.split_whitespace());
        match command.spawn() {
            Ok(child) => Ok(child),
            Err(_) => bail!("Couldn't execute command")
        }
    }

    /// Turns the String that describes a command into a std::process::Command struct.
    fn make_command(self, name: &str) -> Command {
        let tmp = match name {
            "version" => self.version,
            "install" => self.install.unwrap(),
            "install_local" => self.install_local.unwrap(),
            "remove" => self.remove.unwrap(),
            "remove_local" => self.remove_local.unwrap(),
            _ => panic!("No such command"),
        };
        let mut tmp = tmp.split_whitespace();
        let mut result = Command::new(tmp.nth(0).unwrap());
        let args: Vec<&str> = tmp.collect();
        result.args(args);
        result
    }

    /// Run the install command with the provided arguments
    pub fn install(self, args: &str) -> Result<Child,Error> {
        self.run_command("install", args)
    }

    /// Run the uninstall command with the provided arguments
    pub fn uninstall(self, args: &str) -> Result<Child,Error> {
        self.run_command("uninstall", args)
    }

    /// Run the search command with the provided arguments
    pub fn search(self, args: &str) -> Result<Child,Error> {
        self.run_command("search", args)
    }

    /// Get the name of the package manager
    pub fn get_name(self) -> String {
        self.name
    }

    /// Get the directory of the configuration file that describes the PackageManager
    pub fn get_config_dir(self) -> PathBuf {
        self.config_dir
    }

    /// Run the version command
    pub fn version(self) -> Result<Child,Error> {
        self.run_command("version", "")
    }

    /// Get the Version of the package manager
    pub fn get_version(self) -> Result<Version,Error> {
        let mut command = self.make_command("version");
        let output = command.output()?;
        let version_string = String::from_utf8(output.stdout)?;
        Ok(Version::from_str(&version_string))
    }

    /// Read a toml configuration file with a PackageManager description and create a
    /// PackageManager from this info.
    pub fn from_file(path: &PathBuf) -> Result<PackageManager,Error> {
        let mut file = File::open(&path)?;

        let mut content = String::new();

        file.read_to_string(&mut content)?;

        let resource = content.as_str().parse::<Value>()?;

        let name: String = String::from(path.file_stem().unwrap().to_str().unwrap());

        let version: String = match resource.get("version") {
            Some(s) => s.as_str().unwrap().to_owned(),
            None => bail!("Package manager version command not provided in config")
        };

        let install: Option<String> = match resource.get("install") {
            Some(s) => Some(String::from(s.as_str().unwrap())),
            None => None
        };
        let install_local: Option<String> = match resource.get("install_local") {
            Some(s) => Some(String::from(s.as_str().unwrap())),
            None => None
        };
        let remove: Option<String> = match resource.get("remove") {
            Some(s) => Some(String::from(s.as_str().unwrap())),
            None => None
        };
        let remove_local: Option<String> = match resource.get("remove_local") {
            Some(s) => Some(String::from(s.as_str().unwrap())),
            None => None
        };
        let search: Option<String> = match resource.get("search") {
            Some(s) => Some(String::from(s.as_str().unwrap())),
            None => None
        };

       let config_dir: PathBuf = match path.parent() {
           Some(dir) => dir.to_path_buf(),
           None => PathBuf::new()
       };

        Ok(PackageManager {
            name,
            version,
            config_dir,
            install,
            install_local,
            remove,
            remove_local,
            search,
        })
    }
}

impl PartialEq for PackageManager {
    fn eq(&self, other: &PackageManager) -> bool {
        self.name == other.name
    }
}

impl Ord for PackageManager {
    fn cmp(&self, other: &PackageManager) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for PackageManager {
    fn partial_cmp(&self, other: &PackageManager) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for PackageManager {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

/// Information on a package from a particular package manager
pub struct Package {
    pub name: String,
    pub owner: PackageManager,
    pub version: Version,
    pub description: String,
}

impl Package {
    /// Return whether the package has the specified name
    pub fn is_called(self, name: &str) -> bool {
        self.name == name
    }

    /// Make a new blank package
    pub fn new() -> Package {
        Package{
            name: String::new(),
            owner: PackageManager::new(),
            version: Version::new(),
            description: String::new()
        }
    }

    /// Call install from the PackageManager pointed to by owner.
    pub fn install(self) -> Result<Child,Error> {
        self.owner.install(&self.name)
    }

    /// Call uninstall from the PackageManager pointed to by owner.
    pub fn uninstall(self) -> Result<Child,Error> {
        self.owner.uninstall(&self.name)
    }

    /// Return the package name
    pub fn get_name(self) -> String {
        self.name
    }

    /// Return the package version
    pub fn get_version(self) -> Version {
        self.version
    }

    /// Return the description of the package
    pub fn get_description(self) -> String {
        self.description
    }

    /// Return the PackageManager that owns this
    /// package
    pub fn get_manager(self) -> PackageManager {
        self.owner
    }
}

/// A simple representation of a version string. For semantic versioning it is quite similar to
/// Steve Klabnik's semver crate, but non-semantic versioning is also permitted.
#[derive(Debug)]
pub struct Version {
    pub representation: String,
    pub semantic: bool
}

impl Version {
    /// Create an empty version that is not semantic
    pub fn new() -> Version {
        Version {
            representation: String::new(),
            semantic: false,
        }
    }

    /// Create a version from a string. Checks if the version fits with semantic versioning 2.0.0
    /// and sets semantic to true if it does.
    fn from_str(representation: &str) -> Version {
        let semantic = Version::is_semantic(representation);
        Version {
            representation: String::from(representation),
            semantic,
        }
    }

    pub fn get_representation(self) -> String {
        self.representation
    }

    pub fn set_representation(&mut self, val: String) {
        self.representation = val;
        self.semantic = Version::is_semantic(&self.representation);
    }

    /// Check if a representation fits with semantic versioning
    fn is_semantic(representation: &str) -> bool {
        let re = Version::get_semantic_regex();
        re.is_match(representation)
    }

    fn get_semantic_regex() -> Regex {
        Regex::new(r"^(\d+)\.(\d+)\.(\d+)(?:-([\dA-Za-z-]+(?:\.[\dA-Za-z-]+)*))?(?:\+([\dA-Za-z-]+(?:\.[\dA-Za-z-]+)*))?$").unwrap()
    }

    /// Explicitly set whether the version is semantic. If the version string doesn't pass
    /// is_semantic, then it won't set semantic to true and will return false.
    pub fn set_semantic(&mut self, val: bool) -> bool {
        if val {
            if !Version::is_semantic(&self.representation) {
                return false;
            }
        }
        self.semantic = val;
        true
    }

    pub fn get_semantic(self) -> bool {
        return self.semantic
    }
    
}

impl PartialEq for Version {
    fn eq(&self, other: &Version) -> bool {
        if self.semantic != other.semantic {
            return false;
        }
        else if self.semantic && other.semantic {
            let re = Version::get_semantic_regex();
            let self_groups = re.captures(&self.representation).unwrap();
            let other_groups = re.captures(&other.representation).unwrap();
            return self_groups.get(1)==other_groups.get(1) && self_groups.get(2)==
                other_groups.get(2) && self_groups.get(3) == other_groups.get(3);
        } else {
            return self.representation == other.representation;
        }
    }
}
//TODO implement ordering for Versions

//TODO Panic a bit for certain cases.
// 1. pathbuf is not a directory
// 2. can't read directory
// This should probably return a result
/// Get a vector of any package managers specified in the given directory.
pub fn get_managers(directory: &PathBuf, names: &ManagerSpecifier) -> Vec<PackageManager> {
    let mut result = Vec::new();
    if let Ok(entries) = read_dir(directory) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                let name = entry.file_name();
                if name.to_str().unwrap().ends_with(".toml") {
                    if let &Some(stem) = &path.file_stem() {
                        match &names {
                            &&ManagerSpecifier::Excludes(ref set) => {
                                if set.contains(stem.to_str().unwrap()) {
                                    continue;
                                }
                            },
                            &&ManagerSpecifier::Includes(ref set) => {
                                if !set.contains(stem.to_str().unwrap()) {
                                    continue;
                                }
                            },
                            _ => {}
                        };
                        let manager = PackageManager::from_file(&path);
                        match manager {
                            Ok(man) => result.push(man),
                            Err(_e) => {}
                        }
                    }
                }
            }
        }
    }
    result
}

/// Provide a single type to exclude or solely include certain packagemanager names.
pub enum ManagerSpecifier {
    Excludes(HashSet<&'static str>),
    Includes(HashSet<&'static str>),
    Empty,
}

/// Read the configuration directories listed from highest precedence to lowest with the option to
/// explicitly exclude or include certain package managers. If the include variant of
/// ManagerSpecifier is used then only the specified packagemanager names will be returned if they
/// exist.
pub fn read_config_dirs(directories: Vec<&PathBuf>, exceptions: ManagerSpecifier) -> Vec<PackageManager> {
    let mut result: HashSet<PackageManager> = HashSet::new();
    for dir in directories {
        let tmp = get_managers(&dir, &exceptions);
        for manager in tmp {
            if !result.contains(&manager) {
                result.insert(manager);
            }
        }
    }
//    let global_dir = PathBuf::from(global_conf_dir());
//    let secondary_dir = PathBuf::from(secondary_conf_dir());
    let return_value: Vec<PackageManager> = result.into_iter().collect();
    return_value
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn semantic_matching() {
        let mut semantics: Vec<&str> = Vec::new();
        semantics.push("0.1.1");
        semantics.push("0.1.1-prerelease");
        semantics.push("0.1.1-prerelease.x.3");
        semantics.push("0.1.1-pre-pre-release");
        semantics.push("0.1.1+builddata");
        semantics.push("0.1.1+build-data");
        semantics.push("0.1.1+builddata.3");
        semantics.push("0.1.1-prerelease+builddata");
        let mut jejune: Vec<&str> = Vec::new();
        jejune.push("a.b.c");
        jejune.push("1-1-1");
        jejune.push("0.1.1-b@d");
        jejune.push("0.1.1+b@d");
        for string in &semantics {
            assert!(Version::is_semantic(string), "{} was detected as not semantic", string);
        }
        for string in &jejune {
            assert!(!Version::is_semantic(string), "{} was detected as semantic", string);
        }
    }

    #[test]
    fn creation_test() {
        let blank_version = Version::new();
        assert_eq!(blank_version.representation, String::new());
        assert!(!blank_version.semantic);
        let semantic_string = "0.1.2";
        let non_semantic_string = "1.4rc2";
        let semantic_version = Version::from_str(semantic_string);
        assert!(semantic_version.get_semantic());
        let non_semantic_version = Version::from_str(non_semantic_string);
        assert!(!non_semantic_version.get_semantic());
    }

    #[test]
    fn equality_test() {
        let version1 = Version::from_str("0.1.2");
        let version2 = Version::from_str("1.4rc2");
        let mut version3 = Version::from_str("0.1.2");
        assert_eq!(version1,version3);
        assert_ne!(version1,version2);
        version3.set_semantic(false);
        assert_ne!(version1,version3);
    }
}
