use crate::{
    adapters::primary_device::GlobalConfigProvider, models::secondary_device::DeviceFactory,
};

use super::device_factories_registry::DeviceFactoryRegistry;

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

mod backup;
mod device;
mod project;

#[cfg(test)]
use crate::adapters::primary_device::MockGlobalConfigProvider;

#[cfg(test)]
impl Operations {
    fn new_with_mocked_dependencies() -> Self {
        Operations {
            device_factory_registry: DeviceFactoryRegistry::new(),
            global_config_provider: Box::new(MockGlobalConfigProvider::new()),
        }
    }
}
