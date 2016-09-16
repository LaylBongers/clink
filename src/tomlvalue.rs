use std::path::PathBuf;
use toml::{Table, Value};
use ClinkError;

pub fn toml_value<'a>(table: &'a Table, value_name: &str) -> Result<&'a Value, ClinkError> {
    table.get(value_name).ok_or_else(||
        ClinkError::InvalidProjectFile(format!("Cannot find \"{}\" in toml", value_name))
    )
}

pub fn toml_table<'a>(value: &'a Value, value_name: &str) -> Result<&'a Table, ClinkError> {
    value.as_table()
        .ok_or_else(||
            ClinkError::InvalidProjectFile(format!("{} is invalid type (expected table)", value_name))
        )
}

pub fn toml_value_table<'a>(table: &'a Table, value_name: &str) -> Result<&'a Table, ClinkError> {
    let value = try!(toml_value(table, value_name));
    toml_table(value, value_name)
}

pub fn toml_str<'a>(value: &'a Value, value_name: &str) -> Result<&'a str, ClinkError> {
    value.as_str()
        .ok_or_else(||
            ClinkError::InvalidProjectFile(format!("{} is invalid type (expected string)", value_name))
        )
}

pub fn toml_value_str<'a>(table: &'a Table, value_name: &str) -> Result<&'a str, ClinkError> {
    let value = try!(toml_value(table, value_name));
    toml_str(value, value_name)
}

pub fn toml_slice<'a>(value: &'a Value, value_name: &str) -> Result<&'a [Value], ClinkError> {
    value.as_slice()
        .ok_or_else(||
            ClinkError::InvalidProjectFile(format!("{} is invalid type (expected array)", value_name))
        )
}

pub fn toml_read_paths<'a, F: Fn() -> Vec<PathBuf>>(table: &'a Table, value_name: &str, default: F)
    -> Result<Vec<PathBuf>, ClinkError> {
    let paths = if let Some(paths) = table.get("include") {
        let mut path_entries: Vec<PathBuf> = Vec::new();

        for path in try!(toml_slice(paths, value_name)) {
            let value = try!(toml_str(path, "include_value"));
            path_entries.push(value.into());
        }

        path_entries
    } else {
        default()
    };

    Ok(paths)
}
