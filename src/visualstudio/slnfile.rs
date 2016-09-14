use std::path::Path;
use std::fs::File;
use std::io::Write;
use visualstudio::{ProjDesc, escape};

pub struct SlnFile {
    projects: Vec<ProjDesc>,
}

impl SlnFile {
    pub fn new() -> SlnFile {
        SlnFile {
            projects: Vec::new()
        }
    }

    pub fn add_project(&mut self, proj: ProjDesc) {
        self.projects.push(proj);
    }

    pub fn write_to<P: AsRef<Path>>(&self, path: P) {
        let mut file = File::create(path).unwrap();

        // Generic version metadata
        writeln!(file, "Microsoft Visual Studio Solution File, Format Version 12.00").unwrap();
        writeln!(file, "# Visual Studio 14").unwrap();
        writeln!(file, "VisualStudioVersion = 14.0.25420.1").unwrap();
        writeln!(file, "MinimumVisualStudioVersion = 10.0.40219.1").unwrap();

        // Write all projects
        for project in &self.projects {
            writeln!(
                file, // The hardcoded GUID here is the C++ project type
                "Project(\"{}\") = \"{}\", \"{}\", \"{{{}}}\"",
                "{8BC9CEB8-8B4A-11D0-8D11-00A0C91BC942}",
                project.name, escape(format!("{}", project.vcxproj_path.display())), project.uuid.hyphenated()
            ).unwrap();
            writeln!(file, "EndProject").unwrap();
        }
    }
}
