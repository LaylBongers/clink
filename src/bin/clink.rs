extern crate docopt;
extern crate rustc_serialize;
extern crate clink;

use std::process;
use std::io::{self, Write};
use docopt::Docopt;
use clink::{ClinkProject, ClinkError};

const USAGE: &'static str = "
A simple C++ build system generator
Usage:
    clink [<command> [<args>...]]
Some common clink commands are:
    generate    Generate Visual Studio files for the current project (default)
    init        Create a new clink project in the current directory
";


#[derive(Debug, RustcDecodable)]
pub struct Flags {
    arg_command: Option<String>,
    arg_args: Vec<String>,
}

fn main() {
    let args: Flags = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    let command = args.arg_command.unwrap_or("generate".into());
    let command_func: fn() -> Result<(), ClinkError> = match command.as_ref() {
        "generate" => try_generate,
        "init" => try_init,
        _ => {
            write!(io::stderr(), "Error: Unknown command \"{}\"\n", command).unwrap();
            process::exit(1);
        }
    };

    command_func().unwrap_or_else(|e| {
        write!(io::stderr(), "Error: {}\n", e).unwrap();
        process::exit(1);
    });
}

fn try_generate() -> Result<(), ClinkError> {
    let proj = try!(ClinkProject::open("./"));
    try!(proj.generate_sln());

    Ok(())
}

fn try_init() -> Result<(), ClinkError> {
    unimplemented!()
}
