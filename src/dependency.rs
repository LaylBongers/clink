use std::path::PathBuf;
use {ClinkProject, ClinkError};

#[derive(Debug)]
pub struct Dependency {
    name: String,
    path: PathBuf,
    is_external: bool,
}

impl Dependency {
    pub fn at<P: Into<PathBuf>>(proj_path: P, name: String, depstring: &str) -> Self {
        let mut path = proj_path.into();
        path.push(depstring);

        let mut canonical = path.canonicalize().unwrap();

        // Grrr windows
        let canonical_str = canonical.to_str().unwrap().to_string();
        canonical = canonical_str.replace("\\\\?\\", "").into();

        // Figure out if it's a native clink or external dependency
        let is_external = canonical.extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or("".into()) == "toml";

        Dependency {
            name: name,
            path: canonical,
            is_external: is_external,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn is_external(&self) -> bool {
        self.is_external
    }

    pub fn open(&self) -> Result<ClinkProject, ClinkError> {
        // Can't open external projects as clink projects
        if self.is_external {
            Err(ClinkError::InvalidProjectStructure(
                self.path.clone(),
                "Cannot open external dependency as project".into()
            ))
        } else {
            ClinkProject::open(&self.path)
        }
    }
}
