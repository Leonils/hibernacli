pub mod device {
    use std::rc::Rc;

    #[cfg(test)]
    use mockall::automock;

    use crate::models::secondary_device::{Device, DeviceFactory, DeviceFactoryKey};

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
}

pub mod project {
    use crate::models::project::Project;

    #[cfg(test)]
    use mockall::automock;

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
}
