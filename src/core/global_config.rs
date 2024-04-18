use itertools::Itertools;
use toml::Table;

use crate::{
    adapters::primary_device::GlobalConfigProvider,
    models::{
        backup_requirement::{BackupRequirementClass, SecurityLevel},
        project::{Project, ProjectTrackingStatus},
        secondary_device::Device,
    },
};

use super::device_factories_registry::DeviceFactoryRegistry;

pub struct GlobalConfig {
    devices: Vec<Box<dyn Device>>,
    projects: Vec<Project>,
}

impl GlobalConfig {
    fn get_device_by_name(&self, name: &str) -> Option<&Box<dyn Device>> {
        self.devices.iter().find(|d| d.get_name() == name)
    }

    pub fn add_device(&mut self, device: Box<dyn Device>) -> Result<(), String> {
        if self.get_device_by_name(&device.get_name()).is_some() {
            return Err(format!(
                "Device with name {} already exists",
                device.get_name()
            ));
        }

        self.devices.push(device);
        Ok(())
    }

    pub fn remove_device(&mut self, name: &str) -> Result<(), String> {
        let index = self
            .devices
            .iter()
            .position(|d| d.get_name() == name)
            .ok_or_else(|| "Device not found".to_string())?;

        self.devices.remove(index);
        Ok(())
    }

    pub fn get_devices(self) -> Vec<Box<dyn Device>> {
        self.devices
    }
}

impl GlobalConfig {
    fn get_project_by_name(&self, name: &str) -> Option<&Project> {
        unimplemented!()
    }

    pub fn add_project(&mut self, project: Project) -> Result<(), String> {
        unimplemented!()
    }

    pub fn remove_project(&mut self, name: &str) -> Result<(), String> {
        unimplemented!()
    }

    pub fn get_projects(self) -> Vec<Project> {
        unimplemented!()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
struct PartiallyParsedGlobalConfig {
    devices: Option<Vec<Table>>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
struct PartiallyParsedProjectGlobalConfig {
    projects: Option<Vec<Table>>,
}

impl GlobalConfig {
    pub fn load(
        config_provider: &dyn GlobalConfigProvider,
        device_factories_registry: &DeviceFactoryRegistry,
    ) -> Result<GlobalConfig, String> {
        let config_toml = config_provider.read_global_config()?;

        let (errors, devices) = toml::from_str::<PartiallyParsedGlobalConfig>(&config_toml)
            .map_err(|e| e.to_string())?
            .devices
            .unwrap_or(vec![])
            .into_iter()
            .map(|device_table| -> Result<Box<dyn Device>, String> {
                Self::load_device_from_toml_bloc(device_table, &device_factories_registry)
            })
            .into_iter()
            .partition_map(From::from);

        let (project_errors, projects): (Vec<String>, Vec<Project>) =
            toml::from_str::<PartiallyParsedProjectGlobalConfig>(&config_toml)
                .map_err(|e| e.to_string())?
                .projects
                .unwrap_or(vec![])
                .into_iter()
                .map(|project_table| -> Result<Project, String> {
                    Self::load_project_from_toml_bloc(project_table)
                })
                .into_iter()
                .partition_map(From::from);

        Self::assert_no_errors_in_devices(&errors)?;
        Self::assert_no_duplicate_device(&devices)?;

        Self::assert_no_errors_in_projects(&project_errors)?;
        Self::assert_no_duplicate_project_name(&projects)?;
        Self::assert_no_duplicate_project_path(&projects)?;

        Ok(GlobalConfig { devices, projects })
    }

    pub fn save(&self, config_provider: &dyn GlobalConfigProvider) -> Result<(), String> {
        let device_tables = self
            .devices
            .iter()
            .map(|device| device.to_toml_table())
            .collect::<Vec<_>>();

        let config_toml = toml::to_string(&PartiallyParsedGlobalConfig {
            devices: if device_tables.is_empty() {
                None
            } else {
                Some(device_tables)
            },
        })
        .map_err(|e| e.to_string())?;

        config_provider.write_global_config(&config_toml).unwrap();

        Ok(())
    }

    fn load_device_from_toml_bloc(
        device_table: toml::map::Map<String, toml::Value>,
        device_factories_registry: &DeviceFactoryRegistry,
    ) -> Result<Box<dyn Device>, String> {
        let name = device_table
            .get("name")
            .ok_or_else(|| "Missing name for device".to_string())?
            .as_str()
            .ok_or_else(|| "Invalid string for name".to_string())?;

        let device_type: &str = device_table
            .get("type")
            .ok_or_else(|| "Type not found".to_string())?
            .as_str()
            .ok_or_else(|| "Invalid string for type".to_string())?;

        let factory = device_factories_registry
            .get_device_factory(device_type)
            .ok_or_else(|| "Device factory not found".to_string())?;

        let device = factory.build_from_toml_table(&name, &device_table)?;
        Ok(device)
    }

    fn load_project_from_toml_bloc(
        project_table: toml::map::Map<String, toml::Value>,
    ) -> Result<Project, String> {
        let name = project_table
            .get("name")
            .ok_or_else(|| "Missing name for project".to_string())?
            .as_str()
            .ok_or_else(|| "Invalid string for name".to_string())?;

        let path = project_table
            .get("path")
            .ok_or_else(|| "Missing path for project".to_string())?
            .as_str()
            .ok_or_else(|| "Invalid string for path".to_string())?;

        let tracking_status = project_table
            .get("tracking_status")
            .ok_or_else(|| "No tracking status saved".to_string())?
            .as_str()
            .ok_or_else(|| "Invalid string for tracking_status".to_string())?;

        let tracking_status = match tracking_status {
            "TrackedProject" => Ok(ProjectTrackingStatus::TrackedProject {
                backup_requirement_class: BackupRequirementClass::default(),
                last_update: None,
                current_copies: vec![],
            }),
            "UntrackedProject" => Ok(ProjectTrackingStatus::UntrackedProject),
            "IgnoredProject" => Ok(ProjectTrackingStatus::IgnoredProject),
            _ => Err("Unknown tracking status".to_string()),
        }?;

        Ok(Project::new(
            name.to_string(),
            path.to_string(),
            Some(tracking_status),
        ))
    }

    fn assert_no_errors_in_devices(errors: &Vec<String>) -> Result<(), String> {
        if !errors.is_empty() {
            return Err(errors.join(", "));
        }

        Ok(())
    }

    fn assert_no_duplicate_device(devices: &Vec<Box<dyn Device>>) -> Result<(), String> {
        let device_count_by_name = devices
            .iter()
            .map(|device| device.get_name())
            .fold(std::collections::HashMap::new(), |mut acc, name| {
                *acc.entry(name).or_insert(0) += 1;
                acc
            })
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .sorted_by(|(name1, _), (name2, _)| name1.cmp(name2))
            .collect::<Vec<_>>();

        if !device_count_by_name.is_empty() {
            return Err(format!(
                "Duplicate device found in configuration file: {}",
                device_count_by_name.iter().map(|(name, _)| name).join(", ")
            ));
        }

        Ok(())
    }

    fn assert_no_errors_in_projects(errors: &Vec<String>) -> Result<(), String> {
        if !errors.is_empty() {
            return Err(errors.join(", "));
        }
        Ok(())
    }

    fn assert_no_duplicate_project_name(projects: &Vec<Project>) -> Result<(), String> {
        let project_count_by_name = projects
            .iter()
            .map(|project| project.get_name())
            .fold(std::collections::HashMap::new(), |mut acc, name| {
                *acc.entry(name).or_insert(0) += 1;
                acc
            })
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .sorted_by(|(name1, _), (name2, _)| name1.cmp(name2))
            .collect::<Vec<_>>();

        if !project_count_by_name.is_empty() {
            return Err(format!(
                "Duplicated name found in configuration file for project : {}",
                project_count_by_name
                    .iter()
                    .map(|(name, _)| name)
                    .join(", ")
            ));
        }

        Ok(())
    }

    fn assert_no_duplicate_project_path(projects: &Vec<Project>) -> Result<(), String> {
        let project_count_by_path = projects
            .iter()
            .map(|project| project.get_location())
            .fold(std::collections::HashMap::new(), |mut acc, path| {
                *acc.entry(path).or_insert(0) += 1;
                acc
            })
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .sorted_by(|(path1, _), (path2, _)| path1.cmp(path2))
            .collect::<Vec<_>>();

        if !project_count_by_path.is_empty() {
            return Err(format!(
                "Duplicated path found in configuration file for project at : {}",
                project_count_by_path
                    .iter()
                    .map(|(path, _)| path)
                    .join(", ")
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use crate::{
        adapters::primary_device::MockGlobalConfigProvider,
        core::{
            device_factories_registry::DeviceFactoryRegistry,
            test_utils::mocks::{
                MockDevice, MockDeviceFactory, MockDeviceWithParameters,
                MockDeviceWithParametersFactory, MockGlobalConfigProviderFactory,
            },
        },
        models::backup_requirement::{BackupRequirementClass, SecurityLevel},
        models::project::ProjectTrackingStatus,
        models::secondary_device::DeviceFactory,
    };

    use super::GlobalConfig;

    fn get_mock_device_factory_registry() -> DeviceFactoryRegistry {
        let mut registry = DeviceFactoryRegistry::new();
        registry.register_device(
            "MockDevice".to_string(),
            "MockDevice".to_string(),
            Box::new(MockDeviceFactory),
        );
        registry.register_device(
            "MockDeviceWithParameters".to_string(),
            "MockDeviceWithParameters".to_string(),
            Box::new(MockDeviceWithParametersFactory),
        );
        registry
    }

    #[test]
    fn when_failing_to_retrieve_config_it_shall_return_the_error() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new_failing_to_read();
        let config = GlobalConfig::load(&config_provider, &device_factories_registry);
        assert!(config.is_err());
    }

    #[test]
    fn when_retrieving_config_it_shall_return_the_config() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new("");
        let config = GlobalConfig::load(&config_provider, &device_factories_registry);
        assert!(config.is_ok());
    }

    #[test]
    fn when_retrieving_config_with_no_device_it_shall_have_no_device_in_global_config() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new("");
        let config = GlobalConfig::load(&config_provider, &device_factories_registry).unwrap();
        assert_eq!(config.devices.len(), 0);
    }

    #[test]
    fn when_retrieving_config_with_no_project_it_shall_have_no_project_in_global_config() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new("");
        let config = GlobalConfig::load(&config_provider, &device_factories_registry).unwrap();
        assert_eq!(config.projects.len(), 0);
    }

    #[test]
    fn when_retrieving_config_with_one_mock_device_it_shall_have_one_device_in_global_config() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[devices]]
    name = "MockDevice"
    type = "MockDevice"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry).unwrap();
        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].get_device_type_name(), "MockDevice");
        assert_eq!(config.devices[0].get_name(), "MockDevice");
    }

    #[test]
    fn when_retrieving_config_with_one_project_it_shall_have_one_project_in_global_config() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProject"
    path = "/tmp"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry).unwrap();
        assert_eq!(config.projects.len(), 1);
    }

    #[test]
    fn when_retrieving_config_with_multiple_project_it_should_have_the_project_in_global_config() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProject"
    path = "/tmp"

        [[projects]]
    name = "MySecondAwesomeProject"
    path = "/root"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry).unwrap();
        assert_eq!(config.projects.len(), 2);
    }

    #[test]
    fn when_retrieving_config_with_different_name_it_shall_be_reflected_in_device() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[devices]]
    name = "MyPersonalDevice"
    type = "MockDevice"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry).unwrap();
        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].get_device_type_name(), "MockDevice");
        assert_eq!(config.devices[0].get_name(), "MyPersonalDevice");
    }

    #[test]
    fn when_retrieving_config_with_different_name_it_shall_be_reflected_in_project() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProject"
    path = "/tmp"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry).unwrap();
        assert_eq!(config.projects.len(), 1);
        assert_eq!(config.projects[0].get_name(), "MyProject");
        assert_eq!(config.projects[0].get_location(), "/tmp");
    }

    #[test]
    fn when_retrieving_config_with_random_name_it_shall_be_reflected_in_project() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "ergergerger"
    path = "/tmp"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry).unwrap();
        assert_eq!(config.projects.len(), 1);
        assert_eq!(config.projects[0].get_name(), "ergergerger");
        assert_eq!(config.projects[0].get_location(), "/tmp");
    }

    #[test]
    fn when_retrieving_config_with_random_path_it_shall_be_reflected_in_project() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProject"
    path = "/gerger/gerg/zfer/zgze"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry).unwrap();
        assert_eq!(config.projects.len(), 1);
        assert_eq!(config.projects[0].get_name(), "MyProject");
        assert_eq!(config.projects[0].get_location(), "/gerger/gerg/zfer/zgze");
    }

    #[test]
    fn when_retrieving_config_with_multiple_devices_it_shall_have_all_devices_in_global_config() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[devices]]
    name = "MyPersonalDevice"
    type = "MockDevice"

    [[devices]]
    name = "MySecondPersonalDevice"
    type = "MockDevice"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry).unwrap();
        assert_eq!(config.devices.len(), 2);
        assert_eq!(config.devices[0].get_device_type_name(), "MockDevice");
        assert_eq!(config.devices[0].get_name(), "MyPersonalDevice");
        assert_eq!(config.devices[1].get_device_type_name(), "MockDevice");
        assert_eq!(config.devices[1].get_name(), "MySecondPersonalDevice");
    }

    #[test]
    fn when_retrieving_config_with_multiple_projects_it_shall_have_all_projects_in_global_config() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProject"
    path = "/tmp"

    [[projects]]
    name = "MySecondProject"
    path = "/root"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry).unwrap();
        assert_eq!(config.projects.len(), 2);
        assert_eq!(config.projects[0].get_name(), "MyProject");
        assert_eq!(config.projects[0].get_location(), "/tmp");
        assert_eq!(config.projects[1].get_name(), "MySecondProject");
        assert_eq!(config.projects[1].get_location(), "/root");
    }

    #[test]
    fn if_type_is_not_in_registry_it_shall_return_error() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[devices]]
    name = "MyPersonalDevice"
    type = "UnknownDevice"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry);
        assert!(config.is_err());
        assert_eq!(config.err().unwrap(), "Device factory not found");
    }

    #[test]
    fn if_name_is_missing_it_shall_fail() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[devices]]
    type = "MockDevice"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry);
        assert!(config.is_err());
        assert_eq!(config.err().unwrap(), "Missing name for device");
    }

    #[test]
    fn if_name_is_missing_in_project_it_should_fail() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    path = "/tmp"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry);
        assert!(config.is_err());
        assert_eq!(config.err().unwrap(), "Missing name for project");
    }

    #[test]
    fn if_path_is_missing_in_project_it_should_fail() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProject"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry);
        assert!(config.is_err());
        assert_eq!(config.err().unwrap(), "Missing path for project");
    }

    #[test]
    fn if_multiple_errors_in_device_it_shall_return_first_error_of_each_device() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[devices]]
    type = "MockDevice"

    [[devices]]
    name = "MyPersonalDevice"
    type = "UnknownDevice"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry);
        assert!(config.is_err());
        assert_eq!(
            config.err().unwrap(),
            "Missing name for device, Device factory not found"
        );
    }

    #[test]
    fn if_multiple_errors_in_projects_it_shall_return_first_error_of_each_project() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProject"

    [[projects]]
    path = "/tmp"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry);
        assert!(config.is_err());
        assert_eq!(
            config.err().unwrap(),
            "Missing path for project, Missing name for project"
        );
    }

    #[test]
    fn when_there_are_multiple_device_types_each_device_type_shall_be_parsed() {
        let registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[devices]]
    name = "MyPersonalDevice"
    type = "MockDevice"

    [[devices]]
    name = "MySecondPersonalDevice"
    type = "MockDeviceWithParameters"
    parameter = "MyParameter"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &registry).unwrap();
        assert_eq!(config.devices.len(), 2);
        assert_eq!(config.devices[0].get_device_type_name(), "MockDevice");
        assert_eq!(config.devices[0].get_name(), "MyPersonalDevice");
        assert_eq!(
            config.devices[1].get_device_type_name(),
            "MockDeviceWithParameters"
        );
        assert_eq!(config.devices[1].get_name(), "MySecondPersonalDevice");
    }

    #[test]
    fn when_a_type_with_additional_parameters_has_a_missing_parameter_it_should_propagate_error() {
        let registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[devices]]
    name = "MySecondPersonalDevice"
    type = "MockDeviceWithParameters"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &registry);
        assert!(config.is_err());
        assert_eq!(config.err().unwrap(), "Missing parameter");
    }

    #[test]
    fn when_loading_configuration_if_there_is_2_devices_of_same_name_there_should_be_an_error() {
        let registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[devices]]
    name = "MyPersonalDevice"
    type = "MockDevice"

    [[devices]]
    name = "MyPersonalDevice"
    type = "MockDevice"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &registry);
        assert!(config.is_err());
        assert_eq!(
            config.err().unwrap(),
            "Duplicate device found in configuration file: MyPersonalDevice"
        );
    }

    #[test]
    fn when_loading_if_there_is_two_projects_with_same_name_there_should_be_an_error() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProject"
    path = "/firstPath"

    [[projects]]
    name = "MyProject"
    path = "/secondPath"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry);
        assert!(config.is_err());
        assert_eq!(
            config.err().unwrap(),
            "Duplicated name found in configuration file for project : MyProject"
        );
    }

    #[test]
    fn when_loading_if_there_is_two_projects_with_same_path_there_should_be_an_error() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProjectInOnePath"
    path = "/path"

    [[projects]]
    name = "MyProjectInAnotherPath"
    path = "/path"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry);
        assert!(config.is_err());
        assert_eq!(
            config.err().unwrap(),
            "Duplicated path found in configuration file for project at : /path"
        );
    }

    #[test]
    fn when_loading_configuration_if_there_is_4_devices_of_same_name_there_should_be_an_error() {
        let registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[devices]]
    name = "MyPersonalDevice"
    type = "MockDevice"

    [[devices]]
    name = "MyPersonalDevice"
    type = "MockDevice"

    [[devices]]
    name = "MyOtherDevice"
    type = "MockDevice"

    [[devices]]
    name = "MyOtherDevice"
    type = "MockDevice"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &registry);
        assert!(config.is_err());
        assert_eq!(
            config.err().unwrap(),
            "Duplicate device found in configuration file: MyOtherDevice, MyPersonalDevice"
        );
    }

    #[test]
    fn when_adding_device_to_global_config_it_shall_add_it() {
        let mut global_config = GlobalConfig {
            devices: vec![],
            projects: vec![],
        };

        let device = MockDeviceFactory
            .build_from_toml_table("MyPersonalDevice", &toml::Table::new())
            .unwrap();

        global_config.add_device(device).unwrap();
        assert_eq!(global_config.devices.len(), 1);
        assert_eq!(global_config.devices[0].get_name(), "MyPersonalDevice");
    }

    #[test]
    fn when_adding_device_to_global_config_if_device_already_exists_it_shall_return_error() {
        let mut global_config = GlobalConfig {
            devices: vec![],
            projects: vec![],
        };

        let device = MockDeviceFactory
            .build_from_toml_table("MyPersonalDevice", &toml::Table::new())
            .unwrap();

        let device2 = MockDeviceFactory
            .build_from_toml_table("MyPersonalDevice", &toml::Table::new())
            .unwrap();

        let result = global_config.add_device(device);
        assert!(result.is_ok());

        let result = global_config.add_device(device2);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            "Device with name MyPersonalDevice already exists"
        );

        assert_eq!(global_config.devices.len(), 1);
    }

    #[test]
    fn when_saving_config_it_shall_call_save_on_config_provider() {
        let mut config_provider = MockGlobalConfigProvider::new();
        config_provider
            .expect_write_global_config()
            .times(1)
            .with(eq(""))
            .return_const(Ok(()));

        let global_config = GlobalConfig {
            devices: vec![],
            projects: vec![],
        };
        global_config.save(&config_provider).unwrap();
    }

    #[test]
    fn when_saving_config_with_some_devices_it_shall_save_config_with_devices() {
        let mut config_provider = MockGlobalConfigProvider::new();
        config_provider
            .expect_write_global_config()
            .times(1)
            .with(eq(r#"[[devices]]
name = "MockDevice"
type = "MockDevice"
"#))
            .return_const(Ok(()));

        let device = MockDeviceFactory.build().unwrap();
        let global_config = GlobalConfig {
            devices: vec![device],
            projects: vec![],
        };

        global_config.save(&config_provider).unwrap();
    }

    #[test]
    fn when_saving_config_with_multiple_devices_it_shall_save_config_with_devices() {
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
        let global_config = GlobalConfig {
            devices: vec![Box::new(device1), Box::new(device2)],
            projects: vec![],
        };

        global_config.save(&config_provider).unwrap();
    }

    #[test]
    fn when_removing_device_from_global_config_it_shall_remove_it() {
        let mut global_config = GlobalConfig {
            devices: vec![],
            projects: vec![],
        };

        let device1 = MockDevice::new("MyPersonalDevice");
        let device2 = MockDevice::new("MySecondPersonalDevice");

        global_config.add_device(Box::new(device1)).unwrap();
        global_config.add_device(Box::new(device2)).unwrap();
        assert_eq!(global_config.devices.len(), 2);

        global_config.remove_device("MyPersonalDevice").unwrap();
        assert_eq!(global_config.devices.len(), 1);
        assert_eq!(
            global_config.devices[0].get_name(),
            "MySecondPersonalDevice"
        );
    }

    #[test]
    fn when_removing_non_existant_device_it_shall_return_error() {
        let mut global_config = GlobalConfig {
            devices: vec![],
            projects: vec![],
        };
        let result = global_config.remove_device("NonExistantDevice");
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Device not found");
    }

    #[test]
    fn when_loading_projects_they_should_have_a_tracking_status_by_default() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProjectInOnePath"
    path = "/path"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry).unwrap();
        assert!(matches!(
            config.projects[0].get_tracking_status(),
            ProjectTrackingStatus::TrackedProject { .. }
        ));
    }

    #[test]
    fn when_loading_project_they_should_have_correct_default_fields_for_tracking_status() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProjectInOnePath"
    path = "/path"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry).unwrap();
        let tracking_status = config.projects[0].get_tracking_status();
        let backup_requirement = tracking_status.get_backup_requirement_class().unwrap();
        assert_eq!(backup_requirement.get_name(), "Default");
        assert_eq!(backup_requirement.get_target_copies(), 3);
        assert_eq!(backup_requirement.get_target_locations(), 2);
        assert!(matches!(
            backup_requirement.get_min_security_level(),
            SecurityLevel::NetworkUntrustedRestricted
        ));
    }

    #[test]
    fn when_loading_with_untracked_backup_class_config_it_should_reflect_in_the_project() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProjectInOnePath"
    path = "/path"
    tracking_status = "UntrackedProject"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry).unwrap();
        assert!(matches!(
            config.projects[0].get_tracking_status(),
            ProjectTrackingStatus::UntrackedProject
        ));
    }

    #[test]
    fn when_loading_with_ignored_backup_class_config_it_should_reflect_in_the_project() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProjectInOnePath"
    path = "/path"
    tracking_status = "IgnoredProject"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry).unwrap();
        assert!(matches!(
            config.projects[0].get_tracking_status(),
            ProjectTrackingStatus::IgnoredProject
        ));
    }

    #[test]
    fn when_loading_with_unspeciefied_tracking_class_it_should_throw_an_error() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProjectInOnePath"
    path = "/path"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry);
        assert!(config.is_err());
        assert_eq!(config.err().unwrap(), "No tracking status saved");
    }

    #[test]
    fn when_loading_with_unknown_tracking_class_it_should_throw_an_error() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProjectInOnePath"
    path = "/path"
    tracking_status = "UnknownTrackingStatus"
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry);
        assert!(config.is_err());
        assert_eq!(config.err().unwrap(), "Unknown tracking status");
    }

    #[test]
    fn when_loading_with_tracked_backup_class_config_it_should_reflect_in_the_project_tracking_config(
    ) {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProjectInOnePath"
    path = "/path"

    [[projects.tracking_status]]
    last_update = ""
    current_copies = []

    [[projects.tracking_status.backup_requirement_class]]
    target_copies = "1" 
    target_locations = "1"
    min_security_level = "NetworkUntrustedRestricted"
    name = ""
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry).unwrap();
        assert!(matches!(
            config.projects[0].get_tracking_status(),
            ProjectTrackingStatus::TrackedProject { .. }
        ));
        let tracking_status = config.projects[0].get_tracking_status();
        assert_eq!(tracking_status.get_current_copies().unwrap().len(), 0);
        assert_eq!(tracking_status.get_last_update(), None);
    }
}
