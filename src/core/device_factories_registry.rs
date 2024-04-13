use std::collections::HashMap;

use crate::models::secondary_device::DeviceFactory;

pub struct DeviceFactoryRegistry {
    devices: HashMap<String, Box<dyn DeviceFactory>>,
}

impl DeviceFactoryRegistry {
    pub fn new() -> Self {
        DeviceFactoryRegistry {
            devices: HashMap::new(),
        }
    }

    pub fn register_device(
        &mut self,
        device_factory_key: String,
        device_factory: Box<dyn DeviceFactory>,
    ) {
        self.devices.insert(device_factory_key, device_factory);
    }

    pub fn get_device_factory(&self, device_factory_key: &str) -> Option<&Box<dyn DeviceFactory>> {
        self.devices.get(device_factory_key)
    }

    pub fn list_factories(&self) -> Vec<String> {
        self.devices.keys().cloned().collect()
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use crate::{
        core::test_utils::mocks::MockDeviceFactory,
        models::{backup_requirement::SecurityLevel, question::Question, secondary_device::Device},
    };

    use super::*;

    #[test]
    fn test_registered_device_factory_no_factory() {
        let registry = DeviceFactoryRegistry::new();
        let device_factory = registry.get_device_factory("MockDevice");
        assert!(device_factory.is_none());
    }

    #[test]
    fn test_register_retrieve_device_factory() {
        let mut registry = DeviceFactoryRegistry::new();
        registry.register_device("MockDevice".to_string(), Box::new(MockDeviceFactory));

        let device_factory = registry.get_device_factory("MockDevice");
        assert!(device_factory.is_some());

        let device_factory = device_factory.unwrap();
        let device = device_factory.build();
        assert_eq!(device.get_name(), "MockDevice");
    }

    #[test]
    fn when_listing_factories_with_empty_registry_we_shall_get_empty() {
        let registry = DeviceFactoryRegistry::new();
        let factories = registry.list_factories();
        assert_eq!(0, factories.len());
    }

    #[test]
    fn when_listing_factories_with_one_registered_factory_we_shall_get_one() {
        let mut registry = DeviceFactoryRegistry::new();
        registry.register_device("MockDevice".to_string(), Box::new(MockDeviceFactory));

        let factories = registry.list_factories();
        assert_eq!(vec!["MockDevice"], factories);
    }
}
