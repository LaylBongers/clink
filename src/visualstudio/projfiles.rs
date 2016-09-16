use std::path::PathBuf;
use walkdir::WalkDir;
use wincanonicalize::wincanonicalize;

pub struct ProjFiles {
    pub compile: Vec<PathBuf>,
    pub include: Vec<PathBuf>,
}

impl ProjFiles {
    pub fn scan(paths: &Vec<PathBuf>) -> ProjFiles {
        let mut compile = Vec::new();
        let mut include = Vec::new();

        for path in paths {
            for file in WalkDir::new(path) {
                let file = file.unwrap();
                let file = file.path();

                // Only go over files
                if !file.is_file() { continue; }

                let file = wincanonicalize(file);

                // Different behavior for different files
                let extension: String = file.extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or("".into());

                if extension == "cpp" || extension == "c" || extension == "cc" {
                    // Skip files we already have
                    if compile.iter().find(|f| f == &&file).is_some() {
                        continue;
                    }

                    compile.push(file);
                }
                else if extension == "hpp" || extension == "h" {
                    // Skip files we already have
                    if include.iter().find(|f| f == &&file).is_some() {
                        continue;
                    }

                    include.push(file);
                }

                // Ignore anything else
            }
        }

        ProjFiles {
            compile: compile,
            include: include,
        }
    }
}
