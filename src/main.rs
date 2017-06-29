/* Usage: upm install [{-m --manager} <package-manager>] <package-name(s)>
 * Usage: upm uninstall [{-m --manager} <package-manager>] <package-name(s)>
 * Usage: upm query [{-m --manager} <package-manager>] [<package-name(s)>]
 * Usage: upm [--version | {-h --help}]
 */

extern crate getopts;

use getopts::Options;
use std::env;

fn do_work(inp: &str, out: Option<String>) {
    println!("{}", inp);
    match out {
        Some(x) => println!("{}", x),
        None => println!("No Output"),
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn find_package_managers() {

}

//Should call man pages
fn display_help(/*name: &str*/) {

}

fn no_args() {

}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    if args.len() > 1 {
        match &*args[1] {
            "--help" => display_help(),
            "-h" => display_help(),
            "--version" => {
                println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            }
            _ => no_args()
        }
    } else {
        display_help();
    }

/*
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
