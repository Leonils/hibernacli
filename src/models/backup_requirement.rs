pub enum SecurityLevel {
    // Connected to network, no authorization required
    NetworkPublic,       // referenced, accessible to anyone
    NetworkUnreferenced, // unreferenced, accessible to anyone

    // Connected to network, authorization required
    NetworkUntrustedRestricted, // untrusted provider (e.g. under the PATRIOT Act)
    NetworkTrustedRestricted,   // trusted provider

    // Connected to a local only network
    NetworkLocal, // local network, no internet access

    // Disconnected from network
    Local,            // local location
    LocalMaxSecurity, // local location with maximum security (in a safe?)
}

pub struct BackupRequirementClass {
    // The number of distinct copies we want to keep (including the primary)
    target_copies: u32,

    // The number of distinct physical locations we want to keep the copies
    target_locations: u32,

    // The minimum security level of the backups
    min_security_level: SecurityLevel,

    // Name of the backup requirement class
    name: String,
}

impl BackupRequirementClass {
    pub fn new(
        target_copies: u32,
        target_locations: u32,
        min_security_level: SecurityLevel,
        name: String,
    ) -> BackupRequirementClass {
        BackupRequirementClass {
            target_copies,
            target_locations,
            min_security_level,
            name,
        }
    }

    pub fn get_target_copies(&self) -> u32 {
        self.target_copies
    }

    pub fn get_target_locations(&self) -> u32 {
        self.target_locations
    }

    pub fn get_min_security_level(&self) -> &SecurityLevel {
        &self.min_security_level
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
}

impl Default for BackupRequirementClass {
    fn default() -> Self {
        BackupRequirementClass {
            target_copies: 3,
            target_locations: 2,
            min_security_level: SecurityLevel::NetworkUntrustedRestricted,
            name: "Default".to_string(),
        }
    }
}
