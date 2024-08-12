use std::{io, path::PathBuf};
use walkdir::WalkDir;

use crate::core::util::metadata::MetadataExt;

use super::backup_index::{BackupIndex, ToBuffer};

pub trait ArchiveWriter {
    fn add_file(&mut self, path: &PathBuf, ctime: u64, mtime: u64, size: u64);
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
        archiver_writer: &mut impl ArchiveWriter,
    ) -> Result<(), std::io::Error> {
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
                archiver_writer.add_file(&PathBuf::from(path_relative_to_root), ctime, mtime, size);
            }

            self.index.mark_visited(&path_relative_to_root);
            self.new_index
                .insert(ctime, mtime, size, PathBuf::from(path_relative_to_root));
        }

        for entry in self.index.enumerate_unvisited_entries() {
            self.deleted_entries.push(PathBuf::from(entry.path()));
        }

        Ok(())
    }

    #[cfg(test)]
    pub fn from_new_index_and_deleted_entries(
        new_index: BackupIndex,
        deleted_entries: Vec<PathBuf>,
    ) -> Self {
        Self {
            index: BackupIndex::new(),
            new_index,
            root_path: PathBuf::new(),
            deleted_entries,
        }
    }
}

impl ToBuffer for BackupExecution {
    fn to_index_writer(&self, mut writer: impl io::Write) -> Result<(), io::Error> {
        // Number of entries
        writer.write_all(&self.new_index.index_size().to_le_bytes())?;

        // Write entries
        self.new_index.to_index_writer(&mut writer)?;

        // Write deleted entries
        for entry in &self.deleted_entries {
            writer.write_all(entry.to_str().unwrap().as_bytes())?;
            writer.write_all(b"\n")?;
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

    struct MockArchiveWriter {
        added_files: Vec<(PathBuf, u64, u64, u64)>,
    }
    impl MockArchiveWriter {
        fn new() -> Self {
            Self {
                added_files: Vec::new(),
            }
        }
    }
    impl ArchiveWriter for MockArchiveWriter {
        fn add_file(&mut self, path: &PathBuf, ctime: u64, mtime: u64, size: u64) {
            self.added_files.push((path.clone(), ctime, mtime, size));
        }
    }

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
        execution.execute(&mut MockArchiveWriter::new()).unwrap();

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
        let mut archiver = MockArchiveWriter::new();
        let mut execution = BackupExecution::new(index, dir);
        execution.execute(&mut archiver).unwrap();

        // Should be no deleted entries
        assert_eq!(execution.deleted_entries.len(), 0);

        // Check archiver
        assert_eq!(
            vec![(PathBuf::from("added.txt"), ctime, ctime, 13)],
            archiver.added_files
        );

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
        let mut archiver = MockArchiveWriter::new();
        let mut execution = BackupExecution::new(index, dir);
        execution.execute(&mut archiver).unwrap();

        // Nothing in the archive
        assert_eq!(archiver.added_files.len(), 0);

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
        let mut archiver = MockArchiveWriter::new();
        let mut execution = BackupExecution::new(index, dir);
        execution.execute(&mut archiver).unwrap();

        // Empty archive
        assert_eq!(archiver.added_files.len(), 0);

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
        let mut archiver = MockArchiveWriter::new();
        let mut execution = BackupExecution::new(index, dir);
        execution.execute(&mut archiver).unwrap();

        // Check archiver
        assert_eq!(
            vec![(PathBuf::from("updated.txt"), ctime, ctime, 13)],
            archiver.added_files
        );

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
        let mut archiver = MockArchiveWriter::new();
        let mut execution = BackupExecution::new(index, dir);
        execution.execute(&mut archiver).unwrap();

        // Check archiver
        assert_eq!(
            vec![
                (PathBuf::from("updated.txt"), ctime, ctime, 13),
                (PathBuf::from("added.txt"), ctime, ctime, 13),
            ],
            archiver.added_files
        );

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

    #[test]
    fn test_backup_execution_to_buffer() {
        let mock_backup_execution = BackupExecution::from_new_index_and_deleted_entries(
            BackupIndex::new()
                .with_entry(1, 2, 3, PathBuf::from("test1.txt"))
                .with_entry(4, 5, 6, PathBuf::from("test2.txt")),
            vec![PathBuf::from("deleted.txt")],
        );

        let mut buffer = Vec::new();
        mock_backup_execution.to_index_writer(&mut buffer).unwrap();

        assert_eq!(
            buffer,
            b"\x02\x00\x00\x00\x00\x00\x00\x00\
            \x01\x00\x00\x00\x00\x00\x00\x00\
            \x02\x00\x00\x00\x00\x00\x00\x00\
            \x03\x00\x00\x00\x00\x00\x00\x00\
            test1.txt\n\
            \x04\x00\x00\x00\x00\x00\x00\x00\
            \x05\x00\x00\x00\x00\x00\x00\x00\
            \x06\x00\x00\x00\x00\x00\x00\x00\
            test2.txt\n\
            deleted.txt\n"
        );
    }
}
