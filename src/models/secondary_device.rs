#[cfg(test)]
use mockall::automock;

use std::time::Instant;

use super::{backup_requirement::SecurityLevel, question::QuestionType};

#[derive(Debug, PartialEq, Clone)]
pub struct DeviceFactoryKey {
    pub key: String,
    pub readable_name: String,
}

#[cfg_attr(test, automock)]
pub trait Device {
    // The name of the device
    fn get_name(&self) -> String;

    // The physical location of the device (home, work, aws, ...)
    fn get_location(&self) -> String;

    // The security level of the device
    fn get_security_level(&self) -> SecurityLevel;

    // The type of the device
    fn get_device_type_name(&self) -> String;

    // The last time the device was connected
    fn get_last_connection(&self) -> Option<Instant>;

    // The last time the device was disconnected
    fn get_last_disconnection(&self) -> Option<Instant>;

    // Serialize the device to a TOML table
    fn to_toml_table(&self) -> toml::value::Table;
}

pub trait DeviceFactory {
    fn get_question_statement(&self) -> &str;
    fn get_question_type(&self) -> &QuestionType;
    fn set_question_answer(&mut self, answer: String) -> Result<(), String>;
    fn has_next(&self) -> bool;
    fn build(&self) -> Result<Box<dyn Device>, String>;
    fn build_from_toml_table(
        &self,
        name: &str,
        table: &toml::value::Table,
    ) -> Result<Box<dyn Device>, String>;
}
