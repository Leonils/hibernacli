use std::time::Instant;

use crate::{
    adapters::primary_device::GlobalConfigProvider,
    models::{
        backup_requirement::SecurityLevel,
        secondary_device::{Device, DeviceFactory},
    },
};

pub struct MockDeviceFactory;
impl DeviceFactory for MockDeviceFactory {
    fn get_question_statement(&self) -> &str {
        panic!("No question")
    }
    fn get_question_type(&self) -> &crate::models::question::QuestionType {
        panic!("No question")
    }
    fn set_question_answer(&mut self, _answer: String) -> Result<(), String> {
        panic!("No question")
    }
    fn has_next(&self) -> bool {
        false
    }
    fn build(&self) -> Result<Box<dyn Device>, String> {
        Ok(Box::new(MockDevice))
    }
}

pub struct MockDevice;
impl Device for MockDevice {
    fn get_name(&self) -> String {
        "MockDevice".to_string()
    }
    fn get_location(&self) -> String {
        "Home".to_string()
    }
    fn get_security_level(&self) -> SecurityLevel {
        SecurityLevel::NetworkUntrustedRestricted
    }
    fn get_device_type_name(&self) -> String {
        "MockDevice".to_string()
    }
    fn get_last_connection(&self) -> Option<Instant> {
        None
    }
    fn get_last_disconnection(&self) -> Option<Instant> {
        None
    }
}

pub struct MockGlobalConfigProvider {
    pub global_config_toml: String,
    pub fail_on_read: bool,
}
impl MockGlobalConfigProvider {
    pub fn new(global_config_toml: &str) -> Self {
        MockGlobalConfigProvider {
            global_config_toml: global_config_toml.to_string(),
            fail_on_read: false,
        }
    }

    pub fn new_failing_on_read() -> Self {
        MockGlobalConfigProvider {
            global_config_toml: "".to_string(),
            fail_on_read: true,
        }
    }
}
impl GlobalConfigProvider for MockGlobalConfigProvider {
    fn init_global_config_dir(&self) -> Result<(), String> {
        Ok(())
    }

    fn read_global_config_dir(&self) -> Result<String, String> {
        if self.fail_on_read {
            return Err("Failed to read global config".to_string());
        }
        Ok(self.global_config_toml.clone())
    }
}
