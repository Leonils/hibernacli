// Business logic implementation
mod core {
    #[cfg(test)]
    pub mod test_utils {
        pub mod fs;
        pub mod mocks;
    }

    mod backup;
    mod config;

    pub mod util {
        pub mod buffer_ext;
        pub mod timestamps;
    }

    mod backup_exploration;
    mod device_factories_registry;
    pub mod operations;
    mod project_status;
    mod projects_scan;
    mod restore_execution;

    pub use config::GlobalConfigProvider;
}

// Public structures (low behavior, high data)
mod models {
    pub mod backup_requirement;
    pub mod project;
    pub mod question;
    pub mod secondary_device;
}

mod devices {
    pub mod local_file_storage;
    pub mod mounted_folder;
}

pub mod cli;

pub mod macros;

pub mod run;
