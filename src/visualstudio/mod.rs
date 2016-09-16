mod filters;
mod projfiles;
mod slnfile;
mod vcxprojfile;

use std::path::PathBuf;
use uuid::Uuid;

pub use self::filters::generate_filters;
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
