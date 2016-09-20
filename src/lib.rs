extern crate toml;
extern crate uuid;
extern crate walkdir;
extern crate xml;

pub mod visualstudio;
mod dependency;
mod files;
mod project;
mod tomlvalue;
mod wincanonicalize;

use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

pub use project::Project;

pub enum ClinkError {
    InvalidProjectStructure(PathBuf, String), // Project location, Error string
    InvalidProjectFile(String),
}

impl Display for ClinkError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ClinkError::InvalidProjectStructure(ref loc, ref msg) =>
                write!(f, "Invalid project structure\n Location: {}\n Error: {}", loc.display(), msg),
            ClinkError::InvalidProjectFile(ref msg) =>
                write!(f, "Invalid project file\n Error: {}", msg),
        }
    }
}
