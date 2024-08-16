use crate::{
    core::config::GlobalConfig,
    models::{
        backup_requirement::BackupRequirementClass,
        project::{Project, ProjectTrackingStatus},
    },
    now,
};
use std::time::SystemTime;

use super::{AddProjectArgs, Operations, ProjectOperations};

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

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use super::*;
    use crate::{
        core::{
            config::MockGlobalConfigProvider,
            device_factories_registry::DeviceFactoryRegistry,
            operations::Operations,
            test_utils::mocks::{MockDeviceFactory, MockGlobalConfigProviderFactory},
        },
        models::backup_requirement::SecurityLevel,
    };

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
