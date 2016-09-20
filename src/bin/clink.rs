extern crate docopt;
extern crate rustc_serialize;
extern crate clink;

use std::process;
use std::io::{self, Write};
use docopt::Docopt;
use clink::{Project, ClinkError};

const USAGE: &'static str = "
A simple C++ build system generator

Usage:
    clink [<command> [<args>...]]

Some common clink commands are:
    generate    Generate Visual Studio files for the current project (default)
    filters     Generate just the .vcxproj.fiters file for the current project
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
        "filters" => try_filters,
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
    let proj = try!(Project::open("./"));
    try!(proj.generate_sln("./"));

    Ok(())
}

fn try_filters() -> Result<(), ClinkError> {
    let proj = try!(Project::open("./"));
    proj.generate_vcxproj_filters("./");

    Ok(())
}

fn try_init() -> Result<(), ClinkError> {
    // TODO: Verify the project doesn't already exist

    // Assume the name from the current directory
    // TODO: Move this into a utility in clink
    use std::path::PathBuf;
    let path = PathBuf::from("./").canonicalize().unwrap();
    let top = path.iter().last().unwrap();

    // Create the new project and write it to a file
    // TODO: Clean up
    let proj = Project::new(top.to_str().unwrap().into());
    let toml = proj.to_toml();
    use std::fs::File;
    use std::io::Write;
    let mut file = File::create("./Clink.toml").unwrap();
    write!(file, "{}", toml).unwrap();

    Ok(())
}
