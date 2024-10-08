use std::str::FromStr;

use toml::Table;

use crate::core::{device::BackupRequirementClass, project::ProjectTrackingStatus, SecurityLevel};

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

impl<'a> TryRead<'a, SecurityLevel> for &'a Table {
    fn try_read(&'a self, key: &'a str) -> Result<SecurityLevel, String> {
        let v: &str = self.try_read(key)?;
        SecurityLevel::from_str(v)
    }
}

impl<'a> TryRead<'a, BackupRequirementClass> for &'a Table {
    fn try_read(&'a self, key: &'a str) -> Result<BackupRequirementClass, String> {
        let table: &Table = &self.try_read(key)?;
        let target_copies = table.try_read("target_copies")?;
        let target_locations = table.try_read("target_locations")?;
        let min_security_level = table.try_read("min_security_level")?;
        let name: &str = table.try_read("name")?;
        Ok(BackupRequirementClass::new(
            target_copies,
            target_locations,
            min_security_level,
            name.to_string(),
        ))
    }
}

impl<'a> TryRead<'a, ProjectTrackingStatus> for &'a Table {
    fn try_read(&'a self, key: &'a str) -> Result<ProjectTrackingStatus, String> {
        let tracking_status_table = self
            .get(key)
            .ok_or_else(|| "No tracking status saved".to_string())?
            .as_table()
            .ok_or_else(|| "Invalid string for tracking_status".to_string())?;

        let project_type: &str = tracking_status_table.try_read("type")?;

        let status = match project_type {
            "TrackedProject" => {
                let backup_requirement_class =
                    tracking_status_table.try_read("backup_requirement_class")?;

                ProjectTrackingStatus::TrackedProject {
                    backup_requirement_class,
                    last_update: None, // Handle last_update if present in your TOML
                    current_copies: vec![], // Handle current_copies if present in your TOML
                }
            }
            "UntrackedProject" => ProjectTrackingStatus::UntrackedProject,
            "IgnoredProject" => ProjectTrackingStatus::IgnoredProject,
            _ => Err("Unknown tracking status type")?,
        };

        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml::Value;

    // use macro cast from crate
    use crate::{assert_enum_variant, extract_enum_value};

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

    #[test]
    fn test_try_read_security_level() {
        let mut table = Table::new();
        table.insert("key".to_string(), Value::String("Local".to_string()));
        let table = &table;
        let v: SecurityLevel = table.try_read("key").unwrap();
        if let SecurityLevel::Local = v {
        } else {
            panic!("Invalid SecurityLevel");
        }
    }

    #[test]
    fn test_try_read_security_level_missing() {
        let table = &Table::new();
        let v: Result<SecurityLevel, _> = table.try_read("key");
        assert_eq!(v.err().unwrap(), "Missing 'key' field");
    }

    #[test]
    fn test_try_read_security_level_invalid() {
        let mut table = Table::new();
        table.insert("key".to_string(), Value::Integer(42));
        let table = &table;
        let v: Result<SecurityLevel, _> = table.try_read("key");
        assert_eq!(v.err().unwrap(), "Invalid string for 'key'");
    }

    #[test]
    fn test_try_read_security_level_unknown_security_level() {
        let mut table = Table::new();
        table.insert("key".to_string(), Value::String("invalid".to_string()));
        let table = &table;
        let v: Result<SecurityLevel, _> = table.try_read("key");
        assert_eq!(v.err().unwrap(), "Invalid SecurityLevel: invalid");
    }

    #[test]
    fn test_try_read_backup_requirement_class() {
        let mut table = Table::new();
        let mut sub_table = Table::new();
        sub_table.insert("target_copies".to_string(), Value::Integer(42));
        sub_table.insert("target_locations".to_string(), Value::Integer(42));
        sub_table.insert(
            "min_security_level".to_string(),
            Value::String("Local".to_string()),
        );
        sub_table.insert("name".to_string(), Value::String("name".to_string()));
        table.insert("key".to_string(), Value::Table(sub_table));
        let table = &table;
        let v: BackupRequirementClass = table.try_read("key").unwrap();
        assert_eq!(v.get_target_copies(), 42);
        assert_eq!(v.get_target_locations(), 42);
        if let SecurityLevel::Local = v.get_min_security_level() {
        } else {
            panic!("Invalid SecurityLevel");
        }
        assert_eq!(v.get_name(), "name");
    }

    #[test]
    fn test_try_read_backup_requirement_class_missing() {
        let table = &Table::new();
        let v: Result<BackupRequirementClass, _> = table.try_read("key");
        assert_eq!(v.err().unwrap(), "Missing 'key' section");
    }

    #[test]
    fn test_try_read_backup_requirement_class_not_a_section() {
        let mut table = Table::new();
        table.insert("key".to_string(), Value::String("value".to_string()));
        let table = &table;
        let v: Result<BackupRequirementClass, _> = table.try_read("key");
        assert_eq!(v.err().unwrap(), "'key' is not a section");
    }

    #[test]
    fn test_try_read_ignored_project_status() {
        let mut table = Table::new();
        let mut sub_table = Table::new();
        sub_table.insert(
            "type".to_string(),
            Value::String("IgnoredProject".to_string()),
        );
        table.insert("tracking_status".to_string(), Value::Table(sub_table));
        let table = &table;
        assert_enum_variant!(
            table.try_read("tracking_status").unwrap(),
            ProjectTrackingStatus::IgnoredProject
        );
    }

    #[test]
    fn test_try_read_untracked_project_status() {
        let mut table = Table::new();
        let mut sub_table = Table::new();
        sub_table.insert(
            "type".to_string(),
            Value::String("UntrackedProject".to_string()),
        );
        table.insert("tracking_status".to_string(), Value::Table(sub_table));
        let table = &table;
        assert_enum_variant!(
            table.try_read("tracking_status").unwrap(),
            ProjectTrackingStatus::UntrackedProject
        );
    }

    #[test]
    fn test_try_read_tracked_project_status() {
        let toml = r#"
[tracking_status]
type = "TrackedProject"

[tracking_status.backup_requirement_class]
min_security_level = "Local"
name = "name"
target_copies = 42
target_locations = 42
"#;
        let table: Table = toml::from_str(toml).unwrap();
        let table = &table;

        extract_enum_value!(
            table.try_read("tracking_status").unwrap(),
            ProjectTrackingStatus::TrackedProject {
                backup_requirement_class,
                last_update,
                current_copies
            } => {
                assert_eq!(backup_requirement_class.get_target_copies(), 42);
                assert_eq!(backup_requirement_class.get_target_locations(), 42);
                assert_enum_variant!(
                    backup_requirement_class.get_min_security_level(),
                    SecurityLevel::Local
                );
                assert_eq!(backup_requirement_class.get_name(), "name");
                assert_eq!(last_update, None);
                assert_eq!(current_copies.len(), 0);
            }
        );
    }
}
