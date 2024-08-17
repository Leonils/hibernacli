#[cfg(test)]
use mockall::automock;

use super::{project::Project, Device};

mod from_toml;
mod global {
    mod devices;
    mod load;
    mod projects;
}
mod project;
mod to_toml;
mod toml_try_read;

pub struct GlobalConfig {
    devices: Vec<Box<dyn Device>>,
    projects: Vec<Project>,
}

#[cfg(test)]
impl GlobalConfig {
    pub fn new(devices: Vec<Box<dyn Device>>, projects: Vec<Project>) -> Self {
        Self { devices, projects }
    }
}

#[cfg_attr(test, automock)]
pub trait GlobalConfigProvider {
    fn init_global_config(&self) -> Result<(), String>;
    fn read_global_config(&self) -> Result<String, String>;
    fn write_global_config(&self, content: &str) -> Result<(), String>;
}
