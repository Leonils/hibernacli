#[cfg(test)]
use super::config::MockGlobalConfigProvider;
use super::{
    device::{Device, DeviceFactory, DeviceFactoryKey, DeviceFactoryRegistry},
    project::Project,
    GlobalConfigProvider,
};

#[cfg(test)]
use mockall::automock;

mod backup;
mod device;
mod project;

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

#[cfg(test)]
impl Operations {
    fn new_with_mocked_dependencies() -> Self {
        Operations {
            device_factory_registry: DeviceFactoryRegistry::new(),
            global_config_provider: Box::new(MockGlobalConfigProvider::new()),
        }
    }
}

/// Manage devices where backups are stored
/// Devices are identified by their unique name
/// The consumer of these operations do not know what exact
/// type of device exists, so in order to add a device, the
/// consumer must first get the list of available device factories
/// and then use one of them to create a device and then add it to
/// the list of devices
///
/// It is then saved to a configuration file. From which
/// the list of devices is loaded at the start of the application
///
#[cfg_attr(test, automock)]
pub trait DeviceOperations {
    /// Get the list of available device factories
    /// Each one is identified by a key, and has a readable name
    fn get_available_device_factories(&self) -> Vec<DeviceFactoryKey>;

    /// Get a device factory by its key
    /// The key is the one returned by get_available_device_factories
    /// It might panic if the key does not exist
    fn get_device_factory(&self, device_type: String) -> Option<Box<dyn DeviceFactory>>;

    /// Add a device to the list of devices
    /// The device is built by the factory returned by get_device_factory
    fn add_device(&self, device: Box<dyn Device>) -> Result<(), Box<String>>;

    /// Once created, a device is identified by its unique name
    /// This function removes the device by its name
    fn remove_by_name(&self, name: String) -> Result<(), Box<String>>;

    /// List all devices
    /// The list is sorted by the device name
    fn list(&self) -> Result<Vec<Box<dyn Device>>, String>;
}

#[derive(Debug, PartialEq)]
pub struct AddProjectArgs {
    pub name: String,
    pub location: String,
}

/// Projects are a set of files that are a single unit for the user
/// The operations in this trait allow the user to manage the projects
///
/// The project backup configuration is saved locally in the project,
/// And only a reference to the project is saved in the global configuration
/// So that the project can be moved around without losing its configuration
///
#[cfg_attr(test, automock)]
pub trait ProjectOperations {
    /// Add a project to the list of projects
    /// The project is identified by its name, mut AddProjectArgs
    /// could be extended in the future to include more information
    fn add_project(&self, args: AddProjectArgs) -> Result<(), String>;

    /// A project shall be uniquely identified by its name
    /// So the name is enough to remove a project
    fn remove_project_by_name(&self, name: String) -> Result<(), String>;

    /// List all projects with their status
    fn list_projects(&self) -> Result<Vec<Project>, String>;
}

#[cfg_attr(test, automock)]
pub trait BackupOperations {
    /// Backup one project by its name to one device by its name
    fn backup_project_to_device(&self, project_name: &str, device_name: &str)
        -> Result<(), String>;
}
