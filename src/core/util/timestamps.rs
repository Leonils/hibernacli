use std::{fs::Metadata, io, time::SystemTime};

pub enum TimeStampError {
    IoError(std::io::Error),
    SystemTimeError(std::time::SystemTimeError),
}

pub trait Timestamp {
    fn duration_since_epoch(self) -> Result<std::time::Duration, TimeStampError>;
    fn ms_since_epoch(self) -> Result<u128, TimeStampError>;
    fn s_since_epoch(self) -> Result<u64, TimeStampError>;
}

pub trait MetadataExt {
    fn ctime_ms(&self) -> Result<u128, TimeStampError>;
    fn mtime_ms(&self) -> Result<u128, TimeStampError>;
}

impl MetadataExt for Metadata {
    fn ctime_ms(&self) -> Result<u128, TimeStampError> {
        self.created().ms_since_epoch()
    }

    fn mtime_ms(&self) -> Result<u128, TimeStampError> {
        self.modified().ms_since_epoch()
    }
}

impl Timestamp for io::Result<SystemTime> {
    fn duration_since_epoch(self) -> Result<std::time::Duration, TimeStampError> {
        self.map_err(|e| TimeStampError::IoError(e))?
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(TimeStampError::SystemTimeError)
    }

    fn ms_since_epoch(self) -> Result<u128, TimeStampError> {
        Ok(self.duration_since_epoch()?.as_millis())
    }

    fn s_since_epoch(self) -> Result<u64, TimeStampError> {
        Ok(self.duration_since_epoch()?.as_secs())
    }
}
