use itertools::Itertools;
use toml::Table;

use crate::{
    core::device_factories_registry::DeviceFactoryRegistry,
    models::{project::Project, secondary_device::Device},
};

use super::toml_try_read::TryRead;

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
struct PartiallyParsedGlobalConfig {
    devices: Option<Vec<Table>>,
    projects: Option<Vec<Table>>,
}

fn from_toml_load_tables_of<T>(
    tables: Option<Vec<Table>>,
    loader: impl Fn(&Table) -> Result<T, String>,
) -> (Vec<String>, Vec<T>) {
    tables
        .unwrap_or(vec![])
        .into_iter()
        .map(|table| loader(&table))
        .into_iter()
        .partition_map(From::from)
}

fn load_device_from_toml_bloc(
    device_table: &Table,
    device_factories_registry: &DeviceFactoryRegistry,
) -> Result<Box<dyn Device>, String> {
    let name: &str = device_table.try_read("name")?;
    let device_type: &str = device_table.try_read("type")?;

    let factory = device_factories_registry
        .get_device_factory(device_type)
        .ok_or_else(|| "Device factory not found".to_string())?;

    let device = factory.build_from_toml_table(&name, &device_table)?;
    Ok(device)
}

fn load_project_from_toml_bloc(project_table: &Table) -> Result<Project, String> {
    let name: &str = project_table.try_read("name")?;
    let path: &str = project_table.try_read("path")?;
    let tracking_status = project_table.try_read("tracking_status")?;

    Ok(Project::new(
        name.to_string(),
        path.to_string(),
        Some(tracking_status),
    ))
}

pub struct ParseTomlResult {
    pub devices: Vec<Box<dyn Device>>,
    pub projects: Vec<Project>,
    pub device_errors: Vec<String>,
    pub project_errors: Vec<String>,
}

pub fn parse_toml_global_config(
    config_toml: &str,
    device_factories_registry: &DeviceFactoryRegistry,
) -> Result<ParseTomlResult, String> {
    let parsed_config =
        toml::from_str::<PartiallyParsedGlobalConfig>(config_toml).map_err(|e| e.to_string())?;

    let (device_errors, devices) = from_toml_load_tables_of(
        parsed_config.devices,
        |device_table| -> Result<Box<dyn Device>, String> {
            load_device_from_toml_bloc(&device_table, &device_factories_registry)
        },
    );

    let (project_errors, projects) =
        from_toml_load_tables_of(parsed_config.projects, load_project_from_toml_bloc);

    Ok(ParseTomlResult {
        devices,
        projects,
        device_errors,
        project_errors,
    })
}
