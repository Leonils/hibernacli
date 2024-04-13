use std::time::Instant;

use crate::models::{
    backup_requirement::SecurityLevel,
    question::Question,
    secondary_device::{Device, DeviceFactory},
};

pub struct MockDeviceFactory;
impl DeviceFactory for MockDeviceFactory {
    fn get_question(&self) -> Question {
        panic!("No question")
    }
    fn has_next(&self) -> bool {
        false
    }
    fn build(&self) -> Box<dyn Device> {
        Box::new(MockDevice)
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
