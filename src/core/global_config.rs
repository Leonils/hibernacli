use crate::adapters::primary_device::GlobalConfigProvider;

struct GlobalConfig;

impl GlobalConfig {
    pub fn load(config_provider: impl GlobalConfigProvider) -> Result<GlobalConfig, String> {
        let config_toml = config_provider.read_global_config_dir()?;
        Ok(GlobalConfig)
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
        let config_provider = MockGlobalConfigProvider::new("config");
        let config = GlobalConfig::load(config_provider);
        assert!(config.is_ok());
    }
}
