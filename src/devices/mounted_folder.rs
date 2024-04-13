use std::{path::PathBuf, time::Instant};

use crate::models::{
    backup_requirement::SecurityLevel,
    question::{Question, QuestionType},
    secondary_device::{Device, DeviceFactory},
};

struct MountedFolder {
    name: Option<String>,
    path: PathBuf,
}

impl Device for MountedFolder {
    fn get_name(&self) -> String {
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
}

#[derive(Default)]
struct MountedFolderFactory {
    path: Option<PathBuf>,
    name: Option<String>,
    step: u8,
}

impl DeviceFactory for MountedFolderFactory {
    fn get_question(&mut self) -> Question {
        let question = match self.step {
            0 => Question::new(
                "What is the path of the folder?".to_string(),
                QuestionType::UnixPath,
            ),
            1 => Question::new(
                "How would you name this device?".to_string(),
                QuestionType::String,
            ),
            _ => panic!("No more questions"),
        };

        self.step += 1;
        question
    }

    fn has_next(&self) -> bool {
        self.step < 2
    }

    fn build(&self) -> Box<dyn Device> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn when_getting_questions_step_increment_and_question_change() {
        let mut factory = MountedFolderFactory::default();

        assert!(factory.has_next());
        assert_eq!(
            factory.get_question().get_statement(),
            "What is the path of the folder?"
        );
        assert_eq!(factory.step, 1);

        assert!(factory.has_next());
        assert_eq!(
            factory.get_question().get_statement(),
            "How would you name this device?"
        );
        assert_eq!(factory.step, 2);

        assert!(!factory.has_next());
    }

    #[test]
    #[should_panic]
    fn when_answering_too_many_questions_it_shall_panic() {
        let mut factory = MountedFolderFactory::default();

        factory.get_question();
        factory.get_question();
        factory.get_question();
    }
}
