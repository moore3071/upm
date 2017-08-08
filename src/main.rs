extern crate getopts;

mod install;
mod uninstall;
mod query;

mod managers;
use managers::PackageManager;

use getopts::Options;
use std::env;

fn print_usage(program: String) {
    println!("Usage: {} [--version | {{-h --help}}]", program);
/* Reimplement if options are added for the base command
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
*/
}

fn find_package_managers(possible: Vec<PackageManager>) {

}

//Should call man pages
fn display_help(name: &str) {
    match name {
        "install" => install::display_help(),
        "uninstall" => uninstall::display_help(),
        "query" => query::display_help(),
        _ => {}
            //TODO
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let pkg_name = env!("CARGO_PKG_NAME");
    let pkg_version = env!("CARGO_PKG_VERSION");

    managers::get_managers();

    if args.len() > 1 {
        match &*args[1] {
            "--help" => display_help(&*args[2]),
            "-h" => display_help(&*args[2]),
            "--version" => {
                println!("{} v{}", pkg_name, pkg_version);
            },
            _ => {
                println!("Invalid {} command: {}", pkg_name, args[1]);
                print_usage(program);
            }
        }
    } else {
        print_usage(program);
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
