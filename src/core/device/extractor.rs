use std::{collections::HashSet, io, path::PathBuf};

pub trait DifferentialArchiveStep {
    fn get_step_name(&self) -> &str;
    fn extract_to(
        &self,
        to: &PathBuf,
        paths_to_extract: &HashSet<PathBuf>,
    ) -> Result<HashSet<PathBuf>, ExtractorError>;
}

pub trait Extractor: DoubleEndedIterator<Item = Box<dyn DifferentialArchiveStep>> {}

#[derive(Debug)]
pub struct ExtractorError {
    pub message: String,
}
impl From<&str> for ExtractorError {
    fn from(message: &str) -> Self {
        ExtractorError {
            message: message.to_string(),
        }
    }
}
impl From<std::path::StripPrefixError> for ExtractorError {
    fn from(_: std::path::StripPrefixError) -> Self {
        ExtractorError {
            message: "Strip prefix error".to_string(),
        }
    }
}
impl From<io::Error> for ExtractorError {
    fn from(error: io::Error) -> Self {
        ExtractorError {
            message: error.to_string(),
        }
    }
}
