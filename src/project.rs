use std::fs::File;
use std::path::PathBuf;
use std::io::Read;
use toml;
use files;
use dependency::{Dependency};
use tomlvalue::{toml_value_table, toml_value_str, toml_table, toml_read_paths};
use ClinkError;

pub struct Project {
    pub name: String,
    pub class: ProjectClass,
    pub compile_relative_paths: Vec<PathBuf>,
    pub include_relative_paths: Vec<PathBuf>,

    pub dependencies: Vec<Dependency>,
}

impl Project {
    pub fn new(name: String) -> Self {
        Project {
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
            name: name,
            class: try!(ProjectClass::parse(&class)),
            compile_relative_paths: compile_relative_paths,
            include_relative_paths: include_relative_paths,

            dependencies: dependencies,
        })
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
        assert_eq!(proj.name, "MyProject");
        assert_eq!(proj.class, &ProjectClass::Library);
    }
}
