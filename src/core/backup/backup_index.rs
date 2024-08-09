use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    path::PathBuf,
};

#[derive(Debug, PartialEq)]
struct BackupIndexEntry {
    ctime: u64,
    mtime: u64,
    size: u64,
    path: PathBuf,
}

struct BackupIndex {
    index: HashMap<PathBuf, BackupIndexEntry>,
}

impl BackupIndex {
    pub fn from_index_reader(mut reader: impl BufRead) -> Self {
        let mut index: HashMap<PathBuf, BackupIndexEntry> = HashMap::new();

        let mut buffer = Vec::new();
        while reader.read_until(b'\n', &mut buffer).unwrap() > 0 {
            // Read the first 3 * 8 bytes as u64 values
            let (ctime, mtime, size) = (
                u64::from_le_bytes(buffer[0..8].try_into().unwrap()),
                u64::from_le_bytes(buffer[8..16].try_into().unwrap()),
                u64::from_le_bytes(buffer[16..24].try_into().unwrap()),
            );

            // Read the rest of the line as a path, excluding the newline character
            let path =
                PathBuf::from(String::from_utf8(buffer[24..buffer.len() - 1].to_vec()).unwrap());

            // Insert the entry into the index
            index.insert(
                path.clone(),
                BackupIndexEntry {
                    ctime,
                    mtime,
                    size,
                    path,
                },
            );
        }

        BackupIndex { index }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_from_empty_file() {
        let reader = Cursor::new(b"");
        let index = BackupIndex::from_index_reader(BufReader::new(reader));
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
        let index = BackupIndex::from_index_reader(BufReader::new(reader));
        assert_eq!(index.index.len(), 1);
        assert_eq!(
            index.index.get(&PathBuf::from("test.txt")).unwrap(),
            &BackupIndexEntry {
                ctime: 1,
                mtime: 2,
                size: 3,
                path: PathBuf::from("test.txt"),
            }
        );
    }
}
