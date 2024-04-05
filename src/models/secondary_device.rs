use std::time::Instant;

use super::{backup_requirement::SecurityLevel, question::Question};

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

pub trait DeviceFactory {
    fn get_question(&self) -> Question;
    fn has_next(&self) -> bool;
    fn build(&self) -> impl Device;
}
