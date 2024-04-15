use toml::Table;

use crate::{adapters::primary_device::GlobalConfigProvider, models::secondary_device::Device};

use super::device_factories_registry::DeviceFactoryRegistry;

struct GlobalConfig {
    devices: Vec<Box<dyn Device>>,
}

#[derive(serde::Deserialize, Debug, Default)]
struct PartiallyParsedGlobalConfig {
    devices: Option<Vec<Table>>,
}

impl GlobalConfig {
    pub fn load(
        config_provider: impl GlobalConfigProvider,
        device_factories_registry: DeviceFactoryRegistry,
    ) -> Result<GlobalConfig, String> {
        let config_toml = config_provider.read_global_config_dir()?;

        let parsed_config: PartiallyParsedGlobalConfig =
            toml::from_str(&config_toml).map_err(|e| e.to_string())?;

        if parsed_config.devices.is_none() {
            return Ok(GlobalConfig { devices: vec![] });
        }

        let devices = parsed_config
            .devices
            .unwrap()
            .iter()
            .map(|device_table| {
                device_factories_registry
                    .get_device_factory(device_table["type"].as_str().unwrap())
                    .ok_or_else(|| "Device factory not found".to_string())
                    .and_then(|device_factory| device_factory.build_from_toml_table(device_table))
            })
            .collect::<Result<Vec<Box<dyn Device>>, String>>()?;

        Ok(GlobalConfig { devices })
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{
        device_factories_registry::{self, DeviceFactoryRegistry},
        test_utils::mocks::{MockDeviceFactory, MockGlobalConfigProvider},
    };

    use super::GlobalConfig;

    fn get_mock_device_factory_registry() -> DeviceFactoryRegistry {
        let mut registry = DeviceFactoryRegistry::new();
        registry.register_device(
            "MockDevice".to_string(),
            "MockDevice".to_string(),
            Box::new(MockDeviceFactory),
        );
        registry
    }

    #[test]
    fn when_failing_to_retrieve_config_it_shall_return_the_error() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProvider::new_failing_on_read();
        let config = GlobalConfig::load(config_provider, device_factories_registry);
        assert!(config.is_err());
    }

    #[test]
    fn when_retrieving_config_it_shall_return_the_config() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProvider::new("");
        let config = GlobalConfig::load(config_provider, device_factories_registry);
        assert!(config.is_ok());
    }

    #[test]
    fn when_retrieving_config_with_no_device_it_shall_have_no_device_in_global_config() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProvider::new("");
        let config = GlobalConfig::load(config_provider, device_factories_registry).unwrap();
        assert_eq!(config.devices.len(), 0);
    }

    #[test]
    fn when_retrieving_config_with_one_mock_device_it_shall_have_one_device_in_global_config() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProvider::new(
            r#"
[[devices]]
name = "MockDevice"
type = "MockDevice"
"#,
        );
        let config = GlobalConfig::load(config_provider, device_factories_registry).unwrap();
        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].get_device_type_name(), "MockDevice");
        assert_eq!(config.devices[0].get_name(), "MockDevice");
    }

    #[test]
    fn when_retrieving_config_with_different_name_it_shall_be_reflected_in_device() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProvider::new(
            r#"
[[devices]]
name = "MyPersonalDevice"
type = "MockDevice"
"#,
        );
        let config = GlobalConfig::load(config_provider, device_factories_registry).unwrap();
        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].get_device_type_name(), "MockDevice");
        assert_eq!(config.devices[0].get_name(), "MyPersonalDevice");
    }
}
