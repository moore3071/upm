extern crate getopts;

mod install;
mod uninstall;
mod query;

mod managers;
use managers::PackageManager;

use getopts::Options;
use std::env;

use std::process::Command;
use std::process::ExitStatus;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] [install <pkgname> | uninstall <pkgname> | query <pkgname>]", program);
    print!("{}", opts.usage(&brief));
}

/* Checks what package managers are on the system by calling
 * the version command
 */
fn find_package_managers(possible: &Vec<PackageManager>) -> Vec<PackageManager> {
    let mut result: Vec<PackageManager> = Vec::new();
    for pack in possible {
        let ver = &pack.version;
        let ver = ver.clone();
        let tmp = ver.unwrap();
        match run_command(tmp.clone()) {
            Ok(_) => {
                result.push(pack.clone());
            },
            Err(e) => {
            },
        };
    }

    result
}

/* Runs a command given a vector of strings that are the command
 */
fn run_command(command_array: Vec<String>) -> std::io::Result<ExitStatus> {
    Command::new(&command_array[0]).args(command_array).status()
}

//Should call man pages
fn display_help(args: &Vec<String>) {
    let name: &str = " ";
    if args.len() > 2 {
        let name = &args[2];
    }
    match name {
        "install" => {
            Command::new("man").arg("upm-install").status();
        },
        "uninstall" => {
            Command::new("man").arg("upm-uninstall").status();
        },
        "query" => {
            Command::new("man").arg("upm-query").status();
        },
        _ => {
            Command::new("man").arg("upm").status();
        },
    };
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let pkg_name = env!("CARGO_PKG_NAME");
    let pkg_version = env!("CARGO_PKG_VERSION");

//    let managers: Vec<PackageManager> = managers::get_managers();

    if args.len() > 1 {
        match &*args[1] {
            "--help" => display_help(&args),
            "-h" => display_help(&args),
            "--version" => {
                println!("{} v{}", pkg_name, pkg_version);
            },
            "query" => {

            },
            "install" => {

            },
            "uninstall" => {

            },
            _ => {
                println!("Invalid {} command: {}", pkg_name, args[1]);
//                print_usage(&program, opts);
            },
        }
    } else {
//        print_usage(program);
    }

/* Stolen from the getopts crate documentation
    let mut opts = Options::new();
    opts.optopt("o", "", "set output file name", "NAME");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let output = matches.opt_str("o");
    let input = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };
    do_work(&input, output);
*/
}
