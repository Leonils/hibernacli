use std::{path::PathBuf, time::Instant};

use crate::models::{
    backup_requirement::SecurityLevel,
    question::Question,
    secondary_device::{Device, DeviceFactory},
};

struct MountedFolder {
    name: Option<String>,
    path: PathBuf,
}

impl Device for MountedFolder {
    fn get_name(&self) -> String {
        format!("MountedFolder[{}]", self.path.display())
    }

    fn get_location(&self) -> String {
        self.path.display().to_string()
    }

    fn get_security_level(&self) -> SecurityLevel {
        SecurityLevel::Local
    }

    fn get_device_type_name(&self) -> String {
        "MountedFolder".to_string()
    }

    fn get_last_connection(&self) -> Option<Instant> {
        None
    }

    fn get_last_disconnection(&self) -> Option<Instant> {
        None
    }
}

#[derive(Default)]
struct MountedFolderFactory {
    path: Option<PathBuf>,
    name: Option<String>,
    step: u8,
}

impl DeviceFactory for MountedFolderFactory {
    fn get_question(&self) -> Question {
        todo!()
    }

    fn has_next(&self) -> bool {
        todo!()
    }

    fn build(&self) -> Box<dyn Device> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_mounted_folder() {
        let factory = MountedFolderFactory::default();
    }
}
