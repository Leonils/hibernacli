use std::error::Error;

use crate::{
    adapters::operations::device::DeviceOperations,
    models::secondary_device::{Device, DeviceFactory, DeviceFactoryKey},
};

struct Operations;

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
