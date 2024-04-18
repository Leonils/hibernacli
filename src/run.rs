use crate::{
    cli::{CommandRunner, Console},
    core::operations::Operations,
    devices::{
        local_file_storage::{LocalFileStorage, StandardFileSystem, StandardPathProvider},
        mounted_folder::MountedFolderFactory,
    },
};
const DEFAULT_CONFIG: &str = "";

pub fn run(args: Vec<String>) {
    let standard_path_provider = StandardPathProvider {};
    let local_file_storage = LocalFileStorage::new(
        &standard_path_provider,
        &StandardFileSystem {},
        DEFAULT_CONFIG,
    );
    let mut operations = Operations::new(Box::new(local_file_storage));
    operations.register_device_factory(
        "MountedFolder".to_string(),
        "Mounted device".to_string(),
        || Box::new(MountedFolderFactory::new()),
    );

    let command_runner = CommandRunner::new(Console, &operations, &operations);
    command_runner.run(args);
}
