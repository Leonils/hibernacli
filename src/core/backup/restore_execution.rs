use std::{collections::HashSet, fmt::Display, io, path::PathBuf};

use crate::core::{Extractor, ExtractorError};

use super::BackupIndex;

/// Represents the execution of a restore operation
///
/// This module assumes that the backup index can be loaded and that all steps are valid
/// Each step is an archive containing the new/updated files and a list to files to deleted.
/// For best performances, we can apply the steps in reverse order, starting from the last step,
/// then only apply the files that are are in the final index and not yet in the destination.

pub struct RestoreExecution {
    index: BackupIndex,
    restore_to: PathBuf,
    extractor: Box<dyn Extractor>,
}

impl RestoreExecution {
    pub fn new(
        index: BackupIndex,
        restoration_path: PathBuf,
        extractor: Box<dyn Extractor>,
    ) -> RestoreExecution {
        RestoreExecution {
            index,
            restore_to: restoration_path,
            extractor,
        }
    }

    pub fn extract(&mut self) -> Result<(), RestoreExecutionError> {
        // Create the destination directory if it doesn't exist (fails if it already exists)
        if self.restore_to.exists() {
            return Err(RestoreExecutionError::TargetDirectoryAlreadyExists(
                self.restore_to.to_string_lossy().to_string(),
            ));
        }
        std::fs::create_dir_all(&self.restore_to)?;

        // Extract from index the current list of files that should be in the destination
        let mut paths_to_extract: HashSet<PathBuf> = self
            .index
            .enumerate_entries()
            .map(|entry| entry.path().to_path_buf())
            .collect();

        // Extract steps in reverse order, extracting only the files that are not yet in the destination
        // and that are in the final index (so we get the last version of each file of the final state)
        for step in self.extractor.by_ref().rev() {
            println!("Extracting step {}", step.get_step_name());
            let extracted_paths = step.extract_to(&self.restore_to, &paths_to_extract)?;

            // Remove the already extracted paths, so we don't extract them again
            paths_to_extract = paths_to_extract
                .difference(&extracted_paths)
                .cloned()
                .collect();
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum RestoreExecutionError {
    TargetDirectoryAlreadyExists(String),
    IoError(String),
    ExtractorError(String),
}

impl From<ExtractorError> for RestoreExecutionError {
    fn from(e: ExtractorError) -> Self {
        Self::ExtractorError(e.message)
    }
}
impl From<io::Error> for RestoreExecutionError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e.to_string())
    }
}
impl Display for RestoreExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TargetDirectoryAlreadyExists(e) => {
                write!(f, "Target directory already exists: {}", e)
            }
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::ExtractorError(e) => write!(f, "ExtractorError error: {}", e),
        }
    }
}
