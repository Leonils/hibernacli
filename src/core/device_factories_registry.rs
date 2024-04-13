use std::collections::HashMap;

use crate::models::secondary_device::{DeviceFactory, DeviceFactoryKey};

pub struct DeviceFactoryBox {
    name: String,
    factory: Box<dyn DeviceFactory>,
}

pub struct DeviceFactoryRegistry {
    devices: HashMap<String, DeviceFactoryBox>,
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
        device_factory_readable_name: String,
        device_factory: Box<dyn DeviceFactory>,
    ) {
        self.devices.insert(
            device_factory_key.clone(),
            DeviceFactoryBox {
                name: device_factory_readable_name,
                factory: device_factory,
            },
        );
    }

    pub fn get_device_factory(&self, device_factory_key: &str) -> Option<&Box<dyn DeviceFactory>> {
        self.devices
            .get(device_factory_key)
            .map(|device_factory_box| &device_factory_box.factory)
    }

    pub fn list_factories(&self) -> Vec<DeviceFactoryKey> {
        self.devices
            .keys()
            .map(|key| DeviceFactoryKey {
                key: key.clone(),
                readable_name: self.devices.get(key).unwrap().name.clone(),
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use crate::core::test_utils::mocks::MockDeviceFactory;

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
        registry.register_device(
            "MockDevice".to_string(),
            "A mock device".to_string(),
            Box::new(MockDeviceFactory),
        );

        let device_factory = registry.get_device_factory("MockDevice");
        assert!(device_factory.is_some());

        let device_factory = device_factory.unwrap();
        let device = device_factory.build();
        assert_eq!(device.unwrap().get_name(), "MockDevice");
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
        registry.register_device(
            "MockDevice".to_string(),
            "A mock device".to_string(),
            Box::new(MockDeviceFactory),
        );

        let factories = registry.list_factories();
        assert_eq!(
            vec![DeviceFactoryKey {
                key: "MockDevice".to_string(),
                readable_name: "A mock device".to_string(),
            }],
            factories
        );
    }
}
