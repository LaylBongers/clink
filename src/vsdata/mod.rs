mod slnfile;
mod vcxprojfile;

use std::path::PathBuf;

pub use self::slnfile::SlnFile;
pub use self::vcxprojfile::{VcxprojFile, VcxprojType};

#[derive(Clone, Debug)]
pub struct ProjDesc {
    pub name: String,
    pub vcxproj_path: PathBuf,
    pub guid: String,
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
