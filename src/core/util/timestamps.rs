use std::{fmt::Display, fs::Metadata, io, time::SystemTime};

#[derive(Debug)]
pub enum TimeStampError {
    IoError(std::io::Error),
    SystemTimeError(std::time::SystemTimeError),
}
impl Display for TimeStampError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeStampError::IoError(e) => write!(f, "IO error: {}", e),
            TimeStampError::SystemTimeError(e) => write!(f, "System time error: {}", e),
        }
    }
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

impl Timestamp for SystemTime {
    fn duration_since_epoch(self) -> Result<std::time::Duration, TimeStampError> {
        self.duration_since(SystemTime::UNIX_EPOCH)
            .map_err(TimeStampError::SystemTimeError)
    }

    fn ms_since_epoch(self) -> Result<u128, TimeStampError> {
        Ok(self.duration_since_epoch()?.as_millis())
    }

    fn s_since_epoch(self) -> Result<u64, TimeStampError> {
        Ok(self.duration_since_epoch()?.as_secs())
    }
}

impl Timestamp for io::Result<SystemTime> {
    fn duration_since_epoch(self) -> Result<std::time::Duration, TimeStampError> {
        self.map_err(|e| TimeStampError::IoError(e))?
            .duration_since_epoch()
    }

    fn ms_since_epoch(self) -> Result<u128, TimeStampError> {
        Ok(self.duration_since_epoch()?.as_millis())
    }

    fn s_since_epoch(self) -> Result<u64, TimeStampError> {
        Ok(self.duration_since_epoch()?.as_secs())
    }
}
