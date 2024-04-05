mod help {
    pub trait HelpOperations {
        fn help(&self) -> String;
        fn help_command(&self, command: String) -> String;
    }
}

mod device {
    use crate::models::secondary_device::{Device, DeviceFactory};

    pub trait DeviceOperations {
        fn get_device_factory(&self) -> impl DeviceFactory;
        fn add_device(&self, device: impl Device);
        fn remove_by_name(&self);
        fn list(&self) -> Vec<Box<dyn Device>>;
    }
}

mod project {
    use crate::models::project::Project;

    pub struct AddProjectArgs {
        pub name: String,
    }

    pub trait ProjectOperations {
        fn add(&self, args: AddProjectArgs) -> Project;
        fn remove_by_name(&self, name: String);
        fn list(&self) -> Vec<Project>;
    }
}
