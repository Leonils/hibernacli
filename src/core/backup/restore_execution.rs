use std::{collections::HashSet, path::PathBuf};

use crate::core::{device::DifferentialArchiveStep, Extractor};

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

    pub fn extract(&mut self) -> Result<(), String> {
        let paths_to_extract: HashSet<PathBuf> = self
            .index
            .enumerate_entries()
            .map(|entry| entry.path().to_path_buf())
            .collect();

        for step in self.extractor.by_ref().rev() {
            println!("Extracting step {}", step.get_step_name());
            step.extract_to(&self.restore_to, &paths_to_extract)?;
        }

        // Extract the archive to the destination
        Ok(())
    }
}
