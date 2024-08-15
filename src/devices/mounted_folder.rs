use flate2::write::GzEncoder;

use crate::{
    models::{
        backup_requirement::SecurityLevel,
        question::{Question, QuestionType},
        secondary_device::{ArchiveWriter, Device, DeviceFactory},
    },
    now,
};
use std::{
    fs::File,
    io::{self, BufRead, Cursor, Read},
    path::PathBuf,
    time::Instant,
};
use std::{path::Path, time::SystemTime};

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
        Box::new(MountedFolderArchiveWriter::new(
            self.path.clone(),
            project_name.to_string(),
        ))
    }
}

pub struct MountedFolderArchiveWriter {
    path: PathBuf,
    project_dir: PathBuf,
    archive_path: PathBuf,
    tar_builder: Option<tar::Builder<std::fs::File>>,
}

impl MountedFolderArchiveWriter {
    pub fn new(path: PathBuf, project_name: String) -> MountedFolderArchiveWriter {
        let now = now!()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap() // we are after 1970, this should never fail
            .as_secs();
        let project_dir = Path::join(&path, &project_name);
        let archive_path = Path::join(&project_dir, format!("{}.tar", now));

        MountedFolderArchiveWriter {
            path,
            project_dir,
            archive_path,
            tar_builder: None,
        }
    }

    fn initialize<'a>(&'a mut self) -> &'a mut tar::Builder<std::fs::File> {
        if self.tar_builder.is_some() {
            return self.tar_builder.as_mut().unwrap();
        }

        println!("Initializing archive to {:?}", self.archive_path);

        // create dir if missing
        if !self.project_dir.exists() {
            std::fs::create_dir_all(&self.project_dir).unwrap();
        }

        // Verify that the archive file does not exist
        if self.archive_path.exists() {
            panic!("Archive file already exists");
        }

        // create archive file
        std::fs::File::create(&self.archive_path).unwrap();

        // create tar builder
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(&self.archive_path)
            .unwrap();
        self.tar_builder = Some(tar::Builder::new(file));

        return self.tar_builder.as_mut().unwrap();
    }

    fn add_file_from_bytes(&mut self, data: &[u8], path: &Path) {
        // Add it to archive
        let mut header = tar::Header::new_gnu();
        header.set_path(path).unwrap();
        header.set_size(data.len() as u64);
        header.set_mtime(
            now!()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        header.set_mode(0o644);
        header.set_cksum();
        let archive_writer = self.initialize();

        // Close archive
        archive_writer.append(&header, data).unwrap();
    }
}

impl ArchiveWriter for MountedFolderArchiveWriter {
    fn add_file(&mut self, file: &mut File, path: &PathBuf, _ctime: u64, _mtime: u64, _size: u64) {
        self.initialize()
            .append_file(
                Path::join(Path::new(".files"), path.file_name().unwrap()),
                file,
            )
            .unwrap();
        println!("Adding file {:?} to {:?} secondary device", path, self.path);
    }

    fn add_directory(&mut self, path: &PathBuf, _ctime: u64, _mtime: u64) {
        println!(
            "Adding directory {:?} to {:?} secondary device",
            path, self.path
        );
    }

    fn add_symlink(&mut self, path: &PathBuf, _ctime: u64, _mtime: u64, _target: &PathBuf) {
        println!(
            "Adding symlink {:?} to {:?} secondary device",
            path, self.path
        );
    }

    fn finalize(&mut self, deleted_files: &Vec<PathBuf>, new_index: &Vec<u8>) {
        println!("Finalizing archive to {:?}", self.archive_path);

        // Create a file with the list of deleted files
        let deleted_files_data = deleted_files
            .iter()
            .map(|path| path.to_string_lossy())
            .collect::<Vec<_>>()
            .join("\n");
        let deleted_files_data = deleted_files_data.as_bytes();
        self.add_file_from_bytes(deleted_files_data, Path::new(".deleted-files"));

        // Add a copy of the new index in the archive
        self.add_file_from_bytes(&new_index, Path::new(".index"));

        // Save the index for quick access to the latest version
        let current_index_path = Path::join(&self.project_dir, "current.index");
        std::fs::write(&current_index_path, new_index).unwrap();

        // End the archive
        self.tar_builder.as_mut().unwrap().finish().unwrap();

        // Open the archive and a gzip file to compress it (just add .gz to the file name)
        let tar_file = File::open(&self.archive_path).unwrap();
        let mut gz_file = File::create(&format!("{}.gz", self.archive_path.display())).unwrap();

        // Compress the archive
        let tar_file_size = tar_file.metadata().unwrap().len();
        let mut encoder = GzEncoder::new(gz_file, flate2::Compression::default());
        io::copy(&mut tar_file.take(tar_file_size), &mut encoder).unwrap();
        encoder.finish().unwrap();

        // Remove the uncompressed archive
        std::fs::remove_file(&self.archive_path).unwrap();
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
}
