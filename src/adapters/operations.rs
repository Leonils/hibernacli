mod help {
    pub trait HelpOperations {
        /// Get an help for the struct implementing this trait
        fn help(&self) -> String;

        /// Get an help for a specific command of the struct implementing this trait
        fn help_command(&self, command: String) -> String;
    }
}

pub mod device {
    use std::error::Error;

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
    pub trait DeviceOperations {
        /// Get the list of available device factories
        /// Each one is identified by a key, and has a readable name
        fn get_available_device_factories(&self) -> Vec<DeviceFactoryKey>;

        /// Get a device factory by its key
        /// The key is the one returned by get_available_device_factories
        /// It might panic if the key does not exist
        fn get_device_factory(&self, device_type: String) -> Option<&Box<dyn DeviceFactory>>;

        /// Add a device to the list of devices
        /// The device is built by the factory returned by get_device_factory
        fn add_device(&self, device: Box<dyn Device>) -> Result<Box<dyn Device>, Box<dyn Error>>;

        /// Once created, a device is identified by its unique name
        /// This function removes the device by its name
        fn remove_by_name(&self);

        /// List all devices
        /// The list is sorted by the device name
        fn list(&self) -> Vec<Box<dyn Device>>;
    }
}

mod project {
    use crate::models::project::Project;

    pub struct AddProjectArgs {
        pub name: String,
    }

    /// Projects are a set of files that are a single unit for the user
    /// The operations in this trait allow the user to manage the projects
    ///
    /// The project backup configuration is saved locally in the project,
    /// And only a reference to the project is saved in the global configuration
    /// So that the project can be moved around without losing its configuration
    ///
    pub trait ProjectOperations {
        /// Add a project to the list of projects
        /// The project is identified by its name, mut AddProjectArgs
        /// could be extended in the future to include more information
        fn add(&self, args: AddProjectArgs) -> Project;

        /// A project shall be uniquely identified by its name
        /// So the name is enough to remove a project
        fn remove_by_name(&self, name: String);

        /// List all projects with their status
        fn list(&self) -> Vec<Project>;
    }
}
