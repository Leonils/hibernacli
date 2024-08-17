use flate2::write::GzEncoder;
use itertools::Itertools;

use crate::{
    core::{
        util::timestamps::Timestamp, ArchiveError, ArchiveWriter, Device, DeviceFactory,
        DifferentialArchiveStep, Extractor, Question, QuestionType, SecurityLevel,
    },
    now,
};
use std::{
    fs::File,
    io::{self, BufRead, Cursor, Read},
    path::{Path, PathBuf},
    time::{Instant, SystemTime},
};

struct MountedFolder {
    name: Option<String>,
    path: PathBuf,
}

impl MountedFolder {
    fn get_project_path(&self, project_name: &str) -> PathBuf {
        Path::join(&self.path, &project_name)
    }
}

impl Device for MountedFolder {
    fn get_name(&self) -> String {
        if let Some(name) = &self.name {
            return name.clone();
        }
        format!("MountedFolder[{}]", self.path.display())
    }

    fn get_location(&self) -> String {
        self.path.display().to_string()
    }

    fn get_security_level(&self) -> SecurityLevel {
        SecurityLevel::Local
    }

    fn get_device_type_name(&self) -> String {
        "MountedFolder".to_string()
    }

    fn get_last_connection(&self) -> Option<Instant> {
        None
    }

    fn get_last_disconnection(&self) -> Option<Instant> {
        None
    }

    fn to_toml_table(&self) -> toml::value::Table {
        let mut table = toml::value::Table::new();
        table.insert("type".to_string(), self.get_device_type_name().into());
        table.insert("path".to_string(), self.path.display().to_string().into());
        table.insert("name".to_string(), self.get_name().into());
        table
    }

    fn read_backup_index(&self, project_name: &str) -> Result<Option<Box<dyn BufRead>>, String> {
        let index_path = Path::join(&self.get_project_path(project_name), "current.index");

        match std::fs::read(&index_path) {
            Ok(data) => Ok(Some(Box::new(Cursor::new(data)))),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Ok(None),
                _ => Err(e.to_string()),
            },
        }
    }

    fn test_availability(&self) -> Result<(), String> {
        self.path.read_dir().map(|_| ()).map_err(|e| e.to_string())
    }

    fn get_archive_writer(&self, project_name: &str) -> Box<dyn ArchiveWriter> {
        let now = now!().ms_since_epoch().unwrap();
        let project_dir = Path::join(&self.path, &project_name);
        let archive_path = Path::join(&project_dir, format!("{}.tar", now));

        Box::new(MountedFolderArchiveWriter::new(
            self.path.clone(),
            project_dir,
            archive_path,
        ))
    }

    fn get_extractor(&self, project_name: &str) -> Box<dyn Extractor> {
        let project_dir = Path::join(&self.path, &project_name);

        Box::new(MountedFolderExtractor::new(self.path.clone(), project_dir))
    }
}

pub struct MountedFolderArchiveWriter {
    path: PathBuf,
    project_dir: PathBuf,
    archive_path: PathBuf,
    tar_builder: Option<tar::Builder<std::fs::File>>,
    finalized: bool,
}

impl MountedFolderArchiveWriter {
    pub fn new(
        path: PathBuf,
        project_dir: PathBuf,
        archive_path: PathBuf,
    ) -> MountedFolderArchiveWriter {
        MountedFolderArchiveWriter {
            path,
            project_dir,
            archive_path,
            tar_builder: None,
            finalized: false,
        }
    }

    fn try_get_tar_builder<'a>(
        &'a mut self,
    ) -> Result<&'a mut tar::Builder<std::fs::File>, ArchiveError> {
        return self
            .tar_builder
            .as_mut()
            .ok_or(ArchiveError::from("Tar builder is missing"));
    }

    fn initialize<'a>(&'a mut self) -> Result<&'a mut tar::Builder<std::fs::File>, ArchiveError> {
        if self.finalized {
            return Err(ArchiveError::from("Archive has already been finalized"));
        }

        if self.tar_builder.is_some() {
            return self
                .tar_builder
                .as_mut()
                .ok_or(ArchiveError::from("Tar builder is missing"));
        }

        // create dir if missing
        if !self.project_dir.exists() {
            std::fs::create_dir_all(&self.project_dir).map_err(|_| {
                "Project directory is missing on secondary device and failed to be created"
            })?;
        }

        // Verify that the archive file does not exist
        if self.archive_path.exists() {
            return Err(ArchiveError::from("Archive file already exists"));
        }

        // create archive file
        std::fs::File::create(&self.archive_path)?;

        // create tar builder
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(&self.archive_path)?;

        self.tar_builder = Some(tar::Builder::new(file));
        return self.try_get_tar_builder();
    }

    fn add_file_from_bytes(&mut self, data: &[u8], path: &Path) -> Result<(), ArchiveError> {
        // Prepare headers
        let mut header = tar::Header::new_gnu();
        header.set_path(path)?;
        header.set_size(data.len() as u64);
        header.set_mtime(now!().s_since_epoch()?);
        header.set_mode(0o644);
        header.set_cksum();

        // Open archive and add file
        self.initialize()?.append(&header, data)?;
        Ok(())
    }
}

impl ArchiveWriter for MountedFolderArchiveWriter {
    fn add_file(
        &mut self,
        file: &mut File,
        path: &PathBuf,
        _ctime: u128,
        _mtime: u128,
        _size: u64,
    ) -> Result<(), ArchiveError> {
        self.initialize()?.append_file(
            Path::join(Path::new(".files"), path.file_name().unwrap()),
            file,
        )?;
        println!("Adding file {:?} to {:?} secondary device", path, self.path);
        Ok(())
    }

    fn add_directory(
        &mut self,
        path: &PathBuf,
        _ctime: u128,
        _mtime: u128,
    ) -> Result<(), ArchiveError> {
        println!(
            "Adding directory {:?} to {:?} secondary device",
            path, self.path
        );
        Ok(())
    }

    fn add_symlink(
        &mut self,
        path: &PathBuf,
        _ctime: u128,
        _mtime: u128,
        _target: &PathBuf,
    ) -> Result<(), ArchiveError> {
        println!(
            "Adding symlink {:?} to {:?} secondary device",
            path, self.path
        );
        Ok(())
    }

    fn finalize(
        &mut self,
        deleted_files: &Vec<PathBuf>,
        new_index: &Vec<u8>,
    ) -> Result<(), ArchiveError> {
        println!("Finalizing archive to {:?}", self.archive_path);

        // Create a file with the list of deleted files
        let deleted_files_data = deleted_files
            .iter()
            .map(|path| path.to_string_lossy())
            .collect::<Vec<_>>()
            .join("\n");
        let deleted_files_data = deleted_files_data.as_bytes();
        self.add_file_from_bytes(deleted_files_data, Path::new(".deleted-files"))?;

        // Add a copy of the new index in the archive
        self.add_file_from_bytes(&new_index, Path::new(".index"))?;

        // Save the index for quick access to the latest version
        let current_index_path = Path::join(&self.project_dir, "current.index");
        std::fs::write(&current_index_path, new_index)?;

        // End the archive
        self.try_get_tar_builder()?.finish()?;

        // Open the archive and a gzip file to compress it (just add .gz to the file name)
        let tar_file = File::open(&self.archive_path)?;
        let gz_file = File::create(&format!("{}.gz", self.archive_path.display()))?;

        // Compress the archive
        let tar_file_size = tar_file.metadata()?.len();
        let mut encoder = GzEncoder::new(gz_file, flate2::Compression::default());
        io::copy(&mut tar_file.take(tar_file_size), &mut encoder)?;
        encoder.finish()?;

        // Remove the uncompressed archive
        std::fs::remove_file(&self.archive_path)?;
        self.finalized = true;
        Ok(())
    }
}

pub struct MountedFolderExtractor {
    archive_paths: Vec<PathBuf>,
    index_from_start: usize,
    index_from_end: usize,
}

impl MountedFolderExtractor {
    pub fn new(_path: PathBuf, backup_path: PathBuf) -> MountedFolderExtractor {
        let archive_paths: Vec<PathBuf> = backup_path
            .read_dir()
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.file_name()
                    .and_then(|s| s.to_str())
                    .map_or(false, |s| {
                        s.ends_with(".tar.gz") && s[..s.len() - 7].chars().all(char::is_numeric)
                    })
            })
            .sorted()
            .collect();

        let index_from_end = archive_paths.len();
        MountedFolderExtractor {
            archive_paths,
            index_from_start: 0,
            index_from_end,
        }
    }
}

impl Iterator for MountedFolderExtractor {
    type Item = Box<dyn DifferentialArchiveStep>;

    fn next(&mut self) -> Option<Box<dyn DifferentialArchiveStep>> {
        if self.index_from_start >= self.index_from_end {
            return None;
        }

        let archive_path = &self.archive_paths[self.index_from_start];
        self.index_from_start += 1;

        let archive_name = archive_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        Some(Box::new(MountedFolderDifferentialArchiveStep {
            archive_name,
        }))
    }
}

impl DoubleEndedIterator for MountedFolderExtractor {
    fn next_back(&mut self) -> Option<Box<dyn DifferentialArchiveStep>> {
        if self.index_from_end <= self.index_from_start {
            return None;
        }

        self.index_from_end -= 1;
        let archive_path = &self.archive_paths[self.index_from_end];

        let archive_name = archive_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        Some(Box::new(MountedFolderDifferentialArchiveStep {
            archive_name,
        }))
    }
}

impl Extractor for MountedFolderExtractor {}

pub struct MountedFolderDifferentialArchiveStep {
    archive_name: String,
}

impl DifferentialArchiveStep for MountedFolderDifferentialArchiveStep {
    fn get_step_name(&self) -> &str {
        &self.archive_name
    }
}

pub struct MountedFolderFactory {
    path_question: Question,
    name_question: Question,
    step: u8,
}

impl MountedFolderFactory {
    pub fn new() -> MountedFolderFactory {
        MountedFolderFactory {
            path_question: Question::new(
                "What is the path of the folder?".to_string(),
                QuestionType::UnixPath,
            ),
            name_question: Question::new(
                "How would you name this device?".to_string(),
                QuestionType::String,
            ),
            step: 0,
        }
    }

    fn get_current_question(&self) -> &Question {
        match self.step {
            0 => &self.path_question,
            1 => &self.name_question,
            _ => panic!("No more questions"),
        }
    }
}

impl DeviceFactory for MountedFolderFactory {
    fn get_question_statement(&self) -> &str {
        self.get_current_question().get_statement()
    }

    fn get_question_type(&self) -> &QuestionType {
        self.get_current_question().get_question_type()
    }

    fn set_question_answer(&mut self, answer: String) -> Result<(), String> {
        let status = match self.step {
            0 => self.path_question.set_answer(answer),
            1 => self.name_question.set_answer(answer),
            _ => panic!("No more questions"),
        };

        status?;
        self.step += 1;
        Ok(())
    }

    fn has_next(&self) -> bool {
        self.step < 2
    }

    fn build(&self) -> Result<Box<dyn Device>, String> {
        if self.step < 2 {
            return Err("Not all questions have been answered".to_string());
        }

        let path = self.path_question.get_answer()?;
        let name = self.name_question.get_answer()?;
        let name = if name.is_empty() { None } else { Some(name) };
        Ok(Box::new(MountedFolder {
            name,
            path: PathBuf::from(path),
        }))
    }

    fn build_from_toml_table(
        &self,
        name: &str,
        table: &toml::value::Table,
    ) -> Result<Box<dyn Device>, String> {
        let path = table
            .get("path")
            .ok_or_else(|| "missing field `path`".to_string())?
            .as_str()
            .ok_or_else(|| "Invalid string for 'path'".to_string())?;

        Ok(Box::new(MountedFolder {
            name: Some(name.to_string()),
            path: PathBuf::from(path),
        }))
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use crate::core::test_utils::fs::create_tmp_dir;

    use super::*;

    #[test]
    fn i_should_be_able_to_get_first_questions_with_its_type() {
        let factory = MountedFolderFactory::new();

        assert!(factory.has_next());
        assert_eq!(
            factory.get_question_statement(),
            "What is the path of the folder?"
        );
        assert_eq!(factory.get_question_type(), &QuestionType::UnixPath);
    }

    #[test]
    fn after_answering_q1_i_should_be_able_to_get_second_questions_with_its_type() {
        let mut factory = MountedFolderFactory::new();

        factory.set_question_answer("/".to_string()).unwrap();
        assert!(factory.has_next());
        assert_eq!(
            factory.get_question_statement(),
            "How would you name this device?"
        );
        assert_eq!(factory.get_question_type(), &QuestionType::String);
    }

    #[test]
    #[should_panic]
    fn when_getting_too_many_questions_it_shall_panic() {
        let mut factory = MountedFolderFactory::new();
        factory.set_question_answer("/".to_string()).unwrap();
        factory.set_question_answer("MyFolder".to_string()).unwrap();
        factory.get_question_statement();
    }

    #[test]
    #[should_panic]
    fn when_getting_too_many_questions_type_it_shall_panic() {
        let mut factory = MountedFolderFactory::new();
        factory.set_question_answer("/".to_string()).unwrap();
        factory.set_question_answer("MyFolder".to_string()).unwrap();
        factory.get_question_type();
    }

    #[test]
    #[should_panic]
    fn when_answering_too_many_questions_it_shall_panic() {
        let mut factory = MountedFolderFactory::new();
        factory.set_question_answer("/".to_string()).unwrap();
        factory.set_question_answer("MyFolder".to_string()).unwrap();
        factory.set_question_answer("Random".to_string()).unwrap();
    }

    #[test]
    fn when_answering_questions_device_is_built() {
        let mut factory = MountedFolderFactory::new();

        factory
            .set_question_answer("/media/user/0000-0000".to_string())
            .unwrap();
        factory.set_question_answer("MyUsbKey".to_string()).unwrap();

        let device = factory.build().unwrap();
        assert_eq!(device.get_name(), "MyUsbKey");
        assert_eq!(device.get_location(), "/media/user/0000-0000");
    }

    #[test]
    fn when_answering_questions_but_not_name_default_name_is_used() {
        let mut factory = MountedFolderFactory::new();

        factory
            .set_question_answer("/media/user/0000-0000".to_string())
            .unwrap();
        factory.set_question_answer("".to_string()).unwrap();

        let device = factory.build().unwrap();
        assert_eq!(device.get_name(), "MountedFolder[/media/user/0000-0000]");
        assert_eq!(device.get_location(), "/media/user/0000-0000");
    }

    #[test]
    fn when_not_all_questions_have_been_answered_error_is_returned() {
        let factory = MountedFolderFactory::new();
        let device = factory.build();
        assert_eq!(
            "Not all questions have been answered",
            device.err().unwrap()
        );
    }

    #[test]
    fn when_creating_device_from_toml_table_it_shall_have_the_right_name_and_location() {
        let factory = MountedFolderFactory::new();
        let mut table = toml::value::Table::new();
        table.insert(
            "path".to_string(),
            toml::Value::String("/media/user/0000-0000".to_string()),
        );
        table.insert(
            "name".to_string(),
            toml::Value::String("MyUsbKey".to_string()),
        );

        let device = factory.build_from_toml_table("MyUsbKey", &table).unwrap();
        assert_eq!(device.get_name(), "MyUsbKey");
        assert_eq!(device.get_location(), "/media/user/0000-0000");
    }

    #[test]
    fn when_creating_device_from_toml_with_no_path_it_shall_return_error() {
        let factory = MountedFolderFactory::new();
        let table = toml::value::Table::new();

        let device = factory.build_from_toml_table("MyUsbKey", &table);
        assert_eq!("missing field `path`", device.err().unwrap());
    }

    #[test]
    fn when_creating_device_from_toml_with_invalid_path_it_shall_return_error() {
        let factory = MountedFolderFactory::new();
        let mut table = toml::value::Table::new();
        table.insert("path".to_string(), toml::Value::Integer(42));

        let device = factory.build_from_toml_table("MyUsbKey", &table);
        assert_eq!("Invalid string for 'path'", device.err().unwrap());
    }

    #[test]
    fn when_serializing_device_to_toml_with_name_it_shall_have_name_path_and_type() {
        let device = MountedFolder {
            name: Some("MyUsbKey".to_string()),
            path: PathBuf::from("/media/user/0000-0000"),
        };

        let table = device.to_toml_table();
        let string_table = toml::to_string(&table).unwrap();
        assert_eq!(
            string_table,
            r#"name = "MyUsbKey"
path = "/media/user/0000-0000"
type = "MountedFolder"
"#
        );
    }

    #[test]
    fn when_getting_archive_writer_and_adding_no_file_it_shall_create_empty_archive() {
        let tmp_device = create_tmp_dir();
        let tmp_device_path = tmp_device.clone();
        let device = MountedFolder {
            name: Some("MyUsbKey".to_string()),
            path: tmp_device,
        };

        let mut archive_writer = device.get_archive_writer("MyProject");
        archive_writer.finalize(&vec![], &vec![]).unwrap();

        let project_path = Path::join(&tmp_device_path, "MyProject");
        let index_path = Path::join(&project_path, "current.index");
        let tar_path = Path::join(&project_path, "0.tar.gz");

        // Only 2 files: the index and the tar
        let files = std::fs::read_dir(&project_path).unwrap();
        assert_eq!(2, files.count());
        assert!(tar_path.exists());
        assert!(index_path.exists());
    }

    #[test]
    fn finalizing_two_times_the_same_archive_shall_fail() {
        let tmp_device = create_tmp_dir();
        let device = MountedFolder {
            name: Some("MyUsbKey".to_string()),
            path: tmp_device,
        };

        let mut archive_writer = device.get_archive_writer("MyProject");
        archive_writer.finalize(&vec![], &vec![]).unwrap();
        let result = archive_writer.finalize(&vec![], &vec![]).unwrap_err();
        assert_eq!("Archive has already been finalized", result.message);
    }

    #[test]
    fn when_archiving_the_device_location_shall_exist() {
        let device = MountedFolder {
            name: Some("MyUsbKey".to_string()),
            path: PathBuf::from("/media/user/0000-0000/not-found-device"),
        };

        let mut archive_writer = device.get_archive_writer("MyProject");
        let result = archive_writer.finalize(&vec![], &vec![]).unwrap_err();
        assert_eq!(
            "Project directory is missing on secondary device and failed to be created",
            result.message
        );
    }

    #[test]
    fn test_availability_shall_return_error_if_path_is_missing() {
        let device = MountedFolder {
            name: Some("MyUsbKey".to_string()),
            path: PathBuf::from("/media/user/0000-0000/not-found-device"),
        };

        let result = device.test_availability().unwrap_err();
        assert_eq!("No such file or directory (os error 2)", result.to_string());
    }

    #[test]
    fn test_availability_shall_return_ok_if_dir_exists() {
        let device_path = create_tmp_dir();
        let device = MountedFolder {
            name: Some("MyUsbKey".to_string()),
            path: device_path,
        };

        device.test_availability().unwrap();
    }

    #[test]
    fn when_adding_a_file_to_archive_it_shall_pass() {
        let tmp_device = create_tmp_dir();
        let tmp_project = create_tmp_dir();
        let device = MountedFolder {
            name: Some("MyUsbKey".to_string()),
            path: tmp_device,
        };

        fs::write(Path::join(&tmp_project, "file.txt"), "Hello, world!").unwrap();
        let mut file = fs::File::open(Path::join(&tmp_project, "file.txt")).unwrap();

        let mut archive_writer = device.get_archive_writer("MyProject");
        archive_writer
            .add_file(&mut file, &PathBuf::from("file.txt"), 0, 0, 13)
            .unwrap();
        archive_writer.finalize(&vec![], &vec![]).unwrap();
    }

    #[test]
    fn adding_file_after_finalizing_archive_shall_fail() {
        let tmp_device = create_tmp_dir();
        let tmp_project = create_tmp_dir();
        let device = MountedFolder {
            name: Some("MyUsbKey".to_string()),
            path: tmp_device,
        };

        fs::write(Path::join(&tmp_project, "file.txt"), "Hello, world!").unwrap();
        let mut file = fs::File::open(Path::join(&tmp_project, "file.txt")).unwrap();

        let mut archive_writer = device.get_archive_writer("MyProject");
        archive_writer.finalize(&vec![], &vec![]).unwrap();
        let result = archive_writer
            .add_file(&mut file, &PathBuf::from("file.txt"), 0, 0, 13)
            .unwrap_err();

        assert_eq!("Archive has already been finalized", result.message);
    }
}
