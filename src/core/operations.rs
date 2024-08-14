use std::{path::PathBuf, time::SystemTime};

use crate::{
    adapters::{
        operations::{
            backup::BackupOperations,
            device::DeviceOperations,
            project::{AddProjectArgs, ProjectOperations},
        },
        primary_device::GlobalConfigProvider,
    },
    models::{
        backup_requirement::BackupRequirementClass,
        project::{Project, ProjectTrackingStatus},
        secondary_device::{Device, DeviceFactory, DeviceFactoryKey},
    },
    now,
};

use super::{
    backup::{backup_execution::BackupExecution, backup_index::BackupIndex},
    device_factories_registry::DeviceFactoryRegistry,
    global_config::GlobalConfig,
};

pub struct Operations {
    device_factory_registry: DeviceFactoryRegistry,
    global_config_provider: Box<dyn GlobalConfigProvider>,
}
impl Operations {
    pub fn new(global_config_provider: Box<dyn GlobalConfigProvider>) -> Self {
        Operations {
            device_factory_registry: DeviceFactoryRegistry::new(),
            global_config_provider,
        }
    }

    pub fn register_device_factory(
        &mut self,
        device_factory_key: String,
        device_factory_readable_name: String,
        device_factory: impl Fn() -> Box<dyn DeviceFactory> + 'static,
    ) {
        self.device_factory_registry.register_device(
            device_factory_key,
            device_factory_readable_name,
            device_factory,
        );
    }
}

impl DeviceOperations for Operations {
    fn get_available_device_factories(&self) -> Vec<DeviceFactoryKey> {
        self.device_factory_registry.list_factories()
    }

    fn get_device_factory(&self, device_type: String) -> Option<Box<dyn DeviceFactory>> {
        self.device_factory_registry
            .get_device_factory(&device_type)
    }

    fn add_device(&self, device: Box<dyn Device>) -> Result<(), Box<String>> {
        let mut config = GlobalConfig::load(
            self.global_config_provider.as_ref(),
            &self.device_factory_registry,
        )?;

        config.add_device(device)?;
        config.save(self.global_config_provider.as_ref())?;
        Ok(())
    }

    fn remove_by_name(&self, name: String) -> Result<(), Box<String>> {
        let mut config = GlobalConfig::load(
            self.global_config_provider.as_ref(),
            &self.device_factory_registry,
        )?;

        config.remove_device(&name)?;
        config.save(self.global_config_provider.as_ref())?;

        Ok(())
    }

    fn list(&self) -> Result<Vec<Box<dyn Device>>, String> {
        let config = GlobalConfig::load(
            self.global_config_provider.as_ref(),
            &self.device_factory_registry,
        )?;

        let devices = config.get_devices();
        Ok(devices)
    }
}

impl ProjectOperations for Operations {
    fn add_project(&self, args: AddProjectArgs) -> Result<(), String> {
        let mut config = GlobalConfig::load(
            self.global_config_provider.as_ref(),
            &self.device_factory_registry,
        )?;

        let project = Project::new(
            args.name,
            args.location,
            Some(ProjectTrackingStatus::TrackedProject {
                backup_requirement_class: BackupRequirementClass::default(),
                last_update: Some(now!()),
                current_copies: vec![],
            }),
        );

        config.add_project(project)?;
        config.save(self.global_config_provider.as_ref())?;

        Ok(())
    }

    fn remove_project_by_name(&self, name: String) -> Result<(), String> {
        let mut config = GlobalConfig::load(
            self.global_config_provider.as_ref(),
            &self.device_factory_registry,
        )?;

        config.remove_project(&name)?;
        config.save(self.global_config_provider.as_ref())?;

        Ok(())
    }

    fn list_projects(&self) -> Result<Vec<Project>, String> {
        let config = GlobalConfig::load(
            self.global_config_provider.as_ref(),
            &self.device_factory_registry,
        )?;

        let projects = config.get_projects();
        Ok(projects)
    }
}

impl BackupOperations for Operations {
    fn backup_project_to_device(
        &self,
        project_name: &str,
        device_name: &str,
    ) -> Result<(), String> {
        let config = GlobalConfig::load(
            self.global_config_provider.as_ref(),
            &self.device_factory_registry,
        )?;

        let project = config
            .get_project_by_name(project_name)
            .ok_or_else(|| format!("Project not found: {}", project_name))?;

        let device = config
            .get_device_by_name(device_name)
            .ok_or_else(|| format!("Device not found: {}", device_name))?;

        device.test_availability()?;
        project.test_availability()?;

        let index = device
            .read_backup_index(project.get_name())?
            .map_or(BackupIndex::new(), |reader| {
                BackupIndex::from_index_reader(reader).unwrap()
            });

        let project_root_path = PathBuf::from(project.get_location());
        let mut backup_execution = BackupExecution::new(index, project_root_path);
        let archive_writer = device.get_archive_writer();

        backup_execution
            .execute(archive_writer)
            .map_err(|e| format!("Backup failed: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use mockall::predicate::eq;

    use crate::{
        adapters::primary_device::MockGlobalConfigProvider,
        core::test_utils::mocks::{MockDevice, MockDeviceFactory, MockGlobalConfigProviderFactory},
        models::{backup_requirement::SecurityLevel, project::ProjectTrackingStatus},
    };

    use super::*;

    impl Operations {
        fn new_with_mocked_dependencies() -> Self {
            Operations {
                device_factory_registry: DeviceFactoryRegistry::new(),
                global_config_provider: Box::new(MockGlobalConfigProvider::new()),
            }
        }
    }

    #[test]
    fn with_an_empty_registry_no_factory_is_returned() {
        let operations = Operations::new_with_mocked_dependencies();
        let available_factories = operations.get_available_device_factories();
        assert!(available_factories.is_empty());
    }

    #[test]
    fn after_registering_a_factory_it_can_be_retrieved() {
        let mut operations = Operations::new_with_mocked_dependencies();
        operations.register_device_factory(
            "MockDevice".to_string(),
            "Mock Device".to_string(),
            || Box::new(MockDeviceFactory),
        );

        let available_factories = operations.get_available_device_factories();
        assert_eq!(available_factories.len(), 1);
        assert_eq!(available_factories[0].key, "MockDevice");
        assert_eq!(available_factories[0].readable_name, "Mock Device");
    }

    #[test]
    fn if_no_factory_is_registered_none_is_returned_when_retrieving_a_factory() {
        let operations = Operations::new_with_mocked_dependencies();
        let device_factory = operations.get_device_factory("MockDevice".to_string());
        assert!(device_factory.is_none());
    }

    #[test]
    fn after_registering_a_factory_it_can_be_used_to_create_a_device() {
        let mut operations = Operations::new_with_mocked_dependencies();
        operations.register_device_factory(
            "MockDevice".to_string(),
            "Mock Device".to_string(),
            || Box::new(MockDeviceFactory),
        );

        let device_factory = operations.get_device_factory("MockDevice".to_string());
        assert!(device_factory.is_some());

        let device_factory = device_factory.unwrap();
        let device = device_factory.build();
        assert_eq!(device.unwrap().get_name(), "MockDevice");
    }

    #[test]
    fn when_registering_a_factory_and_retrieving_a_not_added_one_it_shall_return_none() {
        let mut operations = Operations::new_with_mocked_dependencies();
        operations.register_device_factory(
            "MockDevice".to_string(),
            "Mock Device".to_string(),
            || Box::new(MockDeviceFactory),
        );

        let device_factory = operations.get_device_factory("NotAdded".to_string());
        assert!(device_factory.is_none());
    }

    #[test]
    fn when_listing_devices_with_no_config_file_no_devices_are_returned() {
        let operations = Operations {
            device_factory_registry: DeviceFactoryRegistry::new(),
            global_config_provider: Box::new(MockGlobalConfigProviderFactory::new(r#""#)),
        };

        let devices = operations.list().unwrap();
        assert!(devices.is_empty());
    }

    #[test]
    fn when_listing_devices_with_one_it_is_returned() {
        let mut registry = DeviceFactoryRegistry::new();
        registry.register_device("MockDevice".to_string(), "Mock Device".to_string(), || {
            Box::new(MockDeviceFactory)
        });

        let operations = Operations {
            device_factory_registry: registry,
            global_config_provider: Box::new(MockGlobalConfigProviderFactory::new(
                r#"
[[devices]]
name = "MockDevice"
type = "MockDevice"
"#,
            )),
        };

        let devices = operations.list().unwrap();
        assert!(devices.len() == 1);
        assert_eq!(devices[0].get_name(), "MockDevice");
        assert_eq!(devices[0].get_device_type_name(), "MockDevice");
    }

    #[test]
    fn when_adding_a_device_to_empty_config_it_shall_add_it_to_the_configuration() {
        let mut registry = DeviceFactoryRegistry::new();
        registry.register_device("MockDevice".to_string(), "Mock Device".to_string(), || {
            Box::new(MockDeviceFactory)
        });

        let mut provider = MockGlobalConfigProvider::new();
        provider
            .expect_read_global_config()
            .return_const(Ok(r#""#.to_string()));
        provider
            .expect_write_global_config()
            .times(1)
            .with(eq(r#"[[devices]]
name = "MockDevice"
type = "MockDevice"
"#
            .to_string()))
            .return_const(Ok(()));

        let operations = Operations {
            device_factory_registry: registry,
            global_config_provider: Box::new(provider),
        };

        let device = Box::new(MockDevice::new("MockDevice"));
        operations.add_device(device).unwrap();
    }

    #[test]
    fn when_adding_a_device_to_config_with_another_device_it_shall_add_it_to_the_configuration() {
        let mut registry = DeviceFactoryRegistry::new();
        registry.register_device("MockDevice".to_string(), "Mock Device".to_string(), || {
            Box::new(MockDeviceFactory)
        });

        let mut provider = MockGlobalConfigProvider::new();
        provider
            .expect_read_global_config()
            .return_const(Ok(r#"[[devices]]
name = "AnotherDevice"
type = "MockDevice"
"#
            .to_string()));
        provider
            .expect_write_global_config()
            .times(1)
            .with(eq(r#"[[devices]]
name = "AnotherDevice"
type = "MockDevice"

[[devices]]
name = "MockDevice"
type = "MockDevice"
"#
            .to_string()))
            .return_const(Ok(()));

        let operations = Operations {
            device_factory_registry: registry,
            global_config_provider: Box::new(provider),
        };

        let device = Box::new(MockDevice::new("MockDevice"));
        operations.add_device(device).unwrap();
    }

    #[test]
    fn when_removing_last_device_by_name_it_shall_update_the_configuration() {
        let mut registry = DeviceFactoryRegistry::new();
        registry.register_device("MockDevice".to_string(), "Mock Device".to_string(), || {
            Box::new(MockDeviceFactory)
        });

        let mut provider = MockGlobalConfigProvider::new();
        provider
            .expect_read_global_config()
            .return_const(Ok(r#"[[devices]]
name = "AnotherDevice"
type = "MockDevice"
"#
            .to_string()));

        provider
            .expect_write_global_config()
            .times(1)
            .with(eq(r#""#.to_string()))
            .return_const(Ok(()));

        let operations = Operations {
            device_factory_registry: registry,
            global_config_provider: Box::new(provider),
        };

        operations
            .remove_by_name("AnotherDevice".to_string())
            .unwrap();
    }

    #[test]
    fn when_retrieving_projects_from_no_config_it_shall_return_empty_list() {
        let operations = Operations {
            device_factory_registry: DeviceFactoryRegistry::new(),
            global_config_provider: Box::new(MockGlobalConfigProviderFactory::new(r#""#)),
        };

        let projects = operations.list_projects().unwrap();
        assert!(projects.is_empty());
    }

    #[test]
    fn when_retrieving_projects_from_config_with_one_ignored_project_it_shall_return_it() {
        let mut registry = DeviceFactoryRegistry::new();
        registry.register_device("MockDevice".to_string(), "Mock Device".to_string(), || {
            Box::new(MockDeviceFactory)
        });

        let operations = Operations {
            device_factory_registry: registry,
            global_config_provider: Box::new(MockGlobalConfigProviderFactory::new(
                r#"[[projects]]
path = "/path/to/project"
name = "MyProject"

[projects.tracking_status]
last_update = "100"
type = "IgnoredProject"
"#,
            )),
        };

        let projects = operations.list_projects().unwrap();
        assert!(projects.len() == 1);
        assert_eq!(projects[0].get_name(), "MyProject");
        assert_eq!(projects[0].get_location(), "/path/to/project");

        let tracking_status = projects[0].get_tracking_status();
        assert!(matches!(
            tracking_status,
            ProjectTrackingStatus::IgnoredProject
        ));
    }

    #[test]
    fn when_retrieving_projects_from_config_with_one_project_it_shall_return_it() {
        let mut registry = DeviceFactoryRegistry::new();
        registry.register_device("MockDevice".to_string(), "Mock Device".to_string(), || {
            Box::new(MockDeviceFactory)
        });

        let operations = Operations {
            device_factory_registry: registry,
            global_config_provider: Box::new(MockGlobalConfigProviderFactory::new(
                r#"[[projects]]
path = "/path/to/project"
name = "MyProject"

[projects.tracking_status]
last_update = "100"
type = "TrackedProject"

[projects.tracking_status.backup_requirement_class]
min_security_level = "NetworkUntrustedRestricted"
name = "Default"
target_copies = 3
target_locations = 2
"#,
            )),
        };

        let projects = operations.list_projects().unwrap();
        assert!(projects.len() == 1);
        assert_eq!(projects[0].get_name(), "MyProject");
        assert_eq!(projects[0].get_location(), "/path/to/project");

        let tracking_status = projects[0].get_tracking_status();
        assert!(matches!(
            tracking_status,
            ProjectTrackingStatus::TrackedProject { .. }
        ));

        let tracking_status = tracking_status.get_backup_requirement_class().unwrap();
        assert!(matches!(
            tracking_status.get_min_security_level(),
            SecurityLevel::NetworkUntrustedRestricted
        ));
    }

    #[test]
    fn when_retrieving_several_projects_from_config_it_shall_return_them() {
        let mut registry = DeviceFactoryRegistry::new();
        registry.register_device("MockDevice".to_string(), "Mock Device".to_string(), || {
            Box::new(MockDeviceFactory)
        });

        let operations = Operations {
            device_factory_registry: registry,
            global_config_provider: Box::new(MockGlobalConfigProviderFactory::new(
                r#"[[projects]]
path = "/path/to/project"
name = "MyProject"

[projects.tracking_status]
last_update = "100"
type = "UntrackedProject"

[[projects]]
path = "/path/to/project2"
name = "MyProject2"

[projects.tracking_status]
last_update = "100"
type = "UntrackedProject"
"#,
            )),
        };

        let projects = operations.list_projects().unwrap();
        assert!(projects.len() == 2);
        assert_eq!(projects[0].get_name(), "MyProject");
        assert_eq!(projects[0].get_location(), "/path/to/project");
        assert_eq!(projects[1].get_name(), "MyProject2");
        assert_eq!(projects[1].get_location(), "/path/to/project2");
    }

    #[test]
    fn when_adding_a_project_to_empty_config_it_shall_add_it_to_the_configuration() {
        let mut registry = DeviceFactoryRegistry::new();
        registry.register_device("MockDevice".to_string(), "Mock Device".to_string(), || {
            Box::new(MockDeviceFactory)
        });

        let mut provider = MockGlobalConfigProvider::new();
        provider
            .expect_read_global_config()
            .return_const(Ok(r#""#.to_string()));
        provider
            .expect_write_global_config()
            .times(1)
            .with(eq(r#"[[projects]]
name = "MyProject"
path = "/path/to/project"

[projects.tracking_status]
last_update = "0"
type = "TrackedProject"

[projects.tracking_status.backup_requirement_class]
min_security_level = "NetworkUntrustedRestricted"
name = "Default"
target_copies = 3
target_locations = 2
"#
            .to_string()))
            .return_const(Ok(()));

        let operations = Operations {
            device_factory_registry: registry,
            global_config_provider: Box::new(provider),
        };

        let project = AddProjectArgs {
            name: "MyProject".to_string(),
            location: "/path/to/project".to_string(),
        };

        operations.add_project(project).unwrap();
    }

    #[test]
    fn when_adding_project_to_config_with_another_project_it_shall_add_it_to_the_configuration() {
        let mut registry = DeviceFactoryRegistry::new();
        registry.register_device("MockDevice".to_string(), "Mock Device".to_string(), || {
            Box::new(MockDeviceFactory)
        });

        let mut provider = MockGlobalConfigProvider::new();
        provider
            .expect_read_global_config()
            .return_const(Ok(r#"[[projects]]
name = "AnotherProject"
path = "/path/to/project"

[projects.tracking_status]
type = "IgnoredProject"
"#
            .to_string()));
        provider
            .expect_write_global_config()
            .times(1)
            .with(eq(r#"[[projects]]
name = "AnotherProject"
path = "/path/to/project"

[projects.tracking_status]
type = "IgnoredProject"

[[projects]]
name = "MyProject"
path = "/path/to/project2"

[projects.tracking_status]
last_update = "0"
type = "TrackedProject"

[projects.tracking_status.backup_requirement_class]
min_security_level = "NetworkUntrustedRestricted"
name = "Default"
target_copies = 3
target_locations = 2
"#
            .to_string()))
            .return_const(Ok(()));

        let operations = Operations {
            device_factory_registry: registry,
            global_config_provider: Box::new(provider),
        };

        let project = AddProjectArgs {
            name: "MyProject".to_string(),
            location: "/path/to/project2".to_string(),
        };

        operations.add_project(project).unwrap();
    }

    #[test]
    fn when_removing_last_project_by_name_it_shall_update_the_configuration() {
        let mut registry = DeviceFactoryRegistry::new();
        registry.register_device("MockDevice".to_string(), "Mock Device".to_string(), || {
            Box::new(MockDeviceFactory)
        });

        let mut provider = MockGlobalConfigProvider::new();
        provider
            .expect_read_global_config()
            .return_const(Ok(r#"[[projects]]
name = "AnotherProject"
path = "/path/to/project"

[projects.tracking_status]
type = "IgnoredProject"
"#
            .to_string()));
        provider
            .expect_write_global_config()
            .times(1)
            .with(eq(r#""#.to_string()))
            .return_const(Ok(()));

        let operations = Operations {
            device_factory_registry: registry,
            global_config_provider: Box::new(provider),
        };

        operations
            .remove_project_by_name("AnotherProject".to_string())
            .unwrap();
    }

    #[test]
    fn when_removing_project_not_in_config_it_shall_fail() {
        let operations = Operations {
            device_factory_registry: DeviceFactoryRegistry::new(),
            global_config_provider: Box::new(MockGlobalConfigProviderFactory::new(r#""#)),
        };

        let result = operations.remove_project_by_name("NotInConfig".to_string());
        assert!(result.err().unwrap().contains("Project not found"));
    }

    #[test]
    fn when_removing_project_from_config_with_2_projects_it_shall_only_remove_one() {
        let mut registry = DeviceFactoryRegistry::new();
        registry.register_device("MockDevice".to_string(), "Mock Device".to_string(), || {
            Box::new(MockDeviceFactory)
        });

        let mut provider = MockGlobalConfigProvider::new();
        provider
            .expect_read_global_config()
            .return_const(Ok(r#"[[projects]]
name = "AnotherProject"
path = "/path/to/project"

[projects.tracking_status]
type = "IgnoredProject"

[[projects]]
name = "MyProject"
path = "/path/to/project2"

[projects.tracking_status]
last_update = "0"
type = "TrackedProject"

[projects.tracking_status.backup_requirement_class]
min_security_level = "NetworkUntrustedRestricted"
name = "Default"
target_copies = 3
target_locations = 2
"#
            .to_string()));
        provider
            .expect_write_global_config()
            .times(1)
            .with(eq(r#"[[projects]]
name = "AnotherProject"
path = "/path/to/project"

[projects.tracking_status]
type = "IgnoredProject"
"#
            .to_string()))
            .return_const(Ok(()));

        let operations = Operations {
            device_factory_registry: registry,
            global_config_provider: Box::new(provider),
        };

        operations
            .remove_project_by_name("MyProject".to_string())
            .unwrap();
    }
}
