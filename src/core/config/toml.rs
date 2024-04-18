use std::str::FromStr;

use toml::Table;

use crate::models::backup_requirement::{BackupRequirementClass, SecurityLevel};

pub trait TryRead<'a, T> {
    fn try_read(&'a self, key: &'a str) -> Result<T, String>;
}

impl<'a> TryRead<'a, &'a str> for &'a Table {
    fn try_read(&'a self, key: &'a str) -> Result<&'a str, String> {
        self.get(key)
            .ok_or_else(|| format!("Missing '{}' field", key))?
            .as_str()
            .ok_or_else(|| format!("Invalid string for '{}'", key))
    }
}

impl<'a> TryRead<'a, u32> for &'a Table {
    fn try_read(&'a self, key: &'a str) -> Result<u32, String> {
        let v = self
            .get(key)
            .ok_or_else(|| format!("Missing '{}' field", key))?
            .as_integer()
            .ok_or_else(|| format!("Invalid format for '{}'", key))? as u32;
        Ok(v)
    }
}

impl<'a> TryRead<'a, Table> for &'a Table {
    fn try_read(&'a self, key: &'a str) -> Result<Table, String> {
        self.get(key)
            .ok_or_else(|| format!("Missing '{}' section", key))?
            .as_table()
            .ok_or_else(|| format!("'{}' is not a section", key))
            .map(|t| t.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml::Value;

    #[test]
    fn test_try_read_str() {
        let mut table = Table::new();
        table.insert("key".to_string(), Value::String("value".to_string()));
        let table = &table;
        let v: &str = table.try_read("key").unwrap();
        assert_eq!(v, "value");
    }

    #[test]
    fn test_try_read_str_missing() {
        let table = &Table::new();
        let v: Result<&str, _> = table.try_read("key");
        assert_eq!(v.unwrap_err(), "Missing 'key' field");
    }

    #[test]
    fn test_try_read_str_invalid() {
        let mut table = Table::new();
        table.insert("key".to_string(), Value::Integer(42));
        let table = &table;
        let v: Result<&str, _> = table.try_read("key");
        assert_eq!(v.unwrap_err(), "Invalid string for 'key'");
    }

    #[test]
    fn test_try_read_u32() {
        let mut table = Table::new();
        table.insert("key".to_string(), Value::Integer(42));
        let table = &table;
        let v: u32 = table.try_read("key").unwrap();
        assert_eq!(v, 42);
    }

    #[test]
    fn test_try_read_u32_missing() {
        let table = &Table::new();
        let v: Result<u32, _> = table.try_read("key");
        assert_eq!(v.unwrap_err(), "Missing 'key' field");
    }

    #[test]
    fn test_try_read_u32_invalid() {
        let mut table = Table::new();
        table.insert("key".to_string(), Value::String("value".to_string()));
        let table = &table;
        let v: Result<u32, _> = table.try_read("key");
        assert_eq!(v.unwrap_err(), "Invalid format for 'key'");
    }

    #[test]
    fn test_try_read_table() {
        let mut table = Table::new();
        let sub_table = Table::new();
        table.insert("key".to_string(), Value::Table(sub_table.clone()));
        let table = &table;
        let v: Table = table.try_read("key").unwrap();
        assert_eq!(v, sub_table);
    }

    #[test]
    fn test_try_read_table_missing() {
        let table = &Table::new();
        let v: Result<Table, _> = table.try_read("key");
        assert_eq!(v.unwrap_err(), "Missing 'key' section");
    }

    #[test]
    fn test_try_read_table_invalid() {
        let mut table = Table::new();
        table.insert("key".to_string(), Value::String("value".to_string()));
        let table = &table;
        let v: Result<Table, _> = table.try_read("key");
        assert_eq!(v.unwrap_err(), "'key' is not a section");
    }
}
