use itertools::Itertools;

use crate::{
    core::device_factories_registry::DeviceFactoryRegistry,
    models::{project::Project, secondary_device::Device},
};

use super::super::{
    from_toml::{parse_toml_global_config, ParseTomlResult},
    to_toml::ToToml,
    GlobalConfig, GlobalConfigProvider,
};

impl GlobalConfig {
    pub fn load(
        config_provider: &dyn GlobalConfigProvider,
        device_factories_registry: &DeviceFactoryRegistry,
    ) -> Result<GlobalConfig, String> {
        let config_toml = config_provider.read_global_config()?;

        let ParseTomlResult {
            devices,
            projects,
            device_errors,
            project_errors,
        } = parse_toml_global_config(&config_toml, device_factories_registry)?;

        Self::assert_no_errors_in_config(
            &device_errors,
            "Errors while reading devices from config",
        )?;
        Self::assert_no_errors_in_config(
            &project_errors,
            "Errors while reading projects from config",
        )?;
        Self::assert_no_duplicate_device(&devices)?;
        Self::assert_no_duplicate_project_name(&projects)?;
        Self::assert_no_duplicate_project_path(&projects)?;

        Ok(GlobalConfig { devices, projects })
    }

    pub fn save(&self, config_provider: &dyn GlobalConfigProvider) -> Result<(), String> {
        let config_toml = self.to_toml()?;

        config_provider.write_global_config(&config_toml).unwrap();

        Ok(())
    }

    fn assert_no_errors_in_config(
        errors: &Vec<String>,
        prefix_if_errors: &str,
    ) -> Result<(), String> {
        if !errors.is_empty() {
            return Err(format!("{}: {}", prefix_if_errors, errors.iter().join(", ")).to_string());
        }

        Ok(())
    }

    fn assert_no_duplicate_device(devices: &Vec<Box<dyn Device>>) -> Result<(), String> {
        let device_names = devices
            .iter()
            .map(|device| device.get_name())
            .collect::<Vec<String>>();

        Self::assert_no_duplicate(
            device_names.iter(),
            "Duplicate device found in configuration file",
        )
    }

    fn assert_no_duplicate_project_name(projects: &Vec<Project>) -> Result<(), String> {
        Self::assert_no_duplicate(
            projects.iter().map(|project| project.get_name()),
            "Duplicated name found in configuration file for project",
        )
    }

    fn assert_no_duplicate_project_path(projects: &Vec<Project>) -> Result<(), String> {
        Self::assert_no_duplicate(
            projects.iter().map(|project| project.get_location()),
            "Duplicated path found in configuration file for project at",
        )
    }

    fn assert_no_duplicate<'a>(
        keys: impl Iterator<Item = &'a String>,
        error_introduction_if_duplicate: &str,
    ) -> Result<(), String> {
        let duplicate_keys = keys
            .fold(std::collections::HashMap::new(), |mut acc, key| {
                *acc.entry(key).or_insert(0) += 1;
                acc
            })
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .sorted_by(|(key1, _), (key2, _)| key1.cmp(key2))
            .collect::<Vec<_>>();

        if !duplicate_keys.is_empty() {
            return Err(format!(
                "{}: {}",
                error_introduction_if_duplicate,
                duplicate_keys.iter().map(|(key, _)| key).join(", ")
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use crate::{
        core::test_utils::mocks::{
            MockDevice, MockDeviceFactory, MockDeviceWithParameters,
            MockDeviceWithParametersFactory, MockGlobalConfigProviderFactory,
        },
        models::{backup_requirement::SecurityLevel, project::ProjectTrackingStatus},
    };

    use super::super::super::MockGlobalConfigProvider;
    use super::*;

    fn get_mock_device_factory_registry() -> DeviceFactoryRegistry {
        let mut registry = DeviceFactoryRegistry::new();
        registry.register_device("MockDevice".to_string(), "MockDevice".to_string(), || {
            Box::new(MockDeviceFactory)
        });
        registry.register_device(
            "MockDeviceWithParameters".to_string(),
            "MockDeviceWithParameters".to_string(),
            || Box::new(MockDeviceWithParametersFactory),
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
    tracking_status = { type = "IgnoredProject" }
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
    tracking_status = { type = "IgnoredProject" }

        [[projects]]
    name = "MySecondAwesomeProject"
    path = "/root"
    tracking_status = { type = "IgnoredProject" }
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
    tracking_status = { type = "IgnoredProject" }
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
    tracking_status = { type = "IgnoredProject" }
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
    tracking_status = { type = "IgnoredProject" }
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
    tracking_status = { type = "IgnoredProject" }

    [[projects]]
    name = "MySecondProject"
    path = "/root"
    tracking_status = { type = "IgnoredProject" }
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
        assert_eq!(
            config.err().unwrap(),
            "Errors while reading devices from config: Device factory not found"
        );
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
        assert_eq!(
            config.err().unwrap(),
            "Errors while reading devices from config: Missing 'name' field"
        );
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
        assert_eq!(
            config.err().unwrap(),
            "Errors while reading projects from config: Missing 'name' field"
        );
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
        assert_eq!(
            config.err().unwrap(),
            "Errors while reading projects from config: Missing 'path' field"
        );
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
            "Errors while reading devices from config: Missing 'name' field, Device factory not found"
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
            "Errors while reading projects from config: Missing 'path' field, Missing 'name' field"
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
        assert_eq!(
            config.err().unwrap(),
            "Errors while reading devices from config: Missing parameter"
        );
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
    tracking_status = { type = "IgnoredProject" }

    [[projects]]
    name = "MyProject"
    path = "/secondPath"
    tracking_status = { type = "IgnoredProject" }
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry);
        assert!(config.is_err());
        assert_eq!(
            config.err().unwrap(),
            "Duplicated name found in configuration file for project: MyProject"
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
    tracking_status = { type = "IgnoredProject" }

    [[projects]]
    name = "MyProjectInAnotherPath"
    path = "/path"
    tracking_status = { type = "IgnoredProject" }
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry);
        assert!(config.is_err());
        assert_eq!(
            config.err().unwrap(),
            "Duplicated path found in configuration file for project at: /path"
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
    fn when_loading_with_untracked_backup_class_config_it_should_reflect_in_the_project() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProjectInOnePath"
    path = "/path"
    tracking_status = { type = "UntrackedProject"}
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
    tracking_status = { type = "IgnoredProject" }
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
        assert_eq!(
            config.err().unwrap(),
            "Errors while reading projects from config: No tracking status saved"
        );
    }

    #[test]
    fn when_loading_with_unknown_tracking_class_it_should_throw_an_error() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProjectInOnePath"
    path = "/path"
    tracking_status = { type = "UnknownTrackingStatus" }
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry);
        assert!(config.is_err());
        assert_eq!(
            config.err().unwrap(),
            "Errors while reading projects from config: Unknown tracking status type"
        );
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
    tracking_status = { type = "TrackedProject", backup_requirement_class = {target_copies = 3, target_locations = 2, min_security_level = "NetworkUntrustedRestricted", name = "Critical Data"}, last_update = "2024-04-18T15:22:00Z", current_copies = [] }

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
        assert_eq!(
            tracking_status
                .get_backup_requirement_class()
                .unwrap()
                .get_name(),
            "Critical Data"
        );
        assert_eq!(
            tracking_status
                .get_backup_requirement_class()
                .unwrap()
                .get_target_copies(),
            3
        );
        assert_eq!(
            tracking_status
                .get_backup_requirement_class()
                .unwrap()
                .get_target_locations(),
            2
        );
        assert!(matches!(
            tracking_status
                .get_backup_requirement_class()
                .unwrap()
                .get_min_security_level(),
            SecurityLevel::NetworkUntrustedRestricted
        ));
    }

    #[test]
    fn when_loading_with_tracking_status_if_no_backup_requirement_class_it_should_throw_an_error() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProviderFactory::new(
            r#"
    [[projects]]
    name = "MyProjectInOnePath"
    path = "/path"
    tracking_status = { type = "TrackedProject"}
    "#,
        );
        let config = GlobalConfig::load(&config_provider, &device_factories_registry);
        assert!(config.is_err());
        assert_eq!(
            config.err().unwrap(),
            "Errors while reading projects from config: Missing 'backup_requirement_class' section"
        );
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
}
