use std::{io::BufRead, time::Instant};

use crate::core::{
    config::MockGlobalConfigProvider,
    device::{ArchiveWriter, QuestionType},
    Device, DeviceFactory, SecurityLevel,
};

pub struct MockDeviceFactory;
impl DeviceFactory for MockDeviceFactory {
    fn get_question_statement(&self) -> &str {
        panic!("No question")
    }
    fn get_question_type(&self) -> &QuestionType {
        panic!("No question")
    }
    fn set_question_answer(&mut self, _answer: String) -> Result<(), String> {
        panic!("No question")
    }
    fn has_next(&self) -> bool {
        false
    }
    fn build(&self) -> Result<Box<dyn Device>, String> {
        Ok(Box::new(MockDevice {
            name: "MockDevice".to_string(),
        }))
    }
    fn build_from_toml_table(
        &self,
        name: &str,
        _table: &toml::value::Table,
    ) -> Result<Box<dyn Device>, String> {
        Ok(Box::new(MockDevice {
            name: name.to_string(),
        }))
    }
}

pub struct MockDeviceWithParametersFactory;
pub struct MockDeviceWithParameters {
    pub name: String,
    pub parameter: String,
}
impl MockDeviceWithParameters {
    pub fn new(name: &str, parameter: &str) -> MockDeviceWithParameters {
        MockDeviceWithParameters {
            name: name.to_string(),
            parameter: parameter.to_string(),
        }
    }
}
impl Device for MockDeviceWithParameters {
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_location(&self) -> String {
        self.parameter.clone()
    }
    fn get_device_type_name(&self) -> String {
        "MockDeviceWithParameters".to_string()
    }
    fn get_last_connection(&self) -> Option<Instant> {
        None
    }
    fn get_last_disconnection(&self) -> Option<Instant> {
        None
    }
    fn get_security_level(&self) -> SecurityLevel {
        SecurityLevel::Local
    }
    fn to_toml_table(&self) -> toml::value::Table {
        let mut table = toml::value::Table::new();
        table.insert("name".to_string(), self.get_name().into());
        table.insert("type".to_string(), self.get_device_type_name().into());
        table.insert("parameter".to_string(), self.parameter.clone().into());
        table
    }
    fn read_backup_index(&self, _project_name: &str) -> Result<Option<Box<dyn BufRead>>, String> {
        Ok(None)
    }
    fn test_availability(&self) -> Result<(), String> {
        Ok(())
    }
    fn get_archive_writer(&self, _project_name: &str) -> Box<dyn ArchiveWriter> {
        panic!("Mock not implemented for this use case")
    }
}
impl DeviceFactory for MockDeviceWithParametersFactory {
    fn get_question_statement(&self) -> &str {
        panic!("No question")
    }
    fn get_question_type(&self) -> &QuestionType {
        panic!("No question")
    }
    fn set_question_answer(&mut self, _answer: String) -> Result<(), String> {
        panic!("No question")
    }
    fn has_next(&self) -> bool {
        false
    }
    fn build(&self) -> Result<Box<dyn Device>, String> {
        panic!("Mock not implemented for this use case")
    }
    fn build_from_toml_table(
        &self,
        name: &str,
        table: &toml::value::Table,
    ) -> Result<Box<dyn Device>, String> {
        Ok(Box::new(MockDeviceWithParameters {
            name: name.to_string(),
            parameter: table
                .get("parameter")
                .ok_or_else(|| "Missing parameter".to_string())?
                .as_str()
                .ok_or_else(|| "Invalid string for parameter".to_string())?
                .to_string(),
        }))
    }
}

pub struct MockDevice {
    pub name: String,
}
impl MockDevice {
    pub fn new(name: &str) -> MockDevice {
        MockDevice {
            name: name.to_string(),
        }
    }
}
impl Device for MockDevice {
    fn get_name(&self) -> String {
        self.name.clone()
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
    fn to_toml_table(&self) -> toml::value::Table {
        let mut table = toml::value::Table::new();
        table.insert("name".to_string(), self.get_name().into());
        table.insert("type".to_string(), self.get_device_type_name().into());
        table
    }
    fn read_backup_index(&self, _project_name: &str) -> Result<Option<Box<dyn BufRead>>, String> {
        Ok(None)
    }
    fn test_availability(&self) -> Result<(), String> {
        Ok(())
    }
    fn get_archive_writer(&self, _project_name: &str) -> Box<dyn ArchiveWriter> {
        panic!("Mock not implemented for this use case")
    }
}

pub struct MockGlobalConfigProviderFactory;
impl MockGlobalConfigProviderFactory {
    pub fn new(global_config_toml: &str) -> MockGlobalConfigProvider {
        let mut provider = MockGlobalConfigProvider::new();
        provider
            .expect_read_global_config()
            .return_const(Ok(global_config_toml.to_string()));
        provider
    }

    pub fn new_failing_to_read() -> MockGlobalConfigProvider {
        let mut provider = MockGlobalConfigProvider::new();
        provider
            .expect_read_global_config()
            .return_const(Err("Error reading global config".to_string()));
        provider
    }
}
