use std::{
    collections::HashMap,
    io::{self, BufRead},
    path::PathBuf,
};

#[derive(Debug, PartialEq)]
struct BackupIndexEntry {
    ctime: u64,
    mtime: u64,
    size: u64,
    path: PathBuf,
}

impl BackupIndexEntry {
    pub fn from_buffer(buffer: &mut Vec<u8>) -> Result<Self, io::Error> {
        // Read the first 3 * 8 bytes as u64 values
        let (ctime, mtime, size) = (
            Self::read_u64(buffer, 0)?,
            Self::read_u64(buffer, 8)?,
            Self::read_u64(buffer, 16)?,
        );

        // Read the rest of the line as a path, excluding the newline character
        let path = String::from_utf8(buffer[24..buffer.len() - 1].to_vec())
            .map(|s| PathBuf::from(s))
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid data"))?;

        Ok(BackupIndexEntry {
            ctime,
            mtime,
            size,
            path,
        })
    }

    fn read_u64(buffer: &mut Vec<u8>, offset: usize) -> Result<u64, io::Error> {
        Ok(u64::from_le_bytes(
            buffer[offset..offset + 8]
                .try_into()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid data"))?,
        ))
    }
}

struct BackupIndex {
    index: HashMap<PathBuf, BackupIndexEntry>,
}

impl BackupIndex {
    pub fn from_index_reader(mut reader: impl BufRead) -> Result<Self, io::Error> {
        let mut index: HashMap<PathBuf, BackupIndexEntry> = HashMap::new();

        let mut buffer = Vec::new();
        while reader.read_until(b'\n', &mut buffer)? > 0 {
            // Parse the entry from the buffer
            let entry = BackupIndexEntry::from_buffer(&mut buffer)?;
            let path = entry.path.clone();

            // Insert the entry into the index
            index.insert(path.clone(), entry);
        }

        Ok(BackupIndex { index })
    }

    pub fn get_entry(&self, path: &PathBuf) -> Option<&BackupIndexEntry> {
        self.index.get(path)
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
            &BackupIndexEntry {
                ctime: 1,
                mtime: 2,
                size: 3,
                path: PathBuf::from("test.txt"),
            }
        );
    }
}
