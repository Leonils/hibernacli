use std::path::PathBuf;
use walkdir::WalkDir;

use crate::core::util::metadata::MetadataExt;

use super::backup_index::BackupIndex;

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
            self.new_index
                .insert(ctime, mtime, size, PathBuf::from(path_relative_to_root));
        }

        for entry in self.index.enumerate_unvisited_entries() {
            println!("Deleted: {}", entry.path().display());
            self.deleted_entries.push(PathBuf::from(entry.path()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::test_utils::fs::create_tmp_dir;
    use std::{
        fs::{metadata, write},
        time::SystemTime,
    };

    use super::*;

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    #[test]
    fn test_backup_execution_empty_empty() {
        // Prepare empty directory structure and empty index
        let dir = create_tmp_dir();
        let index = BackupIndex::new();

        // Run backup execution
        let mut execution = BackupExecution::new(index, dir);
        execution.execute().unwrap();

        // Should be not deleted entries
        assert_eq!(execution.deleted_entries.len(), 0);

        // Should be no new entries in new index
        let new_index = execution.new_index;
        let expected_new_index = BackupIndex::new();
        assert_eq!(new_index, expected_new_index);
    }

    #[test]
    fn test_backup_empty_index_added_file() {
        // Prepare empty directory structure and empty index
        let dir = create_tmp_dir();
        write(dir.join("added.txt"), "Hello, world!").unwrap();
        let ctime = metadata(dir.join("added.txt")).unwrap().ctime_ms();
        let index = BackupIndex::new();

        // Run backup execution
        let mut execution = BackupExecution::new(index, dir);
        execution.execute().unwrap();

        // Should be no deleted entries
        assert_eq!(execution.deleted_entries.len(), 0);

        // Should be one new entry in new index
        let new_index = execution.new_index;
        let expected_new_index =
            BackupIndex::new().with_entry(ctime, ctime, 13, PathBuf::from("added.txt"));
        assert_eq!(new_index, expected_new_index);
    }

    #[test]
    fn test_backup_deleted_file() {
        // Prepare directory structure
        let dir = create_tmp_dir();

        // Prepare index
        let ctime = now();
        let index = BackupIndex::new().with_entry(ctime, ctime, 13, PathBuf::from("deleted.txt"));

        // Run backup execution
        let mut execution = BackupExecution::new(index, dir);
        execution.execute().unwrap();

        // Check deleted entries
        assert_eq!(execution.deleted_entries.len(), 1);
        assert_eq!(execution.deleted_entries[0], PathBuf::from("deleted.txt"));

        // Check new index
        let new_index = execution.new_index;
        let expected_new_index = BackupIndex::new();
        assert_eq!(new_index, expected_new_index);
    }

    #[test]
    fn test_backup_execution_unchanged_file() {
        // Prepare directory structure
        let dir = create_tmp_dir();
        write(dir.join("unchanged.txt"), "Hello, world!").unwrap();
        let metadata = metadata(dir.join("unchanged.txt")).unwrap();
        let ctime = metadata.ctime_ms();

        // Prepare index
        let index = BackupIndex::new().with_entry(ctime, ctime, 13, PathBuf::from("unchanged.txt"));

        // Run backup execution
        let mut execution = BackupExecution::new(index, dir);
        execution.execute().unwrap();

        // Check deleted entries
        assert_eq!(execution.deleted_entries.len(), 0);

        // Check new index
        let new_index = execution.new_index;
        let expected_new_index =
            BackupIndex::new().with_entry(ctime, ctime, 13, PathBuf::from("unchanged.txt"));
        assert_eq!(new_index, expected_new_index);
    }

    #[test]
    fn test_backup_one_updated_file() {
        // Prepare directory structure
        let dir = create_tmp_dir();
        write(dir.join("updated.txt"), "Hello, world!").unwrap();
        let metadata = metadata(dir.join("updated.txt")).unwrap();
        let ctime = metadata.ctime_ms();

        // Prepare index
        let index =
            BackupIndex::new().with_entry(ctime - 10, ctime - 10, 13, PathBuf::from("updated.txt"));

        // Run backup execution
        let mut execution = BackupExecution::new(index, dir);
        execution.execute().unwrap();

        // Check deleted entries
        assert_eq!(execution.deleted_entries.len(), 0);

        // Check new index
        let new_index = execution.new_index;
        let expected_new_index =
            BackupIndex::new().with_entry(ctime, ctime, 13, PathBuf::from("updated.txt"));
        assert_eq!(new_index, expected_new_index);
    }

    #[test]
    fn test_backup_execution_several_files() {
        // Prepare directory structure
        let dir = create_tmp_dir();
        write(dir.join("unchanged.txt"), "Hello, world!").unwrap();
        write(dir.join("added.txt"), "Hello, world!").unwrap();
        write(dir.join("updated.txt"), "Hello, world!").unwrap();
        let metadata = metadata(dir.join("unchanged.txt")).unwrap();
        let ctime = metadata.ctime_ms();

        // Prepare index
        let index = BackupIndex::new()
            .with_entry(ctime, ctime, 13, PathBuf::from("unchanged.txt"))
            .with_entry(ctime - 10, ctime - 10, 13, PathBuf::from("updated.txt"))
            .with_entry(ctime, ctime, 13, PathBuf::from("deleted.txt"));

        // Run backup execution
        let mut execution = BackupExecution::new(index, dir);
        execution.execute().unwrap();

        // Check deleted entries
        assert_eq!(execution.deleted_entries.len(), 1);
        assert_eq!(execution.deleted_entries[0], PathBuf::from("deleted.txt"));

        // Check new index
        let new_index = execution.new_index;
        let expected_new_index = BackupIndex::new()
            .with_entry(ctime, ctime, 13, PathBuf::from("added.txt"))
            .with_entry(ctime, ctime, 13, PathBuf::from("unchanged.txt"))
            .with_entry(ctime, ctime, 13, PathBuf::from("updated.txt"));
        assert_eq!(new_index, expected_new_index);
    }
}
