use std::collections::HashMap;
use std::process::Command;
use std::process::ExitStatus;
use std::error::Error;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::fs;

/// The name of a packagemanager and the various commands 
/// that it may or may not supply to the user.
#[derive(Clone)]
pub struct PackageManager {
    pub name: String,
    pub version: Option<Vec<String>>,
    pub install: Option<Vec<String>>,
    pub install_local: Option<Vec<String>>,
    pub remove: Option<Vec<String>>,
    pub remove_local: Option<Vec<String>>,
}

/// Information on a package from a particular package 
/// manager
pub struct Package {
    pub name: String,
    pub owner: PackageManager,
    pub version: String,
    pub description: Option<String>,
}

impl PackageManager {
    pub fn new() -> PackageManager {
        PackageManager {
            name: String::from(""),
            version: None,
            install: None,
            install_local: None,
            remove: None,
            remove_local: None,
        }
    }

    //TODO check if pm is installed on system
    pub fn exists(self) -> bool {
        self.clone().has_command("version") && run_command(self.clone().version.clone().unwrap()).unwrap().success()
    }

    pub fn has_command(self, name: &str) -> bool {
        match name {
            "version" => self.version.is_some(),
            "install" => self.install.is_some(),
            "install_local" => self.install_local.is_some(),
            "remove" => self.remove.is_some(),
            "remove_local" => self.remove_local.is_some(),
            &_ => false,
        }
    }
}

impl Package {
    /// Return whether the package has the specified name
    pub fn is_called(self, name: &str) -> bool {
        self.name == name
    }
}

/* Turns the arguments for a package manager into a String vector 
 */
fn split_string(field: String) -> Vec<String> {
   let result: Vec<String> = field.split_whitespace().map(|s| s.to_string()).collect(); 

   result
}

//FIXME: Capture output if needed
/* Runs a command given a vector of strings that are the command
 */
pub fn run_command(mut command_array: Vec<String>) -> ::std::io::Result<ExitStatus> {
    Command::new(&command_array.remove(0)).args(command_array).status()
}

/* Loads a package manager from its config file
 */
fn read_manager_file(name: String, path: &Path) -> Result<PackageManager, ::std::io::Error> {
    let file = match File::open(path) {
        Err(why) => {
            return Err(why);
        },
        Ok(file) => file,
    };

    //Holds all of the commands for the manager
    let mut command_map = HashMap::new();

    let reader = BufReader::new(file);
    let lines = reader.lines().map(|l| l.unwrap());

    let mut key: String = String::from("");
    let mut value: String = String::from("");

    for line in lines {
        //FIXME Could possibly use a rustification of getting rid of statements
        let firstchar = line.trim().chars().next();
        //Ignore comment lines
        if firstchar == Some('#') {
            continue;
        } else if firstchar == Some('%') {
            if key.len()>0 && value.len()>0 {
                //Ownership is passed and key and value disappear
                command_map.insert(key, split_string(value));
            }

            key = String::from(line.trim()).split_off(1);
            value = String::from("");
        } else {
            value.push_str(&line);
        }
    }

    return Ok(make_package_manager(&name, command_map));
}

/// Make a PackageManager from a name and hashmap of command
/// names mapped to a vector of strings representing the 
/// command.
pub fn make_package_manager(name: &str, mut command_map: HashMap<String, Vec<String>> ) -> PackageManager {
    PackageManager {
        name: String::from(name),
        version: command_map.remove("version"),
        install: command_map.remove("install"),
        install_local: command_map.remove("install_local"),
        remove: command_map.remove("remove"),
        remove_local: command_map.remove("remove_local"),
    }
}

/// Retrieve the package managers listed in 
/// /var/lib/upm/managers
pub fn get_managers() -> Vec<PackageManager> {
    let mut result_list: Vec<PackageManager> = Vec::new();

    match fs::read_dir("/var/lib/upm/managers/") {
        Ok(paths) => for entry in paths {
            match entry {
                Ok(entry) => {
                    let name: String = entry.file_name().into_string().unwrap();
                    let tmp = entry.path();
                    let path: &Path = tmp.as_path();
                    match read_manager_file(name, path) {
                        Err(why) =>  eprintln!("couldn't open {}: {}", path.display(), why.description()),
                        Ok(manag) => result_list.push(manag),
                    };
                },
                Err(er) => {
                    eprintln!("A package manager couldn't be accessed: {:?}", er.kind());
                },
            };
        },
        Err(e) => {
            panic!("Package managers could not be found at /var/lib/upm/managers/");
        },
    };

    result_list
}

    #[test]
    fn has_command_test() {
        let test: PackageManager = PackageManager {
            name: String::from("empty"),
        	version: Some(vec![String::from("ls")]),
        	install: None,
        	install_local: None,
        	remove: None,
        	remove_local: None,
        };
        assert!(test.clone().has_command("version"));
        assert!(!&test.has_command("install"));
    }

    //TODO
    #[test]
    fn is_called_test() {
        assert!(false);
    }

    #[test]
    fn split_string_t1() {
        let input = String::from("pacman -S -y -u");
        assert_eq!(split_string(input), vec!["pacman", "-S", "-y", "-u"]);
    }

    #[test]
    fn split_string_empty() {
        let input = String::new();
        let res: Vec<String> = Vec::new();
        assert_eq!(split_string(input), res);
    }
    #[test]
    fn split_string_single_word() {
        let input = String::from("npm");
        assert_eq!(split_string(input), vec!["npm"]);
    }

    #[test]
    fn exists_true() {
        //Let's be honest, rustc should exist if we're compiling on here
        let test: PackageManager = PackageManager {
            name: String::from("rust compiler"),
        	version: Some(vec![String::from("rustc")]),
        	install: Some(vec![String::from("rustc")]),
        	install_local: Some(vec![String::from("rustc")]),
        	remove: Some(vec![String::from("rustc")]),
        	remove_local: Some(vec![String::from("rustc")]),
        };
        assert!(test.exists());
    }

    #[test]
    fn exists_false() {
        let test: PackageManager = PackageManager {
            name: String::from("non-existent manager"),
        	version: Some(vec![String::from("acommandthatdoesnotexist")]),
        	install: Some(vec![String::from("acommandthatdoesnotexist")]),
        	install_local: Some(vec![String::from("acommandthatdoesnotexist")]),
        	remove: Some(vec![String::from("acommandthatdoesnotexist")]),
        	remove_local: Some(vec![String::from("acommandthatdoesnotexist")]),
        };
        assert!(!test.exists());
    }

    #[test]
    fn exists_empty() {
        let test: PackageManager = PackageManager {
            name: String::from("empty"),
        	version: None,
        	install: None,
        	install_local: None,
        	remove: None,
        	remove_local: None,
        };
        assert!(!test.exists());
    }

    //TODO
    #[test]
    fn run_command_empty() {

    }

    //TODO
    #[test]
    fn run_command_single_arg() {

    }

    //TODO
    #[test]
    fn run_command_two_args() {

    }

    //TODO
    #[test]
    fn run_command_multi_arg() {

    }
