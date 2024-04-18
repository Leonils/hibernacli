use std::time::UNIX_EPOCH;

use toml::Table;

use crate::{
    core::global_config::GlobalConfig,
    models::{
        backup_requirement::BackupRequirementClass,
        project::{Project, ProjectTrackingStatus},
    },
};

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
struct PartiallyParsedGlobalConfig {
    devices: Option<Vec<Table>>,
    projects: Option<Vec<Table>>,
}

pub trait ToTomlTable {
    fn to_toml_table(&self) -> Table;
}

pub trait ToToml {
    fn to_toml(&self) -> Result<String, String>;
}

impl ToToml for GlobalConfig {
    fn to_toml(&self) -> Result<String, String> {
        let project_tables = self
            .get_projects_iter()
            .map(|project| project.to_toml_table())
            .collect::<Vec<_>>();

        let device_tables = self
            .get_devices_iter()
            .map(|device| device.to_toml_table())
            .collect::<Vec<_>>();

        let config_toml = toml::to_string(&PartiallyParsedGlobalConfig {
            devices: if device_tables.is_empty() {
                None
            } else {
                Some(device_tables)
            },
            projects: if project_tables.is_empty() {
                None
            } else {
                Some(project_tables)
            },
        })
        .map_err(|e| e.to_string())?;

        Ok(config_toml)
    }
}

impl ToTomlTable for BackupRequirementClass {
    fn to_toml_table(&self) -> Table {
        let mut table = Table::new();
        table.insert(
            "target_copies".to_string(),
            toml::Value::Integer(self.get_target_copies() as i64),
        );
        table.insert(
            "target_locations".to_string(),
            toml::Value::Integer(self.get_target_locations() as i64),
        );
        table.insert(
            "min_security_level".to_string(),
            toml::Value::String(self.get_min_security_level().to_string()),
        );
        table.insert(
            "name".to_string(),
            toml::Value::String(self.get_name().clone()),
        );

        table
    }
}

impl ToToml for BackupRequirementClass {
    fn to_toml(&self) -> Result<String, String> {
        let toml_value = self.to_toml_table();
        let toml = toml_value.to_string();
        Ok(toml)
    }
}

impl ToTomlTable for ProjectTrackingStatus {
    fn to_toml_table(&self) -> Table {
        let type_name = match self {
            ProjectTrackingStatus::TrackedProject { .. } => "TrackedProject",
            ProjectTrackingStatus::UntrackedProject => "UntrackedProject",
            ProjectTrackingStatus::IgnoredProject => "IgnoredProject",
        };

        let mut table = Table::new();
        table.insert(
            "type".to_string(),
            toml::Value::String(type_name.to_string()),
        );

        match self {
            ProjectTrackingStatus::TrackedProject {
                backup_requirement_class,
                last_update,
                ..
            } => {
                table.insert(
                    "backup_requirement_class".to_string(),
                    toml::Value::Table(backup_requirement_class.to_toml_table()),
                );
                table.insert(
                    "last_update".to_string(),
                    toml::Value::String(
                        last_update
                            .map(|instant| {
                                instant
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs()
                                    .to_string()
                            })
                            .unwrap_or_else(|| "".to_string()),
                    ),
                );

                // TODO: Implement ProjectCopy
            }
            _ => {}
        }

        table
    }
}

impl ToToml for ProjectTrackingStatus {
    fn to_toml(&self) -> Result<String, String> {
        let toml_value = self.to_toml_table();
        let toml = toml_value.to_string();
        Ok(toml)
    }
}

impl ToTomlTable for Project {
    fn to_toml_table(&self) -> Table {
        let mut table = Table::new();
        table.insert(
            "name".to_string(),
            toml::Value::String(self.get_name().clone()),
        );
        table.insert(
            "location".to_string(),
            toml::Value::String(self.get_location().clone()),
        );
        table.insert(
            "tracking_status".to_string(),
            toml::Value::Table(self.get_tracking_status().to_toml_table()),
        );

        table
    }
}

impl ToToml for Project {
    fn to_toml(&self) -> Result<String, String> {
        let table = self.to_toml_table();
        let toml = toml::to_string(&table).map_err(|e| e.to_string())?;
        Ok(toml)
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use mockall::predicate::eq;

    use crate::adapters::primary_device::MockGlobalConfigProvider;
    use crate::core::test_utils::mocks::{MockDevice, MockDeviceFactory, MockDeviceWithParameters};
    use crate::models::secondary_device::DeviceFactory;

    use super::super::super::global_config::GlobalConfig;
    use super::*;

    #[test]
    fn when_converting_empty_config_to_toml_it_shall_return_empty_string() {
        let global_config = GlobalConfig::new(vec![], vec![]);
        let toml = global_config.to_toml().unwrap();
        assert_eq!(toml, r#""#.trim());
    }

    #[test]
    fn when_converting_config_with_devices_to_toml_it_shall_return_toml() {
        let device = MockDeviceFactory.build().unwrap();
        let global_config = GlobalConfig::new(vec![device], vec![]);
        let toml = global_config.to_toml().unwrap();
        assert_eq!(
            toml,
            r#"[[devices]]
name = "MockDevice"
type = "MockDevice"
"#
        );
    }

    #[test]
    fn when_converting_config_with_multiple_devices_it_shall_save_config_with_devices() {
        let mut config_provider = MockGlobalConfigProvider::new();
        config_provider
            .expect_write_global_config()
            .times(1)
            .with(eq(r#"[[devices]]
name = "MockDevice"
type = "MockDevice"

[[devices]]
name = "MyDevice"
parameter = "MyParameter"
type = "MockDeviceWithParameters"
"#))
            .return_const(Ok(()));

        let device1 = MockDevice::new("MockDevice");
        let device2 = MockDeviceWithParameters::new("MyDevice", "MyParameter");
        let global_config = GlobalConfig::new(vec![Box::new(device1), Box::new(device2)], vec![]);
        global_config.save(&config_provider).unwrap();
    }

    #[test]
    fn when_converting_backup_requirement_class_to_toml_it_shall_return_toml() {
        let backup_requirement_class = BackupRequirementClass::default();
        let toml = backup_requirement_class.to_toml().unwrap();
        assert_eq!(
            toml,
            r#"min_security_level = "NetworkUntrustedRestricted"
name = "Default"
target_copies = 3
target_locations = 2
"#
        );
    }

    #[test]
    fn when_converting_tracked_project_to_toml_it_shall_return_toml() {
        let backup_requirement_class = BackupRequirementClass::default();
        let last_update = Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(100));
        let project_tracking_status = ProjectTrackingStatus::TrackedProject {
            backup_requirement_class,
            last_update,
            current_copies: vec![],
        };
        let toml = project_tracking_status.to_toml().unwrap();
        assert_eq!(
            toml,
            r#"last_update = "100"
type = "TrackedProject"

[backup_requirement_class]
min_security_level = "NetworkUntrustedRestricted"
name = "Default"
target_copies = 3
target_locations = 2
"#
        );
    }

    #[test]
    fn when_converting_untracked_project_to_toml_it_shall_return_toml() {
        let project_tracking_status = ProjectTrackingStatus::UntrackedProject;
        let toml = project_tracking_status.to_toml().unwrap();
        assert_eq!(
            toml,
            r#"type = "UntrackedProject"
"#
        );
    }

    #[test]
    fn when_converting_ignored_project_to_toml_it_shall_return_toml() {
        let project_tracking_status = ProjectTrackingStatus::IgnoredProject;
        let toml = project_tracking_status.to_toml().unwrap();
        assert_eq!(
            toml,
            r#"type = "IgnoredProject"
"#
        );
    }

    #[test]
    fn when_converting_project_to_toml_it_shall_return_toml() {
        let project = Project::new(
            "MyProject".to_string(),
            "MyLocation".to_string(),
            Some(ProjectTrackingStatus::TrackedProject {
                last_update: Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(100)),
                backup_requirement_class: BackupRequirementClass::default(),
                current_copies: vec![],
            }),
        );

        let toml = project.to_toml().unwrap();
        assert_eq!(
            toml,
            r#"location = "MyLocation"
name = "MyProject"

[tracking_status]
last_update = "100"
type = "TrackedProject"

[tracking_status.backup_requirement_class]
min_security_level = "NetworkUntrustedRestricted"
name = "Default"
target_copies = 3
target_locations = 2
"#
        );
    }

    #[test]
    fn when_converting_config_with_one_project_to_toml_it_shall_return_toml() {
        let project = Project::new(
            "MyProject".to_string(),
            "MyLocation".to_string(),
            Some(ProjectTrackingStatus::TrackedProject {
                last_update: Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(100)),
                backup_requirement_class: BackupRequirementClass::default(),
                current_copies: vec![],
            }),
        );

        let global_config = GlobalConfig::new(vec![], vec![project]);
        let toml = global_config.to_toml().unwrap();
        assert_eq!(
            toml,
            r#"[[projects]]
location = "MyLocation"
name = "MyProject"

[projects.tracking_status]
last_update = "100"
type = "TrackedProject"

[projects.tracking_status.backup_requirement_class]
min_security_level = "NetworkUntrustedRestricted"
name = "Default"
target_copies = 3
target_locations = 2
"#
        );
    }

    #[test]
    fn when_converting_config_with_multiple_projects_to_toml_it_shall_return_toml() {
        let project1 = Project::new(
            "MyProject".to_string(),
            "MyLocation".to_string(),
            Some(ProjectTrackingStatus::TrackedProject {
                last_update: Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(100)),
                backup_requirement_class: BackupRequirementClass::default(),
                current_copies: vec![],
            }),
        );

        let project2 = Project::new(
            "MyProject2".to_string(),
            "MyLocation2".to_string(),
            Some(ProjectTrackingStatus::UntrackedProject),
        );

        let global_config = GlobalConfig::new(vec![], vec![project1, project2]);
        let toml = global_config.to_toml().unwrap();
        assert_eq!(
            toml,
            r#"[[projects]]
location = "MyLocation"
name = "MyProject"

[projects.tracking_status]
last_update = "100"
type = "TrackedProject"

[projects.tracking_status.backup_requirement_class]
min_security_level = "NetworkUntrustedRestricted"
name = "Default"
target_copies = 3
target_locations = 2

[[projects]]
location = "MyLocation2"
name = "MyProject2"

[projects.tracking_status]
type = "UntrackedProject"
"#
        );
    }

    #[test]
    fn when_converting_config_with_2_devices_and_2_projects_to_toml_it_shall_return_toml() {
        let device1 = MockDevice::new("MockDevice");
        let device2 = MockDeviceWithParameters::new("MyDevice", "MyParameter");
        let project1 = Project::new(
            "MyProject".to_string(),
            "MyLocation".to_string(),
            Some(ProjectTrackingStatus::TrackedProject {
                last_update: Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(100)),
                backup_requirement_class: BackupRequirementClass::default(),
                current_copies: vec![],
            }),
        );

        let project2 = Project::new(
            "MyProject2".to_string(),
            "MyLocation2".to_string(),
            Some(ProjectTrackingStatus::UntrackedProject),
        );

        let global_config = GlobalConfig::new(
            vec![Box::new(device1), Box::new(device2)],
            vec![project1, project2],
        );
        let toml = global_config.to_toml().unwrap();
        assert_eq!(
            toml,
            r#"[[devices]]
name = "MockDevice"
type = "MockDevice"

[[devices]]
name = "MyDevice"
parameter = "MyParameter"
type = "MockDeviceWithParameters"

[[projects]]
location = "MyLocation"
name = "MyProject"

[projects.tracking_status]
last_update = "100"
type = "TrackedProject"

[projects.tracking_status.backup_requirement_class]
min_security_level = "NetworkUntrustedRestricted"
name = "Default"
target_copies = 3
target_locations = 2

[[projects]]
location = "MyLocation2"
name = "MyProject2"

[projects.tracking_status]
type = "UntrackedProject"
"#
        );
    }
}
