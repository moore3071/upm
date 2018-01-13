extern crate toml;

use std::fs::File;
use std::path::Path;
use std::io::Write;
use std::io::prelude::*;
use std::env;
use toml::Value;

fn main() {
    let mut global_conf_dir = String::from("./");
    let mut secondary_conf_dir = String::from("");

    //Open a config file if one exists and read configuration values
    match File::open("config.toml") {
        Ok(mut file) => {
            let mut conffile = String::new();
            file.read_to_string(&mut conffile).expect("Failure to read config.toml. Check priviledges.");
            let config = conffile.as_str().parse::<Value>().expect("Failure to read config.toml. Is this valid toml?");

            if config.get("global_conf_dir").is_some() {
                global_conf_dir = config.get("global_conf_dir").unwrap().as_str().unwrap().to_owned();
                println!("Read in global_conf_dir as: {}", global_conf_dir);
            }
            if config.get("secondary_conf_dir").is_some() {
                secondary_conf_dir = config.get("secondary_conf_dir").unwrap().as_str().unwrap().to_owned();
                println!("Read in secondary_conf_dir as: {}", secondary_conf_dir);
            }
        },
        Err(_) => {
            println!("No configuration file provided. Using sane defaults.");
        }
    };

    //Write config variables to a .rs file to include in our src files
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("config.rs");
    let mut config_code = File::create(&dest_path).unwrap();

    match write!(config_code, "
            pub fn global_conf_dir() -> &'static str {{
                \"{}\"
            }}
            pub fn secondary_conf_dir() -> &'static str {{
                \"{}\"
            }}",global_conf_dir,secondary_conf_dir) {
        Ok(_) => {
            println!("Configuration settings written");
        },
        Err(e) => {
            panic!("Failure to write configuration file. Err: {}", e);
        }
    }
}
