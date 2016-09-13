use std::path::Path;
use std::fs::File;
use std::io::Write;
use vsdata::{ProjFiles, escape};

pub fn generate_filters<P: AsRef<Path>>(files: &ProjFiles, path: P) {
    let mut filedata = String::from(include_str!("./template.vcxproj.filters"));

    // TODO: Use an XML library to clean this up and make it safer

    // Write the compile files
    let mut compiled = String::new();
    for filename in &files.compile {
        let filename = escape(format!("{}", filename.display()));
        compiled.push_str(&format!("<ClCompile Include=\"{}\">\n", filename));
        compiled.push_str("<Filter>Source Files</Filter>\n");
        compiled.push_str("</ClCompile>\n");
    }
    filedata = filedata.replace("{COMPILE_FILES}", &compiled);

    // Write the include files
    let mut include = String::new();
    for filename in &files.include {
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
