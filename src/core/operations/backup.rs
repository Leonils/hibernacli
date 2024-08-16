use std::path::PathBuf;

use crate::{
    adapters::operations::backup::BackupOperations,
    core::{
        backup::{backup_execution::BackupExecution, backup_index::BackupIndex},
        global_config::GlobalConfig,
    },
};

use super::Operations;

impl BackupOperations for Operations {
    fn backup_project_to_device(
        &self,
        project_name: &str,
        device_name: &str,
    ) -> Result<(), String> {
        let config = GlobalConfig::load(
            self.global_config_provider.as_ref(),
            &self.device_factory_registry,
        )?;

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

        let index = device
            .read_backup_index(project.get_name())?
            .map_or(Ok(BackupIndex::new()), |reader| {
                BackupIndex::from_index_reader(reader)
            })
            .map_err(|e| format!("Backup index read failed: {}", e))?;

        let project_root_path = PathBuf::from(project.get_location());
        let mut backup_execution = BackupExecution::new(index, project_root_path);
        let archive_writer = device.get_archive_writer(&project.get_name());

        backup_execution
            .execute(archive_writer)
            .map_err(|e| format!("Backup failed: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod test {}
