// Business logic implementation
mod core {
    #[cfg(test)]
    pub mod test_utils {
        pub mod fs;
        pub mod mocks;
    }

    mod config {
        pub mod from_toml;
        pub mod to_toml;
        pub mod toml_try_read;
    }

    mod backup {
        pub mod backup_execution;
        pub mod backup_index;
    }

    pub mod util {
        pub mod buffer_ext;
        pub mod timestamps;
    }

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
    pub mod local_file_storage;
    pub mod mounted_folder;
}

pub mod cli;

pub mod macros;

pub mod run;
