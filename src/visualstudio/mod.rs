mod filters;
mod projfiles;
mod slnfile;
mod vcxprojfile;
mod sxml;

use std::path::PathBuf;
use uuid::Uuid;
use project::{Project, ProjectClass};
use files;
use wincanonicalize::wincanonicalize;
use ClinkError;
use self::filters::generate_filters;

pub use self::projfiles::ProjFiles;
pub use self::slnfile::SlnFile;
pub use self::vcxprojfile::{VcxprojFile, VcxprojType};

#[derive(Clone, Debug)]
pub struct ProjDesc {
    pub name: String,
    pub vcxproj_path: PathBuf,
    pub uuid: Uuid,
    pub can_includes: Vec<PathBuf>,
}

pub fn escape(raw: String) -> String {
    let mut escaped = String::new();

    for c in raw.chars() {
        if c == '\"' {
            escaped.push_str("&quot;");
        } else {
            escaped.push(c);
        }
    }

    escaped
}

pub fn generate_vcxproj_filters<P: Into<PathBuf>>(project: &Project, target_path: P) {
    let target_path = target_path.into();
    let files = ProjFiles::scan(&can_file_paths(&project, &target_path));
    let filename = format!("{}.vcxproj.filters", project.name);
    generate_filters(&target_path, &files, files::clone_push_path(&target_path, &filename));
}

/// Generate the visual studio solution file for this project.
pub fn generate_sln<P: Into<PathBuf>>(project: &Project, target_path: P) -> Result<(), ClinkError> {
    let target_path = target_path.into();

    // Go over this project and all dependencies and generate vcxprojs for them
    let mut projects = Vec::new();
    try!(generate_vcxproj_recursive(project, &target_path, &mut projects));

    // Write out the sln
    let mut sln = SlnFile::new();
    for proj in projects {
        sln.add_project(proj);
    }
    let filename = format!("{}.sln", project.name);
    sln.write_to(files::clone_push_path(&target_path, &filename));

    Ok(())
}

fn can_file_paths(project: &Project, source_path: &PathBuf) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for path in &project.compile_relative_paths {
        let append_path = files::clone_push_path(source_path, path.to_str().unwrap());
        if append_path.exists() {
            paths.push(wincanonicalize(append_path));
        }
    }
    for path in &project.include_relative_paths {
        let append_path = files::clone_push_path(source_path, path.to_str().unwrap());
        if append_path.exists() {
            paths.push(wincanonicalize(append_path));
        }
    }
    paths
}

fn generate_vcxproj_recursive(project: &Project, target_path: &PathBuf, projects: &mut Vec<ProjDesc>) -> Result<(), ClinkError> {
    // Go over all dependencies
    for dep in &project.dependencies {
        // Check if this dependency has already been generated
        if let Some(found) = projects.iter().find(|p| p.name == dep.name) {
            // It exists already, make sure the path is the same then skip it
            let mut found_path = found.vcxproj_path.clone();
            found_path.pop();
            if dep.path != found_path {
                // TODO: Improve error handling, add a DependencyConflict error
                panic!(
                    "Multiple dependencies with same name and different path\n Previous {} at: {}\n Current {} at: {}",
                    found.name, found_path.display(),
                    dep.name, dep.path.display(),
                );
            }

            continue;
        }

        // Open the project
        let proj = try!(dep.open());

        // TODO: Implement external dependencies
        if proj.class == ProjectClass::External {
            continue;
        }

        // Get the base path of the project we've opened
        let mut path = dep.path.clone();
        path = if path.extension().map(|v| v == "toml").unwrap_or(false) {
            path.pop();
            path
        } else {
            path
        };

        // Generate the vcxproj for this project
        try!(generate_vcxproj_recursive(&proj, &path, projects));
    }

    // Generate a vcxproj for this project
    let desc = generate_vcxproj(project, target_path, projects);

    // Track the generated project
    projects.push(desc);

    Ok(())
}

/// Generate the visual studio project file and filters file for this project and return a
/// descriptor for it.
// TODO: Change Vec<ProjDesc> to Vec<AvailableDependency> so we can track more data
fn generate_vcxproj(project: &Project, target_path: &PathBuf, available_dependencies: &Vec<ProjDesc>) -> ProjDesc {
    // Get the project type for our clink project type string
    let class = match &project.class {
        &ProjectClass::Application => VcxprojType::Application,
        &ProjectClass::Library => VcxprojType::StaticLibrary,
        _ => panic!("Internal error: Cannot generate vcxproj for external dependency")
    };

    // Create the project file representation
    let mut vcxproj = VcxprojFile::new(project.name.clone(), class);

    // Add the include folders to the include path
    for path in can_includes(project, target_path) {
        vcxproj.add_include_path(path);
    }

    // Find the .hpp and .cpp files the vcxproj needs
    let files = ProjFiles::scan(&can_file_paths(&project, target_path));
    for file in &files.compile {
        vcxproj.add_compile(file.into());
    }
    for file in &files.include {
        vcxproj.add_include(file.into());
    }

    // Look up and add all the dependencies
    for dep in &project.dependencies {
        // Find the actual project corresponding to the dependency
        let desc = available_dependencies.iter().find(|p| p.name == dep.name);

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

    // Write the vcxproj and vcxproj.filters to disk
    let filename = format!("{}.vcxproj", project.name);
    let desc = vcxproj.write_to(files::clone_push_path(&target_path, &filename));
    let filename = format!("{}.vcxproj.filters", project.name);
    generate_filters(&target_path, &files, files::clone_push_path(&target_path, &filename));

    desc
}

// These "can" prefixed functions are temporary while I'm still moving path out of project
fn can_includes(project: &Project, target_path: &PathBuf) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for path in &project.include_relative_paths {
        let append_path = files::clone_push_path(&target_path, path.to_str().unwrap());
        if append_path.exists() {
            paths.push(wincanonicalize(append_path));
        }
    }
    paths
}
