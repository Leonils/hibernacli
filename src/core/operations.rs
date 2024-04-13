use std::error::Error;

use crate::{
    adapters::operations::device::DeviceOperations,
    models::secondary_device::{Device, DeviceFactory, DeviceFactoryKey},
};

struct Operations;
impl Operations {
    fn new() -> Self {
        Operations
    }
}

impl DeviceOperations for Operations {
    fn get_available_device_factories(&self) -> Vec<DeviceFactoryKey> {
        todo!()
    }

    fn get_device_factory(&self, device_type: String) -> Option<&Box<dyn DeviceFactory>> {
        todo!()
    }

    fn add_device(&self, device: Box<dyn Device>) -> Result<Box<dyn Device>, Box<dyn Error>> {
        todo!()
    }

    fn remove_by_name(&self) {
        todo!()
    }

    fn list(&self) -> Vec<Box<dyn Device>> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn with_an_empty_registry_no_factory_is_returned() {
        let operations = Operations::new();
        let available_factories = operations.get_available_device_factories();
        assert!(available_factories.is_empty());
    }
}
