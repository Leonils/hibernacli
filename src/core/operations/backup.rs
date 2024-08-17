use std::path::{Path, PathBuf};

use crate::core::{
    backup::{BackupExecution, BackupIndex, RestoreExecution},
    config::GlobalConfig,
    project::Project,
    Device,
};

use super::{BackupOperations, Operations};

impl Operations {
    fn get_project_and_device<'a>(
        &self,
        config: &'a GlobalConfig,
        project_name: &str,
        device_name: &str,
    ) -> Result<(&'a Project, &'a Box<dyn Device>), String> {
        let project = config
            .get_project_by_name(project_name)
            .ok_or_else(|| format!("Project not found: {}", project_name))?;

        let device = config
            .get_device_by_name(device_name)
            .ok_or_else(|| format!("Device not found: {}", device_name))?;

        device.test_availability().map_err(|e| {
            format!(
                "Device not available at location {}: {}",
                device.get_location(),
                e
            )
        })?;
        project.test_availability().map_err(|e| {
            format!(
                "Project not available at location {}: {}",
                project.get_location(),
                e
            )
        })?;

        Ok((project, device))
    }

    fn get_index_file(project: &Project, device: &Box<dyn Device>) -> Result<BackupIndex, String> {
        device
            .read_backup_index(project.get_name())?
            .map_or(Ok(BackupIndex::new()), |reader| {
                BackupIndex::from_index_reader(reader)
            })
            .map_err(|e| format!("Backup index read failed: {}", e))
    }
}

impl BackupOperations for Operations {
    fn backup_project_to_device(
        &self,
        project_name: &str,
        device_name: &str,
    ) -> Result<(), String> {
        let config = &GlobalConfig::load(
            self.global_config_provider.as_ref(),
            &self.device_factory_registry,
        )?;
        let (project, device) = self.get_project_and_device(&config, project_name, device_name)?;
        let index = Operations::get_index_file(project, device)?;

        let project_root_path = PathBuf::from(project.get_location());
        let archive_writer = device.get_archive_writer(&project.get_name());

        BackupExecution::new(index, project_root_path)
            .execute(archive_writer)
            .map_err(|e| format!("Backup failed: {}", e))
    }

    fn restore_project_from_device(
        &self,
        project_name: &str,
        device_name: &str,
        to: &str,
    ) -> Result<(), String> {
        let config = &GlobalConfig::load(
            self.global_config_provider.as_ref(),
            &self.device_factory_registry,
        )?;
        let (project, device) = self.get_project_and_device(&config, project_name, device_name)?;
        let index = Operations::get_index_file(project, device)?;

        let restoration_path = PathBuf::from(to);
        let extractor = device.get_extractor(project_name);

        RestoreExecution::new(index, restoration_path, extractor).extract()
    }
}

#[cfg(test)]
mod test {}
