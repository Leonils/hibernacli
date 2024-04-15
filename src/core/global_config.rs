use itertools::Itertools;
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

        let (errors, devices) = toml::from_str::<PartiallyParsedGlobalConfig>(&config_toml)
            .map_err(|e| e.to_string())?
            .devices
            .unwrap_or(vec![])
            .into_iter()
            .map(|device_table| -> Result<Box<dyn Device>, String> {
                Self::load_device_from_toml_bloc(device_table, &device_factories_registry)
            })
            .into_iter()
            .partition_map(From::from);

        Self::assert_no_errors_in_devices(&errors)?;
        Self::assert_no_duplicate_device(&devices)?;

        Ok(GlobalConfig { devices })
    }

    fn load_device_from_toml_bloc(
        device_table: toml::map::Map<String, toml::Value>,
        device_factories_registry: &DeviceFactoryRegistry,
    ) -> Result<Box<dyn Device>, String> {
        let name = device_table
            .get("name")
            .ok_or_else(|| "Missing name for device".to_string())?
            .as_str()
            .ok_or_else(|| "Invalid string for name".to_string())?;

        let device_type: &str = device_table
            .get("type")
            .ok_or_else(|| "Type not found".to_string())?
            .as_str()
            .ok_or_else(|| "Invalid string for type".to_string())?;

        let factory = device_factories_registry
            .get_device_factory(device_type)
            .ok_or_else(|| "Device factory not found".to_string())?;

        let device = factory.build_from_toml_table(&name, &device_table)?;
        Ok(device)
    }

    fn assert_no_errors_in_devices(errors: &Vec<String>) -> Result<(), String> {
        if !errors.is_empty() {
            return Err(errors.join(", "));
        }

        Ok(())
    }

    fn assert_no_duplicate_device(devices: &Vec<Box<dyn Device>>) -> Result<(), String> {
        let device_count_by_name = devices
            .iter()
            .map(|device| device.get_name())
            .fold(std::collections::HashMap::new(), |mut acc, name| {
                *acc.entry(name).or_insert(0) += 1;
                acc
            })
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .sorted_by(|(name1, _), (name2, _)| name1.cmp(name2))
            .collect::<Vec<_>>();

        if !device_count_by_name.is_empty() {
            return Err(format!(
                "Duplicate device found in configuration file: {}",
                device_count_by_name.iter().map(|(name, _)| name).join(", ")
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{
        device_factories_registry::{self, DeviceFactoryRegistry},
        test_utils::mocks::{
            MockDeviceFactory, MockDeviceWithParametersFactory, MockGlobalConfigProvider,
        },
    };

    use super::GlobalConfig;

    fn get_mock_device_factory_registry() -> DeviceFactoryRegistry {
        let mut registry = DeviceFactoryRegistry::new();
        registry.register_device(
            "MockDevice".to_string(),
            "MockDevice".to_string(),
            Box::new(MockDeviceFactory),
        );
        registry.register_device(
            "MockDeviceWithParameters".to_string(),
            "MockDeviceWithParameters".to_string(),
            Box::new(MockDeviceWithParametersFactory),
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

    #[test]
    fn when_retrieving_config_with_multiple_devices_it_shall_have_all_devices_in_global_config() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProvider::new(
            r#"
[[devices]]
name = "MyPersonalDevice"
type = "MockDevice"

[[devices]]
name = "MySecondPersonalDevice"
type = "MockDevice"
"#,
        );
        let config = GlobalConfig::load(config_provider, device_factories_registry).unwrap();
        assert_eq!(config.devices.len(), 2);
        assert_eq!(config.devices[0].get_device_type_name(), "MockDevice");
        assert_eq!(config.devices[0].get_name(), "MyPersonalDevice");
        assert_eq!(config.devices[1].get_device_type_name(), "MockDevice");
        assert_eq!(config.devices[1].get_name(), "MySecondPersonalDevice");
    }

    #[test]
    fn if_type_is_not_in_registry_it_shall_return_error() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProvider::new(
            r#"
[[devices]]
name = "MyPersonalDevice"
type = "UnknownDevice"
"#,
        );
        let config = GlobalConfig::load(config_provider, device_factories_registry);
        assert!(config.is_err());
        assert_eq!(config.err().unwrap(), "Device factory not found");
    }

    #[test]
    fn if_name_is_missing_it_shall_fail() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProvider::new(
            r#"
[[devices]]
type = "MockDevice"
"#,
        );
        let config = GlobalConfig::load(config_provider, device_factories_registry);
        assert!(config.is_err());
        assert_eq!(config.err().unwrap(), "Missing name for device");
    }

    #[test]
    fn if_multiple_errors_in_device_it_shall_return_first_error_of_each_device() {
        let device_factories_registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProvider::new(
            r#"
[[devices]]
type = "MockDevice"

[[devices]]
name = "MyPersonalDevice"
type = "UnknownDevice"
"#,
        );
        let config = GlobalConfig::load(config_provider, device_factories_registry);
        assert!(config.is_err());
        assert_eq!(
            config.err().unwrap(),
            "Missing name for device, Device factory not found"
        );
    }

    #[test]
    fn when_there_are_multiple_device_types_each_device_type_shall_be_parsed() {
        let registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProvider::new(
            r#"
[[devices]]
name = "MyPersonalDevice"
type = "MockDevice"

[[devices]]
name = "MySecondPersonalDevice"
type = "MockDeviceWithParameters"
parameter = "MyParameter"
"#,
        );
        let config = GlobalConfig::load(config_provider, registry).unwrap();
        assert_eq!(config.devices.len(), 2);
        assert_eq!(config.devices[0].get_device_type_name(), "MockDevice");
        assert_eq!(config.devices[0].get_name(), "MyPersonalDevice");
        assert_eq!(
            config.devices[1].get_device_type_name(),
            "MockDeviceWithParameters"
        );
        assert_eq!(config.devices[1].get_name(), "MySecondPersonalDevice");
    }
    #[test]
    fn when_a_type_with_additional_parameters_has_a_missing_parameter_it_should_propagate_error() {
        let registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProvider::new(
            r#"
[[devices]]
name = "MySecondPersonalDevice"
type = "MockDeviceWithParameters"
"#,
        );
        let config = GlobalConfig::load(config_provider, registry);
        assert!(config.is_err());
        assert_eq!(config.err().unwrap(), "Missing parameter");
    }

    #[test]
    fn when_loading_configuration_if_there_is_2_devices_of_same_name_there_should_be_an_error() {
        let registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProvider::new(
            r#"
[[devices]]
name = "MyPersonalDevice"
type = "MockDevice"

[[devices]]
name = "MyPersonalDevice"
type = "MockDevice"
"#,
        );
        let config = GlobalConfig::load(config_provider, registry);
        assert!(config.is_err());
        assert_eq!(
            config.err().unwrap(),
            "Duplicate device found in configuration file: MyPersonalDevice"
        );
    }

    #[test]
    fn when_loading_configuration_if_there_is_4_devices_of_same_name_there_should_be_an_error() {
        let registry = get_mock_device_factory_registry();
        let config_provider = MockGlobalConfigProvider::new(
            r#"
[[devices]]
name = "MyPersonalDevice"
type = "MockDevice"

[[devices]]
name = "MyPersonalDevice"
type = "MockDevice"

[[devices]]
name = "MyOtherDevice"
type = "MockDevice"

[[devices]]
name = "MyOtherDevice"
type = "MockDevice"
"#,
        );
        let config = GlobalConfig::load(config_provider, registry);
        assert!(config.is_err());
        assert_eq!(
            config.err().unwrap(),
            "Duplicate device found in configuration file: MyOtherDevice, MyPersonalDevice"
        );
    }
}
