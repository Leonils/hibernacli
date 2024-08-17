use std::{fmt::Display, io, time::SystemTime};

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const ONE_SECOND: Duration = Duration::from_secs(1);
    const EPOCH: SystemTime = SystemTime::UNIX_EPOCH;

    #[test]
    fn test_timestamp_duration_since_epoch_to_epoch() {
        assert_eq!(
            EPOCH.duration_since_epoch().unwrap(),
            Duration::from_secs(0)
        );
    }

    #[test]
    fn test_timestamp_duration_since_epoch_to_epoch_s() {
        assert_eq!(EPOCH.s_since_epoch().unwrap(), 0);
    }

    #[test]
    fn test_timestamp_duration_since_epoch_to_epoch_ms() {
        assert_eq!(EPOCH.ms_since_epoch().unwrap(), 0);
    }

    #[test]
    fn test_timestamp_duration_since_epoch_to_later() {
        assert_eq!(
            (EPOCH + ONE_SECOND).duration_since_epoch().unwrap(),
            Duration::from_secs(1)
        );
    }

    #[test]
    fn test_timestamp_duration_since_epoch_to_later_s() {
        assert_eq!((EPOCH + ONE_SECOND).s_since_epoch().unwrap(), 1);
    }

    #[test]
    fn test_timestamp_duration_since_epoch_to_later_ms() {
        assert_eq!((EPOCH + ONE_SECOND).ms_since_epoch().unwrap(), 1000);
    }

    #[test]
    fn test_timestamp_duration_since_epoch_to_earlier() {
        assert_eq!(
            (EPOCH - ONE_SECOND)
                .duration_since_epoch()
                .unwrap_err()
                .to_string(),
            "System time error: second time provided was later than self"
        );
    }

    #[test]
    fn test_timestamp_duration_since_epoch_to_earlier_s() {
        assert_eq!(
            (EPOCH - ONE_SECOND)
                .s_since_epoch()
                .unwrap_err()
                .to_string(),
            "System time error: second time provided was later than self"
        );
    }

    #[test]
    fn test_timestamp_duration_since_epoch_to_earlier_ms() {
        assert_eq!(
            (EPOCH - ONE_SECOND)
                .ms_since_epoch()
                .unwrap_err()
                .to_string(),
            "System time error: second time provided was later than self"
        );
    }

    #[test]
    fn test_timestamp_duration_since_epoch_io_error() {
        let result: io::Result<SystemTime> = Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No such file or directory",
        ));
        assert_eq!(
            result.duration_since_epoch().unwrap_err().to_string(),
            "IO error: No such file or directory"
        );
    }

    #[test]
    fn test_timestamp_duration_since_epoch_io_error_s() {
        let result: io::Result<SystemTime> = Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No such file or directory",
        ));
        assert_eq!(
            result.s_since_epoch().unwrap_err().to_string(),
            "IO error: No such file or directory"
        );
    }

    #[test]
    fn test_timestamp_duration_since_epoch_io_error_ms() {
        let result: io::Result<SystemTime> = Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No such file or directory",
        ));
        assert_eq!(
            result.ms_since_epoch().unwrap_err().to_string(),
            "IO error: No such file or directory"
        );
    }

    #[test]
    fn test_timestamp_duration_since_epoch_io_success() {
        assert_eq!(
            Ok(EPOCH + ONE_SECOND).duration_since_epoch().unwrap(),
            ONE_SECOND
        );
    }

    #[test]
    fn test_timestamp_duration_since_epoch_io_success_s() {
        assert_eq!(Ok(EPOCH + ONE_SECOND).s_since_epoch().unwrap(), 1);
    }

    #[test]
    fn test_timestamp_duration_since_epoch_io_success_ms() {
        assert_eq!(Ok(EPOCH + ONE_SECOND).ms_since_epoch().unwrap(), 1000);
    }

    #[test]
    fn test_timestamp_duration_since_epoch_io_success_time_error() {
        assert_eq!(
            Ok(EPOCH - ONE_SECOND)
                .duration_since_epoch()
                .unwrap_err()
                .to_string(),
            "System time error: second time provided was later than self"
        );
    }

    #[test]
    fn test_timestamp_duration_since_epoch_io_success_s_time_error() {
        assert_eq!(
            Ok(EPOCH - ONE_SECOND)
                .s_since_epoch()
                .unwrap_err()
                .to_string(),
            "System time error: second time provided was later than self"
        );
    }

    #[test]
    fn test_timestamp_duration_since_epoch_io_success_ms_time_error() {
        assert_eq!(
            Ok(EPOCH - ONE_SECOND)
                .ms_since_epoch()
                .unwrap_err()
                .to_string(),
            "System time error: second time provided was later than self"
        );
    }
}
