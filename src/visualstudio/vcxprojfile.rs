use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use uuid::Uuid;
use xml::writer::{XmlEvent, EmitterConfig, EventWriter};
use visualstudio::{ProjDesc};
use visualstudio::sxml::write_simple;

pub struct VcxprojFile {
    name: String,
    class: VcxprojType,
    uuid: Uuid,
    include_path: Vec<PathBuf>,
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
            include_path: Vec::new(),
            include_files: Vec::new(),
            compile_files: Vec::new(),
            references: Vec::new(),
        }
    }

    pub fn add_include_path(&mut self, path: PathBuf) {
        self.include_path.push(path);
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

    pub fn write_to<P: Into<PathBuf>>(&self, target: P) -> ProjDesc {
        let target: PathBuf = target.into();

        // Generate the file's data
        let data = self.generate();

        // Write the generated file to disk
        let mut file = File::create(&target).unwrap();
        write!(file, "{}", ::std::str::from_utf8(&data).unwrap()).unwrap();

        ProjDesc {
            name: self.name.clone(),
            vcxproj_path: target,
            uuid: self.uuid,
            can_includes: self.include_path.clone()
        }
    }

    fn generate(&self) -> Vec<u8> {
        let mut b = Vec::new();
        {
            let mut w = EmitterConfig::new().perform_indent(true).create_writer(&mut b);

            w.write(XmlEvent::start_element("Project")
                .attr("DefaultTargets", "Build")
                .attr("ToolsVersion", "14.0")
                .attr("xmlns", "http://schemas.microsoft.com/developer/msbuild/2003")
            ).unwrap();

            let configs = Self::get_configurations();

            self.write_configurations(&mut w, &configs);
            self.write_globals(&mut w);
            self.write_properties(&mut w, &configs);
            self.write_item_definitions(&mut w, &configs);
            self.write_files(&mut w);
            self.write_references(&mut w);

            // Your guess is as good as mine on what this is for
            self.write_import(&mut w, "$(VCTargetsPath)\\Microsoft.Cpp.targets");

            w.write(XmlEvent::end_element()).unwrap();
        }

        b
    }

    fn write_configurations<W: Write>(&self, w: &mut EventWriter<W>, configs: &Vec<ProjectConfiguration>) {
        w.write(XmlEvent::start_element("ItemGroup")
            .attr("Label", "ProjectConfigurations")
        ).unwrap();

        for config in configs {
            self.write_configuration(w, &config);
        }

        w.write(XmlEvent::end_element()).unwrap();
    }

    fn write_configuration<W: Write>(&self, w: &mut EventWriter<W>, config: &ProjectConfiguration) {
        w.write(XmlEvent::start_element("ProjectConfiguration")
            .attr("Include", &config.full_name())
        ).unwrap();

        write_simple(w, "Configuration", &config.configuration);
        write_simple(w, "Platform", &config.platform);

        w.write(XmlEvent::end_element()).unwrap();
    }

    fn write_globals<W: Write>(&self, w: &mut EventWriter<W>) {
        w.write(XmlEvent::start_element("PropertyGroup")
            .attr("Label", "Globals")
        ).unwrap();

        let uuid = format!("{{{}}}", self.uuid.hyphenated());
        write_simple(w, "ProjectGuid", &uuid);
        write_simple(w, "RootNamespace", &self.name);
        write_simple(w, "WindowsTargetPlatformVersion", "8.1");

        w.write(XmlEvent::end_element()).unwrap();
    }

    fn write_properties<W: Write>(&self, w: &mut EventWriter<W>, configs: &Vec<ProjectConfiguration>) {
        // CPP project template defaults
        self.write_import(w, "$(VCTargetsPath)\\Microsoft.Cpp.Default.props");

        // In-line config properties
        for config in configs {
            self.write_config_properties(w, config);
        }

        // Generic CPP import
        self.write_import(w, "$(VCTargetsPath)\\Microsoft.Cpp.props");

        // Property sheets for every config
        for config in configs {
            self.write_config_property_sheet(w, config);
        }

        // Build properties for every config
        for config in configs {
            self.write_config_build_properties(w, config);
        }
    }

    fn write_config_properties<W: Write>(&self, w: &mut EventWriter<W>, config: &ProjectConfiguration) {
        w.write(XmlEvent::start_element("PropertyGroup")
            .attr("Condition", &format!("'$(Configuration)|$(Platform)'=='{}'", config.full_name()))
            .attr("Label", "Configuration")
        ).unwrap();

        write_simple(w, "ConfigurationType", &self.class.name());

        if config.is_debug() {
            write_simple(w, "UseDebugLibraries", "true");
        } else {
            write_simple(w, "UseDebugLibraries", "false");
        }

        write_simple(w, "PlatformToolset", "v140");

        if config.is_optimized() {
            write_simple(w, "WholeProgramOptimization", "true");
        }

        write_simple(w, "CharacterSet", "MultiByte"); // TODO: Unicode?

        w.write(XmlEvent::end_element()).unwrap();
    }

    fn write_config_property_sheet<W: Write>(&self,
        w: &mut EventWriter<W>, config: &ProjectConfiguration
    ) {
        w.write(XmlEvent::start_element("ImportGroup")
            .attr("Label", "PropertySheets")
            .attr("Condition", &format!("'$(Configuration)|$(Platform)'=='{}'", config.full_name()))
        ).unwrap();

        w.write(XmlEvent::start_element("Import")
            .attr("Project", "$(UserRootDir)\\Microsoft.Cpp.$(Platform).user.props")
            .attr("Condition", "exists('$(UserRootDir)\\Microsoft.Cpp.$(Platform).user.props')")
            .attr("Label", "LocalAppDataPlatform")
        ).unwrap();
        w.write(XmlEvent::end_element()).unwrap();

        w.write(XmlEvent::end_element()).unwrap();
    }

    fn write_config_build_properties<W: Write>(&self,
        w: &mut EventWriter<W>, config: &ProjectConfiguration
    ) {
        // First we need to assemble the information into the format the file expects

        // Assemble the include path string
        let mut include_path_str = String::new();
        for inc in &self.include_path {
            include_path_str.push_str(&format!("{}", inc.display()));
            include_path_str.push(';');
        }
        include_path_str.push_str("$(IncludePath)");

        // Now write that data to the XML output
        w.write(XmlEvent::start_element("PropertyGroup")
            .attr("Condition", &format!("'$(Configuration)|$(Platform)'=='{}'", config.full_name()))
        ).unwrap();

        write_simple(w, "IncludePath", &include_path_str);

        w.write(XmlEvent::end_element()).unwrap();
    }

    fn write_item_definitions<W: Write>(&self,
        w: &mut EventWriter<W>, configs: &Vec<ProjectConfiguration>
    ) {
        for config in configs {
            self.write_item_definition(w, config);
        }
    }

    fn write_item_definition<W: Write>(&self,
        w: &mut EventWriter<W>, config: &ProjectConfiguration
    ) {
        w.write(XmlEvent::start_element("ItemDefinitionGroup")
            .attr("Condition", &format!("'$(Configuration)|$(Platform)'=='{}'", config.full_name()))
        ).unwrap();

        // ClCompile entry, in all targets
        w.write(XmlEvent::start_element("ClCompile")).unwrap();
        write_simple(w, "WarningLevel", "Level3"); // TODO: Make this configurable
        if config.is_optimized() {
            write_simple(w, "Optimization", "MaxSpeed");
            write_simple(w, "FunctionLevelLinking", "true");
        } else {
            write_simple(w, "Optimization", "Disabled");
        }
        write_simple(w, "SDLCheck", "true");
        w.write(XmlEvent::end_element()).unwrap();

        // Link entry, only in optimized
        if config.is_optimized() {
            w.write(XmlEvent::start_element("Link")).unwrap();
            write_simple(w, "EnableCOMDATFolding", "true");
            write_simple(w, "OptimizeReferences", "true");
            w.write(XmlEvent::end_element()).unwrap();
        }

        w.write(XmlEvent::end_element()).unwrap();
    }

    fn write_files<W: Write>(&self, w: &mut EventWriter<W>) {
        // Write the compile files
        w.write(XmlEvent::start_element("ItemGroup")).unwrap();
        for file in &self.compile_files {
            w.write(XmlEvent::start_element("ClCompile")
                .attr("Include", &format!("{}", file.display()))
            ).unwrap();
            w.write(XmlEvent::end_element()).unwrap();
        }
        w.write(XmlEvent::end_element()).unwrap();

        // Write the include files
        w.write(XmlEvent::start_element("ItemGroup")).unwrap();
        for file in &self.include_files {
            w.write(XmlEvent::start_element("ClInclude")
                .attr("Include", &format!("{}", file.display()))
            ).unwrap();
            w.write(XmlEvent::end_element()).unwrap();
        }
        w.write(XmlEvent::end_element()).unwrap();
    }

    fn write_references<W: Write>(&self, w: &mut EventWriter<W>) {
        w.write(XmlEvent::start_element("ItemGroup")).unwrap();

        for reference in &self.references {
            w.write(XmlEvent::start_element("ProjectReference")
                .attr("Include", &format!("{}", reference.vcxproj_path.display()))
            ).unwrap();
            write_simple(w, "Project", &format!("{}", reference.uuid.hyphenated()));
            w.write(XmlEvent::end_element()).unwrap();
        }

        w.write(XmlEvent::end_element()).unwrap();
    }

    fn write_import<W: Write>(&self, w: &mut EventWriter<W>, project: &str) {
        w.write(XmlEvent::start_element("Import")
            .attr("Project", project)
        ).unwrap();

        w.write(XmlEvent::end_element()).unwrap();
    }

    // TODO: Replace this with something configurable
    fn get_configurations() -> Vec<ProjectConfiguration> {
        vec!(
            ProjectConfiguration { configuration: "Debug".into(), platform: "Win32".into() },
            ProjectConfiguration { configuration: "Release".into(), platform: "Win32".into() },
            ProjectConfiguration { configuration: "Debug".into(), platform: "x64".into() },
            ProjectConfiguration { configuration: "Release".into(), platform: "x64".into() },
        )
    }
}

pub enum VcxprojType {
    Application, StaticLibrary
}

impl VcxprojType {
    fn name(&self) -> String {
        match self {
            &VcxprojType::Application => "Application".into(),
            &VcxprojType::StaticLibrary => "StaticLibrary".into(),
        }
    }
}

struct ProjectConfiguration {
    configuration: String, // Debug/Release
    platform: String // Win32/x64
}

impl ProjectConfiguration {
    fn full_name(&self) -> String {
        format!("{}|{}", self.configuration, self.platform)
    }

    fn is_debug(&self) -> bool {
        self.configuration == "Debug"
    }

    fn is_optimized(&self) -> bool {
        !self.is_debug()
    }
}
