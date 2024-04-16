use std::path::PathBuf;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait GlobalConfigProvider {
    fn init_global_config_dir(&self) -> Result<(), String>;
    fn read_global_config_dir(&self) -> Result<String, String>;
    fn write_global_config_dir(&self, content: &str) -> Result<(), String>;
}

pub trait PrimaryDeviceFilesProvider {
    fn create_file(&self, file_path: PathBuf) -> Result<(), String>;
    fn read_file(&self, file_path: PathBuf) -> Result<String, String>;
    fn write_file(&self, file_path: PathBuf, content: &str) -> Result<(), String>;
}
