use std::time::Instant;

use super::backup_requirement::SecurityLevel;

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
}

pub enum DeviceFactoryQuestionType {
    Text,
    UnixPath,
}

pub struct DeviceFactoryQuestion {
    pub question: String,
    pub question_type: DeviceFactoryQuestionType,
    pub default: Option<String>,
    pub options: Option<Vec<String>>,
}

pub trait DeviceFactory {
    fn get_question(&self) -> DeviceFactoryQuestion;
    fn set_answer(&self, answer: String);
    fn has_next(&self) -> bool;
    fn build(&self) -> dyn Device;
}
