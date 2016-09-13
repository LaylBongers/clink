use std::path::PathBuf;

pub fn clone_push_path(path: &PathBuf, appended: &str) -> PathBuf {
    let mut path = path.clone();
    path.push(appended);
    path
}
