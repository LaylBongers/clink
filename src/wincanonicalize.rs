use std::path::{Path, PathBuf};

pub fn wincanonicalize<P: AsRef<Path>>(value: P) -> PathBuf {
    let mut canonical = value.as_ref().canonicalize().unwrap();

    // Grrr windows
    let canonical_str = canonical.to_str().unwrap().to_string();
    canonical = canonical_str.replace("\\\\?\\", "").into();

    canonical
}
