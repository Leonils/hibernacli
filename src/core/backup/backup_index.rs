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
    fn new(ctime: u64, mtime: u64, size: u64, path: PathBuf) -> Self {
        BackupIndexEntry {
            ctime,
            mtime,
            size,
            path,
        }
    }

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

    pub fn to_buffer(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&self.ctime.to_le_bytes());
        buffer.extend_from_slice(&self.mtime.to_le_bytes());
        buffer.extend_from_slice(&self.size.to_le_bytes());
        buffer.extend_from_slice(self.path.to_str().unwrap().as_bytes());
        buffer.push(b'\n');
        buffer
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
    pub fn new() -> Self {
        BackupIndex {
            index: HashMap::new(),
        }
    }

    pub fn from_index_reader(mut reader: impl BufRead) -> Result<Self, io::Error> {
        let mut index: HashMap<PathBuf, BackupIndexEntry> = HashMap::new();

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

    pub fn to_index_writer(&self, mut writer: impl io::Write) -> Result<(), io::Error> {
        for entry in self.index.values() {
            writer.write_all(&entry.to_buffer())?;
        }
        Ok(())
    }

    pub fn get_entry(&self, path: &PathBuf) -> Option<&BackupIndexEntry> {
        self.index.get(path)
    }

    fn insert(mut self, entry: BackupIndexEntry) -> Self {
        self.index.insert(entry.path.clone(), entry);
        self
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
    fn test_write_empty_index() {
        let index = BackupIndex::new();
        let mut writer = Vec::new();
        index.to_index_writer(&mut writer).unwrap();
        assert_eq!(writer, b"");
    }

    #[test]
    fn test_write_single_entry_index() {
        let index =
            BackupIndex::new().insert(BackupIndexEntry::new(1, 2, 3, PathBuf::from("test.txt")));

        let mut writer = Vec::new();
        index.to_index_writer(&mut writer).unwrap();
        assert_eq!(
            writer,
            b"\x01\x00\x00\x00\x00\x00\x00\x00\
            \x02\x00\x00\x00\x00\x00\x00\x00\
            \x03\x00\x00\x00\x00\x00\x00\x00\
            test.txt\n"
        );
    }

    #[test]
    fn test_write_read_index_with_2_entries() {
        let index = BackupIndex::new()
            .insert(BackupIndexEntry::new(1, 2, 3, PathBuf::from("test1.txt")))
            .insert(BackupIndexEntry::new(4, 5, 6, PathBuf::from("test2.txt")));

        // Encode
        let mut writer = Vec::new();
        index.to_index_writer(&mut writer).unwrap();

        // Decode
        let reader = Cursor::new(writer);
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
}
