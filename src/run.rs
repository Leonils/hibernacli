use std::rc::Rc;

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
    let device_factory = Rc::new(MountedFolderFactory::new());
    operations.register_device_factory(
        "mounted_folder".to_string(),
        "Mouted device".to_string(),
        device_factory,
    );
    let mut command_runner = CommandRunner::new(Console, &operations, &operations);
    command_runner.run(args);
}
