//! The universal package manager library (upm-lib) provides an abstraction to perform simple
//! commands with any package manager. Currently there are no frontends implemented, but the
//! functionality is exposed for frontends to utilize. Feel free to implement a frontend!
//!
//! If you want to do something with a particular package manager then this probably isn't the
//! library for you. If you want to query multiple package managers at once to search for a package
//! provided by multiple sources, then this is the library for you. This is common for language
//! specific binaries that are provided by language package managers and system package managers.
//!
//! Since certain package managers such as NPM allow installation in a user's home directory or
//! somewhere accessible for all users, there is a distinction between installation and removal of
//! packages on a system-wide level and a local level.
//!
//! It is expected that the frontend would load in the different package managers from
//! configuration files as discussed in [`PackageManager`](struct.PackageManager.html).
//!
//! Versioning is provided by the [Version] struct. [Version] is used in place of
//! [semver](https://crates.io/crates/semver) due to the need to support non-semantic versions.
//!
//! [Version]: struct.Version.html

#[macro_use] extern crate failure;
extern crate regex;
extern crate toml;

use std::process::{Command,Child};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::fs::{File,read_dir};
use std::io::prelude::*;
use std::cmp::Ordering;
use std::path::{PathBuf, Path};
use failure::Error;
use regex::Regex;
use toml::Value;

/// The representation of a package manager. Includes the name of the package manager, a path to
/// reference scripts from, and commands in string form (or scripts to call package manager
/// commands and properly format the output).
#[derive(Eq,Clone,Default)]
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
    //Concats a config_dir with a command that starts with ./ otherwise it returns the command str
    fn fix_relative_path(config_dir: &PathBuf, command: &str) -> String {
        if command.starts_with("./") {
                let mut tmp = config_dir.as_os_str().to_str().unwrap().to_owned();
                tmp.push_str(command);
                tmp
        } else {
            command.to_owned()
        }
    }

    /// Check if the PackageManager is installed by seeing if the version command exits with a
    /// status code of 0.
    pub fn exists(&self) -> bool {
        let mut version_command = self.make_command("version").unwrap();
        let status = version_command.status().expect("Failed to run version command");
        status.success()
    }

    /// Check if the specified command field of the struct is some
    pub fn has_command(&self, name: &str) -> bool {
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
    pub fn run_command(&self, name: &str, args: &str) -> Result<Child,Error> {
        let mut command = self.make_command(name).unwrap();
        command.args(args.split_whitespace());
        match command.spawn() {
            Ok(child) => Ok(child),
            Err(_) => bail!("Couldn't execute command")
        }
    }

    /// Turns the String that describes a command into a std::process::Command struct.
    /// # Panics
    /// Panics if the name provided isn't one of the commands in the PackageManager struct
    fn make_command(&self, name: &str) -> Option<Command> {
        let tmp: Option<&String> = match name {
            "version" => Some(&self.version),
            "install" => self.install.as_ref(),
            "install_local" => self.install_local.as_ref(),
            "remove" => self.remove.as_ref(),
            "remove_local" => self.remove_local.as_ref(),
            _ => panic!("No such command"),
        };
        match tmp {
            Some(s) => {
                let s = PackageManager::fix_relative_path(&self.config_dir, s);
                let mut s = s.split_whitespace();
                let mut result = Command::new(s.nth(0).unwrap());
                let args: Vec<&str> = s.collect();
                result.args(args);
                Some(result)
            },
            None => None,
        }
    }

    /// Run the install command with the provided arguments
    pub fn install(&self, args: &str) -> Result<Child,Error> {
        self.run_command("install", args)
    }

    /// Run the uninstall command with the provided arguments
    pub fn uninstall(&self, args: &str) -> Result<Child,Error> {
        self.run_command("uninstall", args)
    }

    /// Run the search command with the provided arguments
    pub fn search(&self, args: &str) -> Result<Child,Error> {
        self.run_command("search", args)
    }

    /// Get the name of the package manager
    pub fn get_name(&self) -> String {
        self.name.to_owned()
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
        let mut command = self.make_command("version").unwrap();
        let output = command.output()?;
        let version_string = String::from_utf8(output.stdout)?;
        Ok(Version::from_str(&version_string))
    }

    /// Read a toml configuration file with a PackageManager description and create a
    /// PackageManager from this info.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<PackageManager,Error> {
        let mut file = File::open(&path)?;

        let mut content = String::new();

        file.read_to_string(&mut content)?;

        let resource = content.as_str().parse::<Value>()?;

        let name: String = String::from(path.as_ref().file_stem().unwrap().to_str().unwrap());

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

       let config_dir: PathBuf = match path.as_ref().parent() {
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
#[derive(Default)]
pub struct Package {
    pub name: String,
    pub owner: PackageManager,
    pub version: Version,
    pub description: String,
}

impl Package {
    /// Return whether the package has the specified name
    pub fn is_called(&self, name: &str) -> bool {
        self.name == name
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
    pub fn get_name(&self) -> String {
        (&self.name).to_owned()
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

/// A simple representation of a version string. For semantic versioning Steve Klabnik's semver
/// crate is preferable. But non-semantic versioning is also permitted in this struct.
#[derive(Debug,Default)]
pub struct Version {
    representation: String,
    semantic: bool
}

impl Version {
    /// Create a version from a string. Checks if the version fits with semantic versioning 2.0.0
    /// and sets semantic to true if it does.
    fn from_str(representation: &str) -> Version {
        let semantic = Version::is_semantic(representation);
        Version {
            representation: String::from(representation),
            semantic,
        }
    }

    /// Get the string representation of the version
    pub fn get_representation(self) -> String {
        self.representation
    }

    /// Change the version along with checking if this new version appears to be semantic
    pub fn set_representation(&mut self, val: String) {
        self.representation = val;
        self.semantic = Version::is_semantic(&self.representation);
    }

    /// Check if a representation appears to be semantic versioning
    pub fn is_semantic(representation: &str) -> bool {
        let re = Version::get_semantic_regex();
        re.is_match(representation)
    }

    fn get_semantic_regex() -> Regex {
        Regex::new(r"^(\d+)\.(\d+)\.(\d+)(?:-([\dA-Za-z-]+(?:\.[\dA-Za-z-]+)*))?(?:\+([\dA-Za-z-]+(?:\.[\dA-Za-z-]+)*))?$").unwrap()
    }

    /// Explicitly set whether the version is semantic. If the version string doesn't pass
    /// is_semantic, then it won't set semantic to true and will return false.
    pub fn set_semantic(&mut self, val: bool) -> Result<(),Error> {
        if val && !Version::is_semantic(&self.representation) {
            bail!("Version does not match semantic structure");
        }
        self.semantic = val;
        Ok(())
    }

    /// Is this a semantic version?
    pub fn get_semantic(self) -> bool {
        self.semantic
    }
    
}

impl PartialEq for Version {
    fn eq(&self, other: &Version) -> bool {
        if self.semantic != other.semantic {
            false
        }
        else if self.semantic && other.semantic {
            let re = Version::get_semantic_regex();
            let self_groups = re.captures(&self.representation).unwrap();
            let other_groups = re.captures(&other.representation).unwrap();
            self_groups.get(1)==other_groups.get(1) && self_groups.get(2)==
                other_groups.get(2) && self_groups.get(3) == other_groups.get(3)
        } else {
            self.representation == other.representation
        }
    }
}
//TODO implement ordering for Versions

//TODO Give info on what files couldn't be read
/// Get a vector of any package managers specified in the given directory.
pub fn get_managers<P: AsRef<Path>>(directory: P, names: &ManagerSpecifier) -> Result<Vec<PackageManager>, Error> {
    let mut result = Vec::new();
    if let Ok(entries) = read_dir(directory) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                let name = entry.file_name();
                if name.to_str().unwrap().ends_with(".toml") {
                    if let Some(stem) = path.file_stem() {
                        //Skip if the name shouldn't be collected
                        match *names {
                            ManagerSpecifier::Excludes(ref set) => {
                                if set.contains(stem.to_str().unwrap()) {
                                    continue;
                                }
                            },
                            ManagerSpecifier::Includes(ref set) => {
                                if !set.contains(stem.to_str().unwrap()) {
                                    continue;
                                }
                            },
                            _ => {}
                        };
                        //Add the package manager to the result
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
    Ok(result)
}

/// Provide a single type to exclude or solely include certain packagemanager names.
pub enum ManagerSpecifier {
    Excludes(HashSet<&'static str>),
    Includes(HashSet<&'static str>),
    Empty,
}

//TODO: provide info on what directories and files weren't read. This should probably be a new
//struct for 1.0.0
/// Read the configuration directories listed from highest precedence to lowest with the option to
/// explicitly exclude or include certain package managers. If the include variant of
/// `ManagerSpecifier` is used then only the specified packagemanager names will be returned if they
/// exist.
/// # Panics
/// If one of the directories can't be read. This should be changed soon to avoid panicking and
/// instead give feedback on what directories and files were and were not read.
pub fn read_config_dirs<P: AsRef<Path>>(directories: Vec<P>, exceptions: &ManagerSpecifier) -> Vec<PackageManager> {
    let mut result: HashSet<PackageManager> = HashSet::new();
    for dir in directories {
        let tmp = get_managers(dir, exceptions);
        let tmp = match tmp {
            Ok(s) => s,
            Err(_e) => panic!("Couldn't get managers from directory"),
        };
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
        let res = version3.set_semantic(false);
        assert!(!res.is_err());
        assert_ne!(version1,version3);
    }

    #[test]
    fn read_toml() {
        let path = PathBuf::from("./test-files");
        let path_vec = vec!(&path);
        let managers = read_config_dirs(path_vec, ManagerSpecifier::Empty);

        let mut expected_managers = HashSet::new();
        expected_managers.insert(PackageManager {
            name: String::from("pacman"),
            version: String::from("./pacman/version.sh"),
            config_dir: PathBuf::from("./test-files"),
            install: Some(String::from("pacman -S")),
            install_local: None,
            remove: Some(String::from("pacman -Rs")),
            remove_local: None,
            search: Some(String::from("pacman -Ss")),
        });
        for man in managers {
            assert!(expected_managers.contains(&man));
        }
    }

    #[test]
    fn cargo_exists() {
        let cargo = PackageManager {
            name: String::from("cargo"),
            version: String::from("./cargo/version.sh"),
            config_dir: PathBuf::from("./test-files/"),
            install: None,
            install_local: Some(String::from("cargo install")),
            remove: None,
            remove_local: Some(String::from("cargo uninstall")),
            search: Some(String::from("cargo search")),
        };
        assert!(cargo.exists(), "cargo apparently isn't installed here?");
    }

    #[test]
    fn commands_fail_gracefully() {
        let fake_manager = PackageManager {
            name: String::from("fake"),
            version: String::from("./fake/version.sh"), //this file is not executable
            config_dir: PathBuf::from("./test-files/"),
            install: Some(String::from("./fake/beelzebub")), //this is a directory
            install_local: Some(String::from("./fake/baphomet")), //this file doesn't exist
            remove: None,
            remove_local: None,
            search: None,
        };
        assert!(&fake_manager.run_command("version", "").is_err());
        assert!(&fake_manager.run_command("install", "").is_err());
        assert!(&fake_manager.run_command("install_local", "").is_err());
    }
}
