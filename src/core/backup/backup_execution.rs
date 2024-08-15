use std::{fmt::Display, fs::File, path::PathBuf};
use walkdir::WalkDir;

use crate::{
    core::util::timestamps::{MetadataExt, TimeStampError},
    models::secondary_device::ArchiveWriter,
};

use super::backup_index::{BackupIndex, ToBuffer};

#[derive(Debug)]
pub enum BackupExecutionError {
    IoError(std::io::Error),
    SystemTimeError(std::time::SystemTimeError),
    StripPrefixError,
}
impl From<std::path::StripPrefixError> for BackupExecutionError {
    fn from(_: std::path::StripPrefixError) -> Self {
        Self::StripPrefixError
    }
}
impl From<std::io::Error> for BackupExecutionError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}
impl From<TimeStampError> for BackupExecutionError {
    fn from(e: TimeStampError) -> Self {
        match e {
            TimeStampError::IoError(e) => Self::IoError(e),
            TimeStampError::SystemTimeError(e) => Self::SystemTimeError(e),
        }
    }
}
impl From<walkdir::Error> for BackupExecutionError {
    fn from(e: walkdir::Error) -> Self {
        Self::IoError(std::io::Error::from(e))
    }
}
impl Display for BackupExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::SystemTimeError(e) => write!(f, "System time error: {}", e),
            Self::StripPrefixError => write!(f, "Strip prefix error"),
        }
    }
}

pub struct BackupExecution {
    index: BackupIndex,
    new_index: BackupIndex,
    root_path: PathBuf,
    deleted_entries: Vec<PathBuf>,
}
impl BackupExecution {
    pub fn new(index: BackupIndex, root_path: PathBuf) -> Self {
        Self {
            index,
            root_path,
            new_index: BackupIndex::new(),
            deleted_entries: Vec::new(),
        }
    }

    pub fn execute(
        &mut self,
        mut archiver_writer: Box<dyn ArchiveWriter>,
    ) -> Result<(), BackupExecutionError> {
        // Walk through the folder at root_path, and mark visited entries
        // in the index
        for entry in WalkDir::new(&self.root_path).min_depth(1) {
            let entry = entry?;
            let path_relative_to_root = entry.path().strip_prefix(&self.root_path)?;
            let metadata = entry.metadata()?;
            let ctime = metadata.ctime_ms()?;
            let mtime = metadata.mtime_ms()?;
            let size = metadata.len();

            if self
                .index
                .has_changed(path_relative_to_root, ctime, mtime, size)
            {
                let mut file = File::open(entry.path())?;
                archiver_writer.add_file(
                    &mut file,
                    &PathBuf::from(path_relative_to_root),
                    ctime,
                    mtime,
                    size,
                );
            }

            self.index.mark_visited(&path_relative_to_root);
            self.new_index
                .insert(ctime, mtime, size, PathBuf::from(path_relative_to_root));
        }

        for entry in self.index.enumerate_unvisited_entries() {
            self.deleted_entries.push(PathBuf::from(entry.path()));
        }

        archiver_writer.finalize(&self.deleted_entries, &self.new_index.to_buffer()?);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_utils::fs::create_tmp_dir;

    struct MockArchiveWriter {
        added_files: Vec<(PathBuf, u128, u128, u64)>,
    }
    impl MockArchiveWriter {
        fn new() -> Self {
            Self {
                added_files: Vec::new(),
            }
        }
    }
    impl ArchiveWriter for MockArchiveWriter {
        fn add_file(
            &mut self,
            _file: &mut File,
            path: &PathBuf,
            ctime: u128,
            mtime: u128,
            size: u64,
        ) {
            self.added_files.push((path.clone(), ctime, mtime, size));
        }
        fn add_directory(&mut self, _path: &PathBuf, _ctime: u128, _mtime: u128) {
            panic!("Not implemented");
        }
        fn add_symlink(&mut self, _path: &PathBuf, _ctime: u128, _mtime: u128, _target: &PathBuf) {
            panic!("Not implemented");
        }
        fn finalize(&mut self, _deleted_files: &Vec<PathBuf>, _new_index: &Vec<u8>) {
            panic!("Not implemented");
        }
    }

    #[test]
    fn test_backup_execution_empty_empty() {
        // Prepare empty directory structure and empty index
        let dir = create_tmp_dir();
        let index = BackupIndex::new();

        // Run backup execution
        let mut execution = BackupExecution::new(index, dir);
        execution
            .execute(Box::new(MockArchiveWriter::new()))
            .unwrap();

        // Should be not deleted entries
        assert_eq!(execution.deleted_entries.len(), 0);

        // Should be no new entries in new index
        let new_index = execution.new_index;
        let expected_new_index = BackupIndex::new();
        assert_eq!(new_index, expected_new_index);
    }
}
