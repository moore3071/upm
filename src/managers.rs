use std::process::{Command,Child};
use std::fs::File;
use toml::Value;
use std::path::PathBuf;
use std::io::prelude::*;
use failure::Error;
use regex::Regex;

include!(concat!(env!("OUT_DIR"), "/config.rs"));

/// The name of a packagemanager and the various commands 
/// that it may or may not supply to the user as Strings.
/// The only mandatory command is version, meanwhile a pathbuf
/// is included to allow for external scripts to be called.
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
    /// Create a blank PackageManager with no commands
    /// (version and name are empty Strings).
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

    /// Check if the PackageManager is installed by seeing if
    /// the version command exits with a status code of 0.
    pub fn exists(self) -> bool {
        let mut version_command = self.make_command("version");
        match version_command.status() {
            Ok(s) => s.success(),
            Err(_) => false
        }
    }

    /// Check if the specified command field of the struct
    /// is some
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

    /// Attempt to run the PackageManager command specified
    /// by name. Arguments can be supplied with the args
    /// parameter.
    pub fn run_command(self, name: &str, args: &str) -> Result<Child,Error> {
        let mut command = self.make_command(name);
        command.args(args.split_whitespace());
        match command.spawn() {
            Ok(child) => Ok(child),
            Err(_) => bail!("Couldn't execute command")
        }
    }

    /// Turns the String that describes a command into a
    /// std::process::Command struct.
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

    /// Get the directory of the configuration file that
    /// describes the PackageManager
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
        Ok(Version::from_string(version_string))
    }

    /// Read a toml configuration file with a PackageManager
    /// description and create a PackageManager from this info.
    pub fn from_file(path: PathBuf) -> Result<PackageManager,Error> {
        let mut file = File::open(&path)?;

        let mut content = String::new();

        file.read_to_string(&mut content)?;

        let resource = content.as_str().parse::<Value>()?;

        let name: String = match resource.get("name") {
            Some(s) => String::from(s.as_str().unwrap()),
            None => bail!("Package manager name not provided in config")
        };

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

/// Information on a package from a particular package 
/// manager
pub struct Package {
    pub name: String,
    pub owner: PackageManager,
    pub version: String,
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
            version: String::new(),
            description: String::new()
        }
    }

    /// Call install from the PackageManager pointed
    /// to by owner.
    pub fn install(self) -> Result<Child,Error> {
        self.owner.install(&self.name)
    }

    /// Call uninstall from the PackageManager pointed
    /// to by owner.
    pub fn uninstall(self) -> Result<Child,Error> {
        self.owner.uninstall(&self.name)
    }

    /// Return the package name
    pub fn get_name(self) -> String {
        self.name
    }

    /// Return the package version
    pub fn get_version(self) -> String {
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

/// A simple representation of a version string
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

    /// Create a version from a string. Checks if the version fits
    /// with semantic versioning 2.0.0 and sets semantic to true if
    /// it does.
    pub fn from_string(representation: String) -> Version {
        let semantic = Version::is_semantic(&representation);
        Version {
            representation,
            semantic,
        }
    }

    /// Check if a representation fits with semantic versioning
    fn is_semantic(representation: &str) -> bool {
        let re = Regex::new(r"^(\d+)\.(\d+)\.(\d+)(?:-([\dA-Za-z-]+(?:\.[\dA-Za-z-]+)*))?(?:\+([\dA-Za-z-]+(?:\.[\dA-Za-z-]+)*))?$").unwrap();
        re.is_match(representation)
    }

    /// Explicitly set whether the version is semantic
    pub fn set_semantic(mut self, val: bool) {
        self.semantic = val;
    }
    
}
