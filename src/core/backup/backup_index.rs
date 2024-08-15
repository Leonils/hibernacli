use std::{
    collections::BTreeMap,
    io::{self, BufRead},
    path::{Path, PathBuf},
};

use crate::core::util::buffer_ext::BufferExt;

pub trait ToBuffer {
    fn to_buffer(&self) -> Result<Vec<u8>, io::Error>;
}

#[derive(Debug, PartialEq)]
pub struct BackupIndexEntry {
    ctime: u64,
    mtime: u64,
    size: u64,
    path: PathBuf,
    visited: bool,
}

impl BackupIndexEntry {
    fn new(ctime: u64, mtime: u64, size: u64, path: PathBuf) -> Self {
        BackupIndexEntry {
            ctime,
            mtime,
            size,
            path,
            visited: false,
        }
    }

    fn from_buffer(buffer: &mut Vec<u8>) -> Result<Self, io::Error> {
        // Read the first 3 * 8 bytes as u64 values
        let (ctime, mtime, size) = (
            buffer
                .read_u64_from_le(0)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid data"))?,
            buffer
                .read_u64_from_le(8)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid data"))?,
            buffer
                .read_u64_from_le(16)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid data"))?,
        );

        // Read the rest of the line as a path, excluding the newline character
        let path = String::from_utf8(buffer[24..buffer.len() - 1].to_vec())
            .map(|s| PathBuf::from(s))
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid data"))?;

        Ok(BackupIndexEntry::new(ctime, mtime, size, path))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl ToBuffer for BackupIndexEntry {
    fn to_buffer(&self) -> Result<Vec<u8>, io::Error> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&self.ctime.to_le_bytes());
        buffer.extend_from_slice(&self.mtime.to_le_bytes());
        buffer.extend_from_slice(&self.size.to_le_bytes());
        buffer.extend_from_slice(self.path.to_str().unwrap().as_bytes());
        buffer.push(b'\n');
        Ok(buffer)
    }
}

#[derive(Debug, PartialEq)]
pub struct BackupIndex {
    index: BTreeMap<PathBuf, BackupIndexEntry>,
}

impl BackupIndex {
    pub fn new() -> Self {
        BackupIndex {
            index: BTreeMap::new(),
        }
    }

    pub fn from_index_reader(mut reader: impl BufRead) -> Result<Self, io::Error> {
        let mut index = BTreeMap::new();

        let mut buffer = Vec::new();
        while reader.read_until(b'\n', &mut buffer)? > 0 {
            // Parse the entry from the buffer
            let entry = BackupIndexEntry::from_buffer(&mut buffer)?;
            let path = entry.path.clone();

            // Insert the entry into the index
            index.insert(path.clone(), entry);
            buffer.clear();
        }

        Ok(BackupIndex { index })
    }

    pub fn insert(&mut self, ctime: u64, mtime: u64, size: u64, path: PathBuf) {
        let entry = BackupIndexEntry::new(ctime, mtime, size, path);
        self.index.insert(entry.path.clone(), entry);
    }

    pub fn has_changed(&self, path: &Path, ctime: u64, mtime: u64, size: u64) -> bool {
        match self.index.get(path) {
            Some(entry) => entry.ctime != ctime || entry.mtime != mtime || entry.size != size,
            None => true,
        }
    }

    pub fn mark_visited(&mut self, path: &Path) {
        if let Some(entry) = self.index.get_mut(path) {
            entry.visited = true;
        }
    }

    pub fn enumerate_unvisited_entries(&self) -> impl Iterator<Item = &BackupIndexEntry> {
        self.index
            .values()
            .filter(|entry| !entry.visited)
            .into_iter()
    }

    #[cfg(test)]
    pub fn with_entry(mut self, ctime: u64, mtime: u64, size: u64, path: PathBuf) -> Self {
        self.insert(ctime, mtime, size, path);
        self
    }

    #[cfg(test)]
    pub fn get_entry(&self, path: &Path) -> Option<&BackupIndexEntry> {
        self.index.get(path)
    }
}

impl ToBuffer for BackupIndex {
    fn to_buffer(&self) -> Result<Vec<u8>, io::Error> {
        let mut buffer = Vec::new();
        for entry in self.index.values() {
            buffer.extend_from_slice(&entry.to_buffer()?);
        }
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    #[test]
    fn test_read_from_empty_file() {
        let reader = Cursor::new(b"");
        let index = BackupIndex::from_index_reader(BufReader::new(reader)).unwrap();
        assert_eq!(index.index.len(), 0);
    }

    #[test]
    fn test_read_from_single_line_file() {
        // Create a buffer with a single line
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&u64::to_le_bytes(1));
        buffer.extend_from_slice(&u64::to_le_bytes(2));
        buffer.extend_from_slice(&u64::to_le_bytes(3));
        buffer.extend_from_slice(b"test.txt\n");

        // Create a reader from the buffer
        let reader = Cursor::new(buffer);
        let index = BackupIndex::from_index_reader(BufReader::new(reader)).unwrap();
        assert_eq!(index.index.len(), 1);
        assert_eq!(
            index.get_entry(&PathBuf::from("test.txt")).unwrap(),
            &BackupIndexEntry::new(1, 2, 3, PathBuf::from("test.txt")),
        );
    }

    #[test]
    fn test_empty_index_to_buffer() {
        let buffer = BackupIndex::new().to_buffer().unwrap();
        assert_eq!(buffer, b"");
    }

    #[test]
    fn test_write_single_entry_index() {
        let buffer = BackupIndex::new()
            .with_entry(1, 2, 3, PathBuf::from("test.txt"))
            .to_buffer()
            .unwrap();

        assert_eq!(
            buffer,
            b"\x01\x00\x00\x00\x00\x00\x00\x00\
            \x02\x00\x00\x00\x00\x00\x00\x00\
            \x03\x00\x00\x00\x00\x00\x00\x00\
            test.txt\n"
        );
    }

    #[test]
    fn test_write_read_index_with_2_entries() {
        let buffer = BackupIndex::new()
            .with_entry(1, 2, 3, PathBuf::from("test1.txt"))
            .with_entry(4, 5, 6, PathBuf::from("test2.txt"))
            .to_buffer()
            .unwrap();

        // Decode
        let reader = Cursor::new(buffer);
        let index = BackupIndex::from_index_reader(BufReader::new(reader)).unwrap();

        // Check
        assert_eq!(index.index.len(), 2);
        assert_eq!(
            index.get_entry(&PathBuf::from("test1.txt")).unwrap(),
            &BackupIndexEntry::new(1, 2, 3, PathBuf::from("test1.txt")),
        );
        assert_eq!(
            index.get_entry(&PathBuf::from("test2.txt")).unwrap(),
            &BackupIndexEntry::new(4, 5, 6, PathBuf::from("test2.txt")),
        );
    }

    #[test]
    fn test_not_found_file_has_changed() {
        let index = BackupIndex::new();
        assert!(index.has_changed(&PathBuf::from("test.txt"), 1, 2, 3));
    }

    #[test]
    fn test_found_old_file_mismatched_size_has_changed() {
        let index = BackupIndex::new().with_entry(1, 2, 3, PathBuf::from("test.txt"));
        assert!(index.has_changed(&PathBuf::from("test.txt"), 1, 2, 4));
    }

    #[test]
    fn test_found_old_file_mismatched_ctime_has_changed() {
        let index = BackupIndex::new().with_entry(1, 2, 3, PathBuf::from("test.txt"));
        assert!(index.has_changed(&PathBuf::from("test.txt"), 2, 2, 3));
    }

    #[test]
    fn test_found_old_file_mismatched_mtime_has_changed() {
        let index = BackupIndex::new().with_entry(1, 2, 3, PathBuf::from("test.txt"));
        assert!(index.has_changed(&PathBuf::from("test.txt"), 1, 3, 3));
    }

    #[test]
    fn test_found_old_file_has_not_changed() {
        let index = BackupIndex::new().with_entry(1, 2, 3, PathBuf::from("test.txt"));
        assert!(!index.has_changed(&PathBuf::from("test.txt"), 1, 2, 3));
    }

    #[test]
    fn test_mark_visited() {
        let mut index = BackupIndex::new()
            .with_entry(1, 2, 3, PathBuf::from("test.txt"))
            .with_entry(4, 5, 6, PathBuf::from("test2.txt"));
        index.mark_visited(&PathBuf::from("test.txt"));

        let unvisited_entries: Vec<&BackupIndexEntry> =
            index.enumerate_unvisited_entries().collect();
        assert_eq!(unvisited_entries.len(), 1);
        assert_eq!(unvisited_entries[0].path, PathBuf::from("test2.txt"));
    }
}
