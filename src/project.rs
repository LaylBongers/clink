use std::fs::File;
use std::path::PathBuf;
use std::io::Read;
use toml;
use visualstudio::{self, ProjFiles, SlnFile, VcxprojFile, ProjDesc, VcxprojType};
use files;
use dependency::{Dependency};
use tomlvalue::{toml_value_table, toml_value_str, toml_table, toml_read_paths};
use wincanonicalize::wincanonicalize;
use ClinkError;

pub struct Project {
    path: PathBuf, // TODO: Remove path from project, this should be handled by other systems

    name: String,
    class: ProjectClass,
    compile_relative_paths: Vec<PathBuf>,
    include_relative_paths: Vec<PathBuf>,

    dependencies: Vec<Dependency>,
}

impl Project {
    pub fn new(name: String) -> Self {
        Project {
            path: "".into(),

            name: name,
            class: ProjectClass::Library,
            compile_relative_paths: Vec::new(),
            include_relative_paths: Vec::new(),

            dependencies: Vec::new(),
        }
    }

    pub fn open<P: Into<PathBuf>>(path: P) -> Result<Self, ClinkError> {
        // Find the project description file
        let path: PathBuf = path.into();

        // Check if we are a toml path already, if not then add it
        let toml_path = if path.extension().map(|v| v == "toml").unwrap_or(false) {
            path.clone()
        } else {
            files::clone_push_path(&path, "Clink.toml")
        };

        // Read all the text from it
        let mut f = try!(File::open(toml_path).map_err(|_|
            ClinkError::InvalidProjectStructure(path.clone(), "Could not find Clink.toml".into())
        ));
        let mut toml_str = String::new();
        f.read_to_string(&mut toml_str).unwrap();

        // Parse in the toml
        let toml = toml::Parser::new(&toml_str).parse().unwrap();

        // Read in generic information
        let package = try!(toml_value_table(&toml, "package"));
        let name: String = try!(toml_value_str(&package, "name")).into();
        let class: String = try!(toml_value_str(&package, "type")).into();

        // Read in the paths
        let compile_relative_paths = try!(toml_read_paths(&package, "compile", || vec!("./src".into())));
        let include_relative_paths = try!(toml_read_paths(&package, "include", || vec!("./include".into())));

        // Read in all dependencies
        let mut dependencies = Vec::new();
        if let Some(deps_table) = toml.get("dependencies") {
            let deps_table = try!(toml_table(deps_table, "dependencies"));

            for (key, value) in deps_table {
                let dep_path = try!(value.as_str()
                    .ok_or_else(||
                        ClinkError::InvalidProjectFile(format!("{} is invalid type (expected string)", key))
                    )
                );
                dependencies.push(Dependency::at(&path, key.clone(), &dep_path));
            }
        }

        // Store all the information into a helper struct
        Ok(Project {
            path: path,

            name: name,
            class: try!(ProjectClass::parse(&class)),
            compile_relative_paths: compile_relative_paths,
            include_relative_paths: include_relative_paths,

            dependencies: dependencies,
        })
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn class(&self) -> &ProjectClass {
        &self.class
    }

    pub fn to_toml(&self) -> toml::Value {
        let mut table = toml::Table::new();

        // Write out the project's base information
        let mut package = toml::Table::new();
        package.insert("name".into(), toml::Value::String(self.name.clone()));
        package.insert("type".into(), toml::Value::String(self.class.to_string()));
        table.insert("package".into(), toml::Value::Table(package));

        toml::Value::Table(table)
    }

    /// Generate the visual studio solution file for this project.
    pub fn generate_sln(&self) -> Result<(), ClinkError> {
        // Go over this project and all dependencies and generate vcxprojs for them
        let mut projects = Vec::new();
        try!(self.generate_vcxproj_recursive(&mut projects));

        // Write out the sln
        let mut sln = SlnFile::new();
        for proj in projects {
            sln.add_project(proj);
        }
        let filename = format!("{}.sln", self.name);
        sln.write_to(files::clone_push_path(&self.path, &filename));

        Ok(())
    }

    fn generate_vcxproj_recursive(&self, projects: &mut Vec<ProjDesc>) -> Result<(), ClinkError> {
        // Go over all dependencies
        for dep in &self.dependencies {
            // Check if this dependency has already been generated
            if let Some(found) = projects.iter().find(|p| &p.name == dep.name()) {
                // It exists already, make sure the path is the same then skip it
                let mut found_path = found.vcxproj_path.clone();
                found_path.pop();
                if dep.path() != &found_path {
                    // TODO: Improve error handling, add a DependencyConflict error
                    panic!(
                        "Multiple dependencies with same name and different path\n Previous {} at: {}\n Current {} at: {}",
                        found.name, found_path.display(),
                        dep.name(), dep.path().display(),
                    );
                }

                continue;
            }

            // Open the project
            let proj = try!(dep.open());

            // TODO: Implement external dependencies
            if proj.class() == &ProjectClass::External {
                continue;
            }

            // Generate the vcxproj for this project
            try!(proj.generate_vcxproj_recursive(projects));
        }

        // Generate a vcxproj for this project
        let desc = self.generate_vcxproj(projects);

        // Track the generated project
        projects.push(desc);

        Ok(())
    }

    /// Generate the visual studio project file and filters file for this project and return a
    /// descriptor for it.
    // TODO: Change Vec<ProjDesc> to Vec<AvailableDependency> so we can track more data
    pub fn generate_vcxproj(&self, available_dependencies: &Vec<ProjDesc>) -> ProjDesc {
        // Get the project type for our clink project type string
        let class = match &self.class {
            &ProjectClass::Application => VcxprojType::Application,
            &ProjectClass::Library => VcxprojType::StaticLibrary,
            _ => panic!("Internal error: Cannot generate vcxproj for external dependency")
        };

        // Create the project file representation
        let mut vcxproj = VcxprojFile::new(self.name.clone(), class);

        // Add the include folders to the include path
        for path in self.can_includes() {
            vcxproj.add_include_path(path);
        }

        // Find the .hpp and .cpp files the vcxproj needs
        let files = ProjFiles::scan(&self.can_file_paths());
        for file in &files.compile {
            vcxproj.add_compile(file.into());
        }
        for file in &files.include {
            vcxproj.add_include(file.into());
        }

        // Look up and add all the dependencies
        for dep in &self.dependencies {
            // Find the actual project corresponding to the dependency
            let desc = available_dependencies.iter().find(|p| &p.name == dep.name());

            let desc = if let Some(desc) = desc {
                desc
            } else {
                // TODO: Implement external dependencies
                println!("WARNING: External dependency skipped, not yet implemented");
                continue;
            };

            // Add a reference to the dependency
            vcxproj.add_reference(desc.clone());

            // Add the dependency to the include path
            for path in &desc.can_includes {
                vcxproj.add_include_path(path.clone());
            }
        }

        // TODO: Check if the vcxproj needs to be re-generated or not before actually doing it
        // We should only re-generate if the files don't match
        // TODO: Re-use the same project GUID (perhaps use path hashes)
        // Both of these could make use of a Clink.cache file

        // VS Project base path
        let base_path = if self.path.extension().map(|v| v == "toml").unwrap_or(false) {
            let mut base_path = self.path.clone();
            base_path.pop();
            base_path
        } else {
            self.path.clone()
        };

        // Write the vcxproj and vcxproj.filters to disk
        let filename = format!("{}.vcxproj", self.name);
        let desc = vcxproj.write_to(files::clone_push_path(&base_path, &filename));
        let filename = format!("{}.vcxproj.filters", self.name);
        visualstudio::generate_filters(&base_path, &files, files::clone_push_path(&base_path, &filename));

        desc
    }

    pub fn generate_vcxproj_filters(&self) {
        let files = ProjFiles::scan(&self.can_file_paths());
        let filename = format!("{}.vcxproj.filters", self.name);
        visualstudio::generate_filters(&self.path, &files, files::clone_push_path(&self.path, &filename));
    }

    // These "can" prefixed functions are temporary while I'm still moving path out of project
    fn can_file_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for path in &self.compile_relative_paths {
            let append_path = files::clone_push_path(&self.base_path(), path.to_str().unwrap());
            if append_path.exists() {
                paths.push(wincanonicalize(append_path));
            }
        }
        for path in &self.include_relative_paths {
            let append_path = files::clone_push_path(&self.base_path(), path.to_str().unwrap());
            if append_path.exists() {
                paths.push(wincanonicalize(append_path));
            }
        }
        paths
    }

    // These "can" prefixed functions are temporary while I'm still moving path out of project
    fn can_includes(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for path in &self.include_relative_paths {
            let append_path = files::clone_push_path(&self.base_path(), path.to_str().unwrap());
            if append_path.exists() {
                paths.push(wincanonicalize(append_path));
            }
        }
        paths
    }

    fn base_path(&self) -> PathBuf {
        let mut path = self.path.clone();
        if path.extension().map(|v| v == "toml").unwrap_or(false) {
            path.pop();
            path
        } else {
            path
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ProjectClass {
    Application,
    Library,
    External
}

impl ProjectClass {
    pub fn parse(value: &str) -> Result<Self, ClinkError> {
        match value {
            "application" => Ok(ProjectClass::Application),
            "library" => Ok(ProjectClass::Library),
            "external" => Ok(ProjectClass::External),
            v => Err(ClinkError::InvalidProjectFile(format!("\"{}\" is not a valid project type", v)))
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            &ProjectClass::Application => "application".into(),
            &ProjectClass::Library => "library".into(),
            &ProjectClass::External => "external".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Project, ProjectClass};

    #[test]
    fn new_creates_library_with_name() {
        let proj = Project::new("MyProject".into());
        assert_eq!(proj.name(), "MyProject");
        assert_eq!(proj.class(), &ProjectClass::Library);
    }
}
