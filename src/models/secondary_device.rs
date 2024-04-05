use std::time::Instant;

use super::backup_requirement::SecurityLevel;

pub struct Device {
    // The name of the device
    name: String,

    // The physical location of the device (home, work, aws, ...)
    location: String,

    // The security level of the device
    security_level: SecurityLevel,

    // The type of the device
    device_type_name: String,

    // The last time the device was connected
    last_connection: Option<Instant>,

    // The last time the device was disconnected
    last_disconnection: Option<Instant>,
}
