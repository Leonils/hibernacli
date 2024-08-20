use std::{collections::HashSet, path::PathBuf};

use crate::core::Extractor;

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
        // Create the destination directory if it doesn't exist (fails if it already exists)
        if self.restore_to.exists() {
            return Err(format!(
                "Destination directory already exists: {}",
                self.restore_to.display()
            ));
        }

        std::fs::create_dir_all(&self.restore_to)
            .map_err(|e| format!("Failed to create destination directory: {}", e))?;

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
