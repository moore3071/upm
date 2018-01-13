use std::process::{Command,Child};
use std::fs::File;
use toml::Value;
use std::path::PathBuf;
use std::io::prelude::*;
use failure::Error;

include!(concat!(env!("OUT_DIR"), "/config.rs"));

/// The name of a packagemanager and the various commands 
/// that it may or may not supply to the user. The only 
/// mandatory command is version.
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

    pub fn exists(self) -> bool {
        let mut version_command = self.make_command("version");
        match version_command.status() {
            Ok(s) => s.success(),
            Err(_) => false
        }
    }

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

    pub fn run_command(self, name: &str, args: &str) -> Result<Child,Error> {
        let mut command = self.make_command(name);
        command.args(args.split_whitespace());
        match command.spawn() {
            Ok(child) => Ok(child),
            Err(_) => bail!("Couldn't execute command")
        }
    }

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

    pub fn install(self, args: &str) -> Result<Child,Error> {
        self.run_command("install", args)
    }

    pub fn uninstall(self, args: &str) -> Result<Child,Error> {
        self.run_command("uninstall", args)
    }

    pub fn search(self, args: &str) -> Result<Child,Error> {
        self.run_command("search", args)
    }

    pub fn get_name(self) -> String {
        self.name
    }

    pub fn get_version(self) -> Result<Child,Error> {
        self.run_command("version", "")
    }

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

    pub fn new() -> Package {
        Package{
            name: String::new(),
            owner: PackageManager::new(),
            version: String::new(),
            description: String::new()
        }
    }

    pub fn install(self) -> Result<Child,Error> {
        self.owner.install(&self.name)
    }

    pub fn uninstall(self) -> Result<Child,Error> {
        self.owner.uninstall(&self.name)
    }

    pub fn get_name(self) -> String {
        self.name
    }

    pub fn get_version(self) -> String {
        self.version
    }

    pub fn get_description(self) -> String {
        self.description
    }

    pub fn get_manager(self) -> PackageManager {
        self.owner
    }
}
