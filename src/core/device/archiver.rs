use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};

use crate::core::util::timestamps::TimeStampError;

pub trait ArchiveWriter {
    fn add_file(
        &mut self,
        file: &mut File,
        path: &PathBuf,
        ctime: u128,
        mtime: u128,
        size: u64,
    ) -> Result<(), ArchiveError>;

    fn add_directory(
        &mut self,
        src_path: &Path,
        path: &PathBuf,
        ctime: u128,
        mtime: u128,
    ) -> Result<(), ArchiveError>;

    fn add_symlink(
        &mut self,
        path: &PathBuf,
        ctime: u128,
        mtime: u128,
        target: &PathBuf,
    ) -> Result<(), ArchiveError>;

    fn finalize(
        &mut self,
        deleted_files: &Vec<PathBuf>,
        new_index: &Vec<u8>,
    ) -> Result<(), ArchiveError>;
}

#[derive(Debug)]
pub struct ArchiveError {
    pub message: String,
}
impl From<&str> for ArchiveError {
    fn from(message: &str) -> Self {
        ArchiveError {
            message: message.to_string(),
        }
    }
}
impl From<io::Error> for ArchiveError {
    fn from(error: io::Error) -> Self {
        ArchiveError {
            message: error.to_string(),
        }
    }
}
impl From<TimeStampError> for ArchiveError {
    fn from(error: TimeStampError) -> Self {
        ArchiveError {
            message: error.to_string(),
        }
    }
}
