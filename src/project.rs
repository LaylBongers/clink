use std::fs::File;
use std::path::PathBuf;
use std::io::Read;
use toml;
use vsdata::{self, ProjFiles, SlnFile, VcxprojFile, ProjDesc, VcxprojType};
use files;
use dependency::{Dependency};
use tomlvalue::{toml_value_table, toml_value_str, toml_table};
use ClinkError;

pub struct ClinkProject {
    path: PathBuf,
    name: String,
    class: String,
    dependencies: Vec<Dependency>,
}

impl ClinkProject {
    pub fn open<P: Into<PathBuf>>(path: P) -> Result<Self, ClinkError> {
        // Find the project description file
        let path: PathBuf = path.into();
        let toml_path = files::clone_push_path(&path, "Clink.toml");

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
        Ok(ClinkProject {
            path: path,
            class: class,
            name: name,
            dependencies: dependencies,
        })
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
            // TODO: Implement external dependencies
            if dep.is_external() {
                continue;
            }

            // Check if this dependency has already been generated
            if let Some(found) = projects.iter().find(|p| &p.name == dep.name()) {
                // It exists already, make sure the path is the same then skip it
                let mut found_path = found.vcxproj_path.clone();
                found_path.pop();
                if dep.path() != &found_path {
                    // TODO: Improve error handling, add a DependencyConflict error
                    panic!("Multiple dependencies with same name and different path");
                }

                continue;
            }

            // Open the project and generate it as well
            let proj = try!(dep.open());
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
        let class = match self.class.as_str() {
            "binary" => VcxprojType::Application,
            "library" => VcxprojType::StaticLibrary,
            _ => panic!("Unknown type") // TODO: Catch this in open and handle gracefully
        };

        // Create the project file representation
        let mut vcxproj = VcxprojFile::new(self.name.clone(), class);

        // Add the include folder to the include path
        vcxproj.add_include_path(files::clone_push_path(&self.path, "include"));

        // Find the .hpp and .cpp files the vcxproj needs
        let files = ProjFiles::scan(&self.path);
        for file in &files.compile {
            vcxproj.add_compile(file.into());
        }
        for file in &files.include {
            vcxproj.add_include(file.into());
        }

        // Look up and add all the dependencies
        for dep in &self.dependencies {
            // TODO: Implement external dependencies
            if dep.is_external() {
                continue;
            }

            // Find and add a reference for this dependency
            let desc = available_dependencies.iter().find(|p| &p.name == dep.name())
                .expect("Internal error, dependency not found!");
            vcxproj.add_reference(desc.clone());

            // Add the dependency to the include path
            vcxproj.add_include_path(files::clone_push_path(&dep.path(), "include"));
        }

        // TODO: Check if the vcxproj needs to be re-generated or not before actually doing it
        // We should only re-generate if the files don't match
        // TODO: Re-use the same project GUID
        // Both of these could make use of a Clink.cache file

        // Write the vcxproj and vcxproj.filters to disk
        let filename = format!("{}.vcxproj", self.name);
        let desc = vcxproj.write_to(files::clone_push_path(&self.path, &filename));
        let filename = format!("{}.vcxproj.filters", self.name);
        vsdata::generate_filters(&self.path, &files, files::clone_push_path(&self.path, &filename));

        desc
    }

    pub fn generate_vcxproj_filters(&self) {
        let files = ProjFiles::scan(&self.path);
        let filename = format!("{}.vcxproj.filters", self.name);
        vsdata::generate_filters(&self.path, &files, files::clone_push_path(&self.path, &filename));
    }
}
