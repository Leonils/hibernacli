use std::{fs::Metadata, time::SystemTime};

pub trait MetadataExt {
    fn ctime_ms(&self) -> u64;
    fn mtime_ms(&self) -> u64;
}

impl MetadataExt for Metadata {
    fn ctime_ms(&self) -> u64 {
        self.created()
            .unwrap()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn mtime_ms(&self) -> u64 {
        self.modified()
            .unwrap()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}
