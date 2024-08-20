use std::{collections::HashSet, path::PathBuf};

pub trait DifferentialArchiveStep {
    fn get_step_name(&self) -> &str;
    fn extract_to(
        &self,
        to: &PathBuf,
        paths_to_extract: &HashSet<PathBuf>,
    ) -> Result<HashSet<PathBuf>, String>;
}

pub trait Extractor: DoubleEndedIterator<Item = Box<dyn DifferentialArchiveStep>> {}
