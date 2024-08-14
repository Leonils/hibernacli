use crate::{
    models::{
        backup_requirement::SecurityLevel,
        question::{Question, QuestionType},
        secondary_device::{ArchiveWriter, Device, DeviceFactory},
    },
    now,
};
use std::{io::BufRead, path::PathBuf, time::Instant};
use std::{path::Path, time::SystemTime};

struct MountedFolder {
    name: Option<String>,
    path: PathBuf,
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
        Ok(None)
    }

    fn test_availability(&self) -> Result<(), String> {
        self.path.read_dir().map(|_| ()).map_err(|e| e.to_string())
    }

    fn get_archive_writer(&self) -> Box<dyn ArchiveWriter> {
        Box::new(MountedFolderArchiveWriter::new(
            self.path.clone(),
            self.get_name(),
        ))
    }
}

pub struct MountedFolderArchiveWriter {
    path: PathBuf,
    initialized: bool,
    project_dir: PathBuf,
    archive_path: PathBuf,
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
            initialized: false,
            project_dir,
            archive_path,
        }
    }

    fn initialize(&mut self) {
        if self.initialized {
            return;
        }

        println!("Initializing archive to {:?}", self.archive_path);

        // create dir if missing
        if !self.project_dir.exists() {
            std::fs::create_dir_all(&self.project_dir).unwrap();
        }

        // create archive file
        std::fs::File::create(&self.archive_path).unwrap();

        self.initialized = true;
    }
}

impl ArchiveWriter for MountedFolderArchiveWriter {
    fn add_file(&mut self, path: &PathBuf, _ctime: u64, _mtime: u64, _size: u64) {
        self.initialize();
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

    fn finalize(&mut self, _deleted_files: Vec<PathBuf>) {
        println!("Finalizing archive to {:?} secondary device", self.path);
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
