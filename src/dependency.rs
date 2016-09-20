use std::path::PathBuf;
use wincanonicalize::wincanonicalize;
use {Project, ClinkError};

#[derive(Debug)]
pub struct Dependency {
    pub name: String,
    pub path: PathBuf,
}

impl Dependency {
    pub fn at<P: Into<PathBuf>>(proj_path: P, name: String, depstring: &str) -> Self {
        let mut path = proj_path.into();
        path.push(depstring);

        let canonical = wincanonicalize(path);

        Dependency {
            name: name,
            path: canonical,
        }
    }
    
    pub fn open(&self) -> Result<Project, ClinkError> {
        Project::open(&self.path)
    }
}
