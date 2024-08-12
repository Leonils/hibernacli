use std::path::PathBuf;
use walkdir::WalkDir;

use crate::core::util::metadata::MetadataExt;

use super::backup_index::BackupIndex;

pub struct BackupExecution {
    index: BackupIndex,
    root_path: PathBuf,
}

impl BackupExecution {
    pub fn new(index: BackupIndex, root_path: PathBuf) -> Self {
        Self { index, root_path }
    }

    pub fn execute(&mut self) -> Result<(), std::io::Error> {
        // Walk through the folder at root_path, and mark visited entries
        // in the index
        for entry in WalkDir::new(&self.root_path).min_depth(1) {
            let entry = entry.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            let path_relative_to_root = entry
                .path()
                .strip_prefix(&self.root_path)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            let metadata = entry.metadata()?;
            let ctime = metadata.ctime_ms();
            let mtime = metadata.mtime_ms();
            let size = metadata.len();

            if self
                .index
                .has_changed(path_relative_to_root, ctime, mtime, size)
            {
                println!("Changed: {}", path_relative_to_root.display());
            } else {
                println!("Unchanged: {}", path_relative_to_root.display());
            }
            self.index.mark_visited(&path_relative_to_root);
        }

        for entry in self.index.enumerate_unvisited_entries() {
            println!("Deleted: {}", entry.path().display());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::test_utils::fs::create_tmp_dir;
    use std::fs::{metadata, write};

    use super::*;

    #[test]
    fn test_backup_execution_one_file() {
        // Prepare directory structure
        let dir = create_tmp_dir();
        write(dir.join("unchanged.txt"), "Hello, world!").unwrap();
        write(dir.join("added.txt"), "Hello, world!").unwrap();
        write(dir.join("updated.txt"), "Hello, world!").unwrap();
        let metadata = metadata(dir.join("unchanged.txt")).unwrap();
        let ctime = metadata.ctime_ms();

        // Prepare index
        let index = BackupIndex::new()
            .insert(ctime, ctime, 13, PathBuf::from("unchanged.txt"))
            .insert(ctime - 10, ctime - 10, 13, PathBuf::from("updated.txt"))
            .insert(ctime, ctime, 13, PathBuf::from("deleted.txt"));

        // Run backup execution
        let mut execution = BackupExecution::new(index, dir);
        execution.execute().unwrap();
    }
}
