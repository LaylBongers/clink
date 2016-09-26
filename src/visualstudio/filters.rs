use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;
use uuid::Uuid;
use xml::writer::{XmlEvent, EmitterConfig};
use visualstudio::{ProjFiles};
use wincanonicalize::wincanonicalize;

struct FileEntry {
    location: PathBuf,
    class: String,
    filter: String,
}

fn add_filter_recursive(filters: &mut Vec<String>, filter: &String) {
    // If we're already there, we don't need to do anything
    if filters.iter().find(|f| f == &filter).is_some() {
        return;
    }

    // Add this filter
    filters.push(filter.clone());

    // Check if we have a parenting filter we need to add
    let mut splitted: Vec<&str> = filter.split("\\").collect();
    if splitted.len() > 1 {
        // Merge it together into the next filter to add
        splitted.pop();
        let mut filter = String::new();
        for slice in splitted {
            filter.push_str(slice);
            filter.push('\\');
        }

        // Trim the trailing \
        filter.pop();

        // Pass it on to the next level
        add_filter_recursive(filters, &filter);
    }
}

pub fn generate_filters<Pr: AsRef<Path>, Pt: AsRef<Path>>(project_root: Pr, files: &ProjFiles, target: Pt) {
    // We're interested in all files, but we do need to know what they are
    let mut all_files = Vec::new();
    for file in &files.compile {
        let file = wincanonicalize(file);

        all_files.push(FileEntry {
            location: file,
            class: "ClCompile".into(),
            filter: "".into()
        });
    }
    for file in &files.include {
        let file = wincanonicalize(file);

        all_files.push(FileEntry {
            location: file,
            class: "ClInclude".into(),
            filter: "".into()
        });
    }

    // Get the canonical root of the project
    let root = wincanonicalize(project_root);

    // Generate a list of needed filters and files associated with filters
    let mut filters: Vec<String> = Vec::new();
    for file in &mut all_files {
        // Find the filter string this file needs to be in
        assert!(file.location.starts_with(&root),
            "Internal Error: {:?} does not start with {:?}", file.location, root
        );
        let mut filter = file.location.strip_prefix(&root).unwrap().to_owned();
        filter.pop();
        let filter = format!("{}", filter.display());

        // Add the filter if it doesn't exist yet
        add_filter_recursive(&mut filters, &filter);

        // Store the filter
        file.filter = filter;
    }

    // Generate the actual file
    let mut b = Vec::new();
    {
        let mut w = EmitterConfig::new().perform_indent(true).create_writer(&mut b);

        w.write(XmlEvent::start_element("Project")
            .attr("ToolsVersion", "4.0")
            .attr("xmlns", "http://schemas.microsoft.com/developer/msbuild/2003")
        ).unwrap();

        // Write the filters
        w.write(XmlEvent::start_element("ItemGroup")).unwrap();
        for filter in filters {
            w.write(XmlEvent::start_element("Filter")
                .attr("Include", &filter)
            ).unwrap();

            w.write(XmlEvent::start_element("UniqueIdentifier")).unwrap();
            let uuid = format!("{{{}}}", Uuid::new_v4().hyphenated());
            w.write(XmlEvent::characters(&uuid)).unwrap();
            w.write(XmlEvent::end_element()).unwrap();

            w.write(XmlEvent::end_element()).unwrap();
        }
        w.write(XmlEvent::end_element()).unwrap();

        // Write the files
        w.write(XmlEvent::start_element("ItemGroup")).unwrap();
        for file in all_files {
            let class: &str = &file.class;
            w.write(XmlEvent::start_element(class)
                .attr("Include", &format!("{}", file.location.display()))
            ).unwrap();

            w.write(XmlEvent::start_element("Filter")).unwrap();
            w.write(XmlEvent::characters(&file.filter)).unwrap();
            w.write(XmlEvent::end_element()).unwrap();

            w.write(XmlEvent::end_element()).unwrap();
        }
        w.write(XmlEvent::end_element()).unwrap();

        w.write(XmlEvent::end_element()).unwrap();
    }

    // Finally, write the generated file to disk
    let mut file = File::create(target).unwrap();
    write!(file, "{}", ::std::str::from_utf8(&b).unwrap()).unwrap();
}
