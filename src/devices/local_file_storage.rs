use crate::adapters::primary_device::GlobalConfigProvider;
use directories::ProjectDirs;
use std::path::{Path, PathBuf};

struct LocalFileStorage<'a> {
    config_dir: Box<Path>,
    file_system: &'a dyn FileSystem,
    default_config: &'a str,
}

impl<'a> LocalFileStorage<'a> {
    pub fn new(
        path_provider: &impl PathProvider,
        file_system: &'a dyn FileSystem,
        default_config: &'a str,
    ) -> LocalFileStorage<'a> {
        let config_dir = path_provider
            .get_config_dir("hibernacli")
            .or_else(|| panic!("Could not get the config directory."))
            .unwrap();
        LocalFileStorage {
            config_dir,
            file_system,
            default_config,
        }
    }
}

impl<'a> GlobalConfigProvider for LocalFileStorage<'a> {
    fn init_global_config_dir(&self) -> Result<(), String> {
        if !self.config_dir.exists() {
            self.file_system
                .create_dir_all(self.config_dir.to_owned().into_path_buf())?;
        }

        let config_path = self.config_dir.join("config.toml");
        if !config_path.exists() {
            self.file_system
                .write_file(config_path, &self.default_config)?;
        }

        Ok(())
    }

    fn read_global_config_dir(&self) -> Result<String, String> {
        self.init_global_config_dir().map_err(|_| {
            "Config file was missing and an error occurred while creating it.".to_string()
        })?;

        self.file_system
            .read_file(self.config_dir.join("config.toml"))
    }
}

trait PathProvider {
    fn get_config_dir(&self, project_name: &str) -> Option<Box<Path>>;
}

struct StandardPathProvider;
impl PathProvider for StandardPathProvider {
    fn get_config_dir(&self, project_name: &str) -> Option<Box<Path>> {
        let project_dir = ProjectDirs::from("", "", project_name)?;
        Some(project_dir.config_dir().to_path_buf().into_boxed_path())
    }
}

trait FileSystem {
    fn write_file(&self, file_path: PathBuf, content: &str) -> Result<(), String>;
    fn read_file(&self, _file_path: PathBuf) -> Result<String, String>;
    fn create_dir_all(&self, dir_path: PathBuf) -> Result<(), String>;
}

struct StandardFileSystem;
impl FileSystem for StandardFileSystem {
    fn write_file(&self, file_path: PathBuf, content: &str) -> Result<(), String> {
        std::fs::write(file_path, content).map_err(|e| e.to_string())
    }
    fn read_file(&self, file_path: PathBuf) -> Result<String, String> {
        std::fs::read_to_string(file_path).map_err(|e| e.to_string())
    }
    fn create_dir_all(&self, dir_path: PathBuf) -> Result<(), String> {
        std::fs::create_dir_all(dir_path).map_err(|e| e.to_string())
    }
}

// tests only run on linux systems
#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::{
        adapters::primary_device::GlobalConfigProvider,
        devices::local_file_storage::{LocalFileStorage, StandardFileSystem},
    };

    use super::PathProvider;

    struct TmpLinuxPathProvider {
        tmp_path: PathBuf,
    }
    impl TmpLinuxPathProvider {
        fn new() -> TmpLinuxPathProvider {
            let random_project_name = format!("hibernacli-tests-{}", uuid::Uuid::new_v4());
            let tmp_path = std::env::temp_dir().join(random_project_name);
            std::fs::create_dir_all(&tmp_path).unwrap();
            TmpLinuxPathProvider { tmp_path }
        }
        fn get_tmp_path(&self) -> &PathBuf {
            &self.tmp_path
        }
    }
    impl PathProvider for TmpLinuxPathProvider {
        fn get_config_dir(&self, project_name: &str) -> Option<Box<Path>> {
            Some(self.tmp_path.clone().join(project_name).into_boxed_path())
        }
    }

    #[test]
    fn when_there_is_already_a_config_it_should_skip_creation() {
        // arrange
        let mock_path_provider = TmpLinuxPathProvider::new();
        let file_system = StandardFileSystem {};
        let config_dir = mock_path_provider.get_tmp_path().join("hibernacli");
        std::fs::create_dir_all(&config_dir).unwrap();
        let config_path = config_dir.join("config.toml");
        std::fs::write(&config_path, "previous-content").unwrap();

        // act
        let local_unix_file_storage =
            LocalFileStorage::new(&mock_path_provider, &file_system, "test");
        let res = local_unix_file_storage.init_global_config_dir();

        // assert
        assert_eq!(res, Ok(()));
        assert_eq!(config_path.exists(), true);
        assert_eq!(
            std::fs::read_to_string(config_path).unwrap(),
            "previous-content"
        );
    }

    #[test]
    fn when_the_folder_already_exists_but_not_the_file_it_shall_create_the_file() {
        // arrange
        let mock_path_provider = TmpLinuxPathProvider::new();
        let file_system = StandardFileSystem {};
        let config_dir = mock_path_provider.get_tmp_path().join("hibernacli");
        std::fs::create_dir_all(&config_dir).unwrap();

        // act
        let local_unix_file_storage =
            LocalFileStorage::new(&mock_path_provider, &file_system, "test");
        let res = local_unix_file_storage.init_global_config_dir();

        // assert
        let config_path = config_dir.join("config.toml");
        assert_eq!(res, Ok(()));
        assert_eq!(config_path.exists(), true);
        assert_eq!(std::fs::read_to_string(config_path).unwrap(), "test");
    }

    #[test]
    fn when_there_is_no_config_at_all_it_should_create_a_new_one() {
        // arrange
        let mock_path_provider = TmpLinuxPathProvider::new();
        let file_system = StandardFileSystem {};

        // act
        let local_unix_file_storage =
            LocalFileStorage::new(&mock_path_provider, &file_system, "test");
        let res = local_unix_file_storage.init_global_config_dir();

        // assert
        let config_path = mock_path_provider
            .get_tmp_path()
            .join("hibernacli")
            .join("config.toml");
        assert_eq!(res, Ok(()));
        assert_eq!(config_path.exists(), true);
        assert_eq!(std::fs::read_to_string(config_path).unwrap(), "test");
    }

    struct FailingPathProvider;
    impl PathProvider for FailingPathProvider {
        fn get_config_dir(&self, _project_name: &str) -> Option<Box<Path>> {
            None
        }
    }

    #[test]
    #[should_panic]
    fn when_the_config_dir_cannot_be_retrieved_it_should_panic() {
        let failing_path_provider = FailingPathProvider;
        let file_system = StandardFileSystem;
        LocalFileStorage::new(&failing_path_provider, &file_system, "test");
    }

    struct FailingWriteFileSystem;
    impl super::FileSystem for FailingWriteFileSystem {
        fn read_file(&self, _file_path: PathBuf) -> Result<String, String> {
            Ok("".to_string())
        }
        fn write_file(&self, _file_path: PathBuf, _content: &str) -> Result<(), String> {
            Err("Could not write file".to_string())
        }
        fn create_dir_all(&self, _dir_path: PathBuf) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn when_the_file_cannot_be_written_it_should_return_an_error() {
        // arrange
        let mock_path_provider = TmpLinuxPathProvider::new();
        let file_system = FailingWriteFileSystem;

        // act
        let local_unix_file_storage =
            LocalFileStorage::new(&mock_path_provider, &file_system, "test");
        let res = local_unix_file_storage.init_global_config_dir();

        // assert
        assert_eq!(res, Err("Could not write file".to_string()));
    }

    struct FailingCreateDirFileSystem;
    impl super::FileSystem for FailingCreateDirFileSystem {
        fn read_file(&self, _file_path: PathBuf) -> Result<String, String> {
            Ok("".to_string())
        }
        fn write_file(&self, _file_path: PathBuf, _content: &str) -> Result<(), String> {
            Ok(())
        }
        fn create_dir_all(&self, _dir_path: PathBuf) -> Result<(), String> {
            Err("Could not create dir".to_string())
        }
    }

    #[test]
    fn when_the_dir_cannot_be_created_it_should_return_an_error() {
        // arrange
        let mock_path_provider = TmpLinuxPathProvider::new();
        let file_system = FailingCreateDirFileSystem;

        // act
        let local_unix_file_storage =
            LocalFileStorage::new(&mock_path_provider, &file_system, "config");
        let res = local_unix_file_storage.init_global_config_dir();

        // assert
        assert_eq!(res, Err("Could not create dir".to_string()));
    }

    #[test]
    fn when_the_config_file_exists_it_should_read_it() {
        // arrange
        let mock_path_provider = TmpLinuxPathProvider::new();
        let file_system = StandardFileSystem {};
        let config_dir = mock_path_provider.get_tmp_path().join("hibernacli");
        std::fs::create_dir_all(&config_dir).unwrap();
        let config_path = config_dir.join("config.toml");
        std::fs::write(&config_path, "previous-content").unwrap();

        // act
        let local_unix_file_storage =
            LocalFileStorage::new(&mock_path_provider, &file_system, "config");
        let res = local_unix_file_storage.read_global_config_dir();

        // assert
        assert_eq!(res, Ok("previous-content".to_string()));
    }

    #[test]
    fn when_the_config_file_does_not_exist_it_should_create_it_with_default_content() {
        // arrange
        let mock_path_provider = TmpLinuxPathProvider::new();
        let file_system = StandardFileSystem {};

        // act
        let local_unix_file_storage =
            LocalFileStorage::new(&mock_path_provider, &file_system, "config");
        let res = local_unix_file_storage.read_global_config_dir();

        // assert
        assert_eq!(res, Ok("config".to_string()));
    }

    #[test]
    fn when_the_config_file_is_missing_and_fail_to_be_created_error_shall_be_explained() {
        // arrange
        let mock_path_provider = TmpLinuxPathProvider::new();
        let file_system = FailingWriteFileSystem;

        // act
        let local_unix_file_storage =
            LocalFileStorage::new(&mock_path_provider, &file_system, "config");
        let res = local_unix_file_storage.read_global_config_dir();

        // assert
        assert_eq!(
            res,
            Err("Config file was missing and an error occurred while creating it.".to_string())
        );
    }
}
