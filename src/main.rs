#[macro_use]
extern crate clap;

mod managers;
use managers::PackageManager;
use managers::Package;

use clap::{Arg, App, SubCommand, AppSettings};

use std::collections::HashMap;

/// Checks what package managers are on the system by calling
/// the version command
fn find_package_managers() {
    //TODO
}

fn install() {
    //TODO
    
}

fn query() {
    //TODO
}

fn uninstall() {
//TODO
}

fn main() {
    let managers: Vec<PackageManager> = managers::get_managers();

    let managers_arg = Arg::with_name("manager")
         .short("m")
         .long("manager")
         .help("Specifies the package managers to search for the package in")
         .value_name("MANAGER")
         .takes_value(true);
    let exclude_managers = Arg::with_name("excludes managers")
        .long("exclude-managers")
        .help("Specifies package managers to not use")
        .takes_value(true)
        .value_name("MANAGER");

    //Clap is awesome! 
    let matches = App::new("universal package manager")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Universal package manager provides a single interface for basic \npackage management across multiple package managers.")
        .global_setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::with_name("list managers")
             .long("list-managers")
             .help("list the package managers available on this system"))
        .subcommand(SubCommand::with_name("query")
                    .about("Search for a package")
                    .arg(&managers_arg)
                    .arg(&exclude_managers))
        .subcommand(SubCommand::with_name("install")
                    .about("Search for a package and then install via a chosen package manager")
                    .arg(&managers_arg)
                    .arg(&exclude_managers))
        .subcommand(SubCommand::with_name("uninstall")
                    .about("Search for an installed package and then uninstall it")
                    .arg(&managers_arg)
                    .arg(&exclude_managers))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("query") {
        query()
    } else if let Some(matches) = matches.subcommand_matches("install") {
        install()
    } else if let Some(matches) = matches.subcommand_matches("uninstall") {
        uninstall()
    } else if matches.is_present("list managers") {
        //TODO
    }
}

