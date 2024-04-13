use std::time::Instant;

use crate::models::{
    backup_requirement::SecurityLevel,
    question::Question,
    secondary_device::{Device, DeviceFactory},
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
