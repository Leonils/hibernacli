use toml::Table;

use crate::core::global_config::GlobalConfig;

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
struct PartiallyParsedGlobalConfig {
    devices: Option<Vec<Table>>,
    projects: Option<Vec<Table>>,
}

pub trait ToToml {
    fn to_toml(&self) -> Result<String, String>;
}

impl ToToml for GlobalConfig {
    fn to_toml(&self) -> Result<String, String> {
        let device_tables = self
            .get_devices_iter()
            .map(|device| device.to_toml_table())
            .collect::<Vec<_>>();

        let config_toml = toml::to_string(&PartiallyParsedGlobalConfig {
            devices: if device_tables.is_empty() {
                None
            } else {
                Some(device_tables)
            },
            projects: None,
        })
        .map_err(|e| e.to_string())?;

        Ok(config_toml)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use crate::adapters::primary_device::MockGlobalConfigProvider;
    use crate::core::test_utils::mocks::{MockDevice, MockDeviceFactory, MockDeviceWithParameters};
    use crate::models::secondary_device::DeviceFactory;

    use super::super::super::global_config::GlobalConfig;
    use super::*;

    #[test]
    fn when_converting_empty_config_to_toml_it_shall_return_empty_string() {
        let global_config = GlobalConfig::new(vec![], vec![]);
        let toml = global_config.to_toml().unwrap();
        assert_eq!(toml, r#""#.trim());
    }

    #[test]
    fn when_converting_config_with_devices_to_toml_it_shall_return_toml() {
        let device = MockDeviceFactory.build().unwrap();
        let global_config = GlobalConfig::new(vec![device], vec![]);
        let toml = global_config.to_toml().unwrap();
        assert_eq!(
            toml,
            r#"[[devices]]
name = "MockDevice"
type = "MockDevice"
"#
        );
    }

    #[test]
    fn when_converting_config_with_multiple_devices_it_shall_save_config_with_devices() {
        let mut config_provider = MockGlobalConfigProvider::new();
        config_provider
            .expect_write_global_config()
            .times(1)
            .with(eq(r#"[[devices]]
name = "MockDevice"
type = "MockDevice"

[[devices]]
name = "MyDevice"
parameter = "MyParameter"
type = "MockDeviceWithParameters"
"#))
            .return_const(Ok(()));

        let device1 = MockDevice::new("MockDevice");
        let device2 = MockDeviceWithParameters::new("MyDevice", "MyParameter");
        let global_config = GlobalConfig::new(vec![Box::new(device1), Box::new(device2)], vec![]);
        global_config.save(&config_provider).unwrap();
    }
}
