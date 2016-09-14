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

pub fn toml_value_str<'a>(table: &'a Table, value_name: &str) -> Result<&'a str, ClinkError> {
    try!(toml_value(table, value_name))
        .as_str()
        .ok_or_else(||
            ClinkError::InvalidProjectFile(format!("{} is invalid type (expected string)", value_name))
        )
}
