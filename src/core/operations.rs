use std::error::Error;

use crate::{
    adapters::operations::device::DeviceOperations,
    models::secondary_device::{Device, DeviceFactory, DeviceFactoryKey},
};

use super::device_factories_registry::DeviceFactoryRegistry;

struct Operations {
    device_factory_registry: DeviceFactoryRegistry,
}
impl Operations {
    fn new() -> Self {
        Operations {
            device_factory_registry: DeviceFactoryRegistry::new(),
        }
    }

    fn register_device_factory(
        &mut self,
        device_factory_key: String,
        device_factory_readable_name: String,
        device_factory: Box<dyn DeviceFactory>,
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

    fn get_device_factory(&self, device_type: String) -> Option<&Box<dyn DeviceFactory>> {
        self.device_factory_registry
            .get_device_factory(&device_type)
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
    use crate::core::test_utils::mocks::MockDeviceFactory;

    use super::*;

    #[test]
    fn with_an_empty_registry_no_factory_is_returned() {
        let operations = Operations::new();
        let available_factories = operations.get_available_device_factories();
        assert!(available_factories.is_empty());
    }

    #[test]
    fn after_registering_a_factory_it_can_be_retrieved() {
        let mut operations = Operations::new();
        operations.register_device_factory(
            "MockDevice".to_string(),
            "Mock Device".to_string(),
            Box::new(MockDeviceFactory),
        );

        let available_factories = operations.get_available_device_factories();
        assert_eq!(available_factories.len(), 1);
        assert_eq!(available_factories[0].key, "MockDevice");
        assert_eq!(available_factories[0].readable_name, "Mock Device");
    }

    #[test]
    fn if_no_factory_is_registered_none_is_returned_when_retrieving_a_factory() {
        let operations = Operations::new();
        let device_factory = operations.get_device_factory("MockDevice".to_string());
        assert!(device_factory.is_none());
    }

    #[test]
    fn after_registering_a_factory_it_can_be_used_to_create_a_device() {
        let mut operations = Operations::new();
        operations.register_device_factory(
            "MockDevice".to_string(),
            "Mock Device".to_string(),
            Box::new(MockDeviceFactory),
        );

        let device_factory = operations.get_device_factory("MockDevice".to_string());
        assert!(device_factory.is_some());

        let device_factory = device_factory.unwrap();
        let device = device_factory.build();
        assert_eq!(device.get_name(), "MockDevice");
    }

    #[test]
    fn when_registering_a_factory_and_retrieving_a_not_added_one_it_shall_return_none() {
        let mut operations = Operations::new();
        operations.register_device_factory(
            "MockDevice".to_string(),
            "Mock Device".to_string(),
            Box::new(MockDeviceFactory),
        );

        let device_factory = operations.get_device_factory("NotAdded".to_string());
        assert!(device_factory.is_none());
    }
}
