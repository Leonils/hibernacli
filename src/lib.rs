// Business logic implementation
mod core {
    mod backup_execution;
    mod backup_exploration;
    mod global_config;
    mod project_config;
    mod project_status;
    mod projects_scan;
    mod restore_execution;
}

// Public structures (low behavior, high data)
mod models {
    mod backup_requirement;
    mod primary_device;
    mod project;
    mod secondary_device;
}

// Adapters (interfaces implemented by core)
mod adapters {
    mod operations;
    mod primary_device;
    mod secondary_device;
}

mod devices {
    mod local_unix_file_storage;
}

pub mod cli;
