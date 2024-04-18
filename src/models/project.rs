use std::time::{Instant, SystemTime};

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

impl Project {
    pub fn new(
        name: String,
        location: String,
        tracking_status: Option<ProjectTrackingStatus>,
    ) -> Project {
        Project {
            name,
            location,
            tracking_status: tracking_status.unwrap_or(ProjectTrackingStatus::default()),
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_location(&self) -> &String {
        &self.location
    }

    pub fn get_tracking_status(&self) -> &ProjectTrackingStatus {
        &self.tracking_status
    }
}

pub enum ProjectTrackingStatus {
    TrackedProject {
        // The target backup requirement class
        backup_requirement_class: BackupRequirementClass,

        // Update date of the last updated file in the project
        last_update: Option<SystemTime>,
        // The actual copies of the project on secondary drives
        current_copies: Vec<Box<ProjectCopy>>,
    },
    UntrackedProject,
    IgnoredProject,
}

impl ProjectTrackingStatus {
    pub fn default() -> ProjectTrackingStatus {
        ProjectTrackingStatus::TrackedProject {
            backup_requirement_class: BackupRequirementClass::default(),
            last_update: Some(SystemTime::now()),
            current_copies: Vec::new(),
        }
    }

    pub fn get_backup_requirement_class(&self) -> Option<&BackupRequirementClass> {
        match self {
            ProjectTrackingStatus::TrackedProject {
                backup_requirement_class,
                ..
            } => Some(backup_requirement_class),
            _ => None,
        }
    }

    pub fn get_last_update(&self) -> Option<SystemTime> {
        match self {
            ProjectTrackingStatus::TrackedProject { last_update, .. } => *last_update,
            _ => None,
        }
    }

    pub fn get_current_copies(&self) -> Option<&Vec<Box<ProjectCopy>>> {
        match self {
            ProjectTrackingStatus::TrackedProject { current_copies, .. } => Some(current_copies),
            _ => None,
        }
    }
}

pub struct ProjectCopy {
    // What is the last time a backup was made
    last_backup: Option<Instant>,

    // What is the device on which it was done?
    secondary_device: dyn Device,
}
