// Business logic implementation
mod core {
    #[cfg(test)]
    pub mod test_utils {
        pub mod fs;
        pub mod mocks;
    }

    pub mod util {
        pub mod buffer_ext;
        pub mod timestamps;
    }

    mod backup;
    mod config;
    mod device;
    mod project;

    pub mod operations;

    pub use config::GlobalConfigProvider;
    pub use device::SecurityLevel;
    pub use device::{
        ArchiveError, ArchiveWriter, Device, DeviceFactory, DeviceFactoryKey,
        DifferentialArchiveStep, Extractor, ExtractorError, Question, QuestionType,
    };

    #[cfg(test)]
    pub use device::{MockDevice, MockDeviceFactory};
}

mod devices {
    pub mod local_file_storage;
    pub mod mounted_folder;
    mod unpack_file_in;
}

pub mod cli;

pub mod macros;

pub mod run;
