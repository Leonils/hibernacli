use std::path::PathBuf;

pub trait PrimaryDevice {
    fn init_global_config_dir(&self) -> Result<(), String>;
    fn read_global_config_dir(&self) -> Result<String, String>;
    fn create_file(&self, file_path: PathBuf) -> Result<(), String>;
    fn read_file(&self, file_path: PathBuf) -> Result<String, String>;
    fn write_file(&self, file_path: PathBuf, content: &str) -> Result<(), String>;
}
