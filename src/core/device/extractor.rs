pub trait DifferentialArchiveStep {
    fn get_step_name(&self) -> &str;
}

pub trait Extractor: DoubleEndedIterator<Item = Box<dyn DifferentialArchiveStep>> {}
