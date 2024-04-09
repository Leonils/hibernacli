use std::time::Instant;

use super::{backup_requirement::BackupRequirementClass, secondary_device::Device};

pub struct Project {
    // The name of the project
    name: String,

    // An indicative location of the project location
    // Actual location type depends on the primary device type
    // The exact API might be found later
    location: String,

    // What is this project tracking status? might be explicitely
    // ignored, implicitly uncategorized, or tracked and ready to be
    // backed up.
    tracking_status: ProjectTrackingStatus,
}

pub enum ProjectTrackingStatus {
    TrackedProject {
        // The target backup requirement class
        backup_requirement_class: BackupRequirementClass,

        // Update date of the last updated file in the project
        last_update: Option<Instant>,

        // The actual copies of the project on secondary drives
        current_copies: Vec<Box<ProjectCopy>>,
    },
    UntrackedProject,
    IgnoredProject,
}

pub struct ProjectCopy {
    // What is the last time a backup was made
    last_backup: Option<Instant>,

    // What is the device on which it was done?
    secondary_device: dyn Device,
}
