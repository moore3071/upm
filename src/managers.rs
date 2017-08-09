use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::fs;

/* Giant struct for holding the different commands of each 
 * package manager.
 */
#[derive(Clone)]
pub struct PackageManager {
    pub name: String,
    pub version: Option<Vec<String>>,
    pub install: Option<Vec<String>>,
    pub install_local: Option<Vec<String>>,
    pub remove: Option<Vec<String>>,
    pub remove_local: Option<Vec<String>>,
}

impl PackageManager {
    
}

/* Turns the arguments for a package manager into a String vector 
 */
fn get_command_as_array(field: String) -> Vec<String> {
   let result: Vec<String> = field.split(" ").map(|s| s.to_string()).collect(); 

   result
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
        let firstchar = line.trim().chars().next();
        //Ignore comment lines
        if firstchar == Some('#') {
            continue;
        } else if firstchar == Some('%') {
            if key.len()>0 && value.len()>0 {
                //Ownership is passed and key and value disappear
                command_map.insert(key, get_command_as_array(value));
            }

            key = String::from(line.trim()).split_off(1);
            value = String::from("");
        } else {
            value.push_str(&line);
        }
    }

    return Ok(make_package_manager(&name, command_map));
}

fn make_package_manager(name: &str, mut command_map: HashMap<String, Vec<String>> ) -> PackageManager {
    let result: PackageManager = PackageManager {
        name: String::from(name),
        version: command_map.remove("version"),
        install: command_map.remove("install"),
        install_local: command_map.remove("install_local"),
        remove: command_map.remove("remove"),
        remove_local: command_map.remove("remove_local"),
    };

    result
}

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
