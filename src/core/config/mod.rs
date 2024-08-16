use crate::models::{project::Project, secondary_device::Device};

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
