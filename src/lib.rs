// Business logic implementation
mod core {
    #[cfg(test)]
    pub mod test_utils {
        pub mod mocks;
    }

    mod backup_execution;
    mod backup_exploration;
    mod device_factories_registry;
    mod global_config;
    pub mod operations;
    mod project_config;
    mod project_status;
    mod projects_scan;
    mod restore_execution;
}

// Public structures (low behavior, high data)
mod models {
    pub mod backup_requirement;
    pub mod project;
    pub mod question;
    pub mod secondary_device;
}

// Adapters (interfaces implemented by core)
pub mod adapters {
    pub mod operations;
    pub mod primary_device;
    mod secondary_device;
}

mod devices {
    mod local_file_storage;
}

pub mod cli;
