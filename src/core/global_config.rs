use crate::{adapters::primary_device::GlobalConfigProvider, models::secondary_device::Device};

struct GlobalConfig {
    devices: Vec<Box<dyn Device>>,
}

impl GlobalConfig {
    pub fn load(config_provider: impl GlobalConfigProvider) -> Result<GlobalConfig, String> {
        let config_toml = config_provider.read_global_config_dir()?;
        Ok(GlobalConfig { devices: vec![] })
    }
}

#[cfg(test)]
mod tests {
    use crate::core::test_utils::mocks::MockGlobalConfigProvider;

    use super::GlobalConfig;

    #[test]
    fn when_failing_to_retrieve_config_it_shall_return_the_error() {
        let config_provider = MockGlobalConfigProvider::new_failing_on_read();
        let config = GlobalConfig::load(config_provider);
        assert!(config.is_err());
    }

    #[test]
    fn when_retrieving_config_it_shall_return_the_config() {
        let config_provider = MockGlobalConfigProvider::new("");
        let config = GlobalConfig::load(config_provider);
        assert!(config.is_ok());
    }

    #[test]
    fn when_retrieving_config_with_no_device_it_shall_have_no_device_in_global_config() {
        let config_provider = MockGlobalConfigProvider::new("");
        let config = GlobalConfig::load(config_provider).unwrap();
        assert_eq!(config.devices.len(), 0);
    }
}
