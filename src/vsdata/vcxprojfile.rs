use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;
use uuid::Uuid;
use vsdata::{ProjDesc, escape};

pub enum VcxprojType {
    Application, StaticLibrary
}

pub struct VcxprojFile {
    name: String,
    class: VcxprojType,
    uuid: Uuid,
    include_files: Vec<PathBuf>,
    compile_files: Vec<PathBuf>,
    references: Vec<ProjDesc>,
}

impl VcxprojFile {
    pub fn new(name: String, class: VcxprojType) -> Self {
        VcxprojFile {
            name: name,
            class: class,
            uuid: Uuid::new_v4(),
            include_files: Vec::new(),
            compile_files: Vec::new(),
            references: Vec::new(),
        }
    }

    pub fn add_include(&mut self, path: PathBuf) {
        self.include_files.push(path);
    }

    pub fn add_compile(&mut self, path: PathBuf) {
        self.compile_files.push(path);
    }

    pub fn add_reference(&mut self, desc: ProjDesc) {
        self.references.push(desc);
    }

    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn write_to<P: Into<PathBuf>>(&self, path: P) -> ProjDesc {
        let path: PathBuf = path.into();
        let mut filedata = String::from(include_str!("./template.vcxproj"));

        // TODO: Use an XML library to clean this up and make it safer

        // Inject the generic project data
        filedata = filedata.replace("{NAME}", &self.name);

        // Inject the project type
        let class = match self.class {
            VcxprojType::Application => "Application",
            VcxprojType::StaticLibrary => "StaticLibrary",
        };
        filedata = filedata.replace("{TYPE}", class);

        // Inject the GUID
        let uuid = format!("{{{}}}", self.uuid.hyphenated());
        filedata = filedata.replace("{UUID}", &uuid);

        // Write the compile files
        let mut compiled = String::new();
        for filename in &self.compile_files {
            let filename = escape(format!("{}", filename.display()));
            compiled.push_str(&format!("<ClCompile Include=\"{}\" />\n", filename));
        }
        filedata = filedata.replace("{COMPILE_FILES}", &compiled);

        // Write the include files
        let mut include = String::new();
        for filename in &self.include_files {
            let filename = escape(format!("{}", filename.display()));
            include.push_str(&format!("<ClInclude Include=\"{}\" />\n", filename));
        }
        filedata = filedata.replace("{INCLUDE_FILES}", &include);

        // Write the references
        let mut references = String::new();
        for reference in &self.references {
            let path = escape(format!("{}", reference.vcxproj_path.display()));
            references.push_str(&format!("<ProjectReference Include=\"{}\">\n", path));
            references.push_str(&format!("<Project>{}</Project>\n", reference.guid));
            references.push_str("</ProjectReference>\n");
        }
        filedata = filedata.replace("{REFERENCES}", &references);

        // Finally, write the generated file to disk
        let mut file = File::create(&path).unwrap();
        write!(file, "{}", filedata).unwrap();

        ProjDesc {
            name: self.name.clone(),
            vcxproj_path: path,
            guid: format!("{{{}}}", self.uuid()),
        }
    }

    pub fn write_filters_to<P: AsRef<Path>>(&self, path: P) {
        let mut filedata = String::from(include_str!("./template.vcxproj.filters"));

        // TODO: Use an XML library to clean this up and make it safer

        // Write the compile files
        let mut compiled = String::new();
        for filename in &self.compile_files {
            let filename = escape(format!("{}", filename.display()));
            compiled.push_str(&format!("<ClCompile Include=\"{}\">\n", filename));
            compiled.push_str("<Filter>Source Files</Filter>\n");
            compiled.push_str("</ClCompile>\n");
        }
        filedata = filedata.replace("{COMPILE_FILES}", &compiled);

        // Write the include files
        let mut include = String::new();
        for filename in &self.include_files {
            let filename = escape(format!("{}", filename.display()));
            include.push_str(&format!("<ClInclude Include=\"{}\">\n", filename));
            include.push_str("<Filter>Header Files</Filter>\n");
            include.push_str("</ClInclude>\n");
        }
        filedata = filedata.replace("{INCLUDE_FILES}", &include);

        // Finally, write the generated file to disk
        let mut file = File::create(path).unwrap();
        write!(file, "{}", filedata).unwrap();
    }
}
