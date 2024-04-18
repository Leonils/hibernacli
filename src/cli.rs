#[cfg(test)]
use mockall::automock;

use crate::adapters::operations::device::DeviceOperations;
use crate::core::operations::Operations;
use crate::devices::local_file_storage::{
    LocalFileStorage, StandardFileSystem, StandardPathProvider,
};
use crate::devices::mounted_folder::MountedFolderFactory;
use crate::models::question::QuestionType;
use crate::models::secondary_device::DeviceFactoryKey;
use std::rc::Rc;

const HELP: &str = r#"
HibernaCLI
Usage:
    hibernacli [command] [options]

Commands:
    help              Display this help message
    --version or -v   Display the version of the application
    device [opt]      Manage devices
    config [opt]      Manage configuration
"#;

const INVALID_COMMAND: &str = "Invalid command, use 'help' to display available commands";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_CONFIG: &str = "";
const NEW_DEVICE_INIT: &str = "Create a new device";

#[cfg_attr(test, automock)]
trait UserInterface {
    fn write(&self, message: &str) -> Result<(), String>;
    fn read(&self) -> Result<String, String>;
}

struct Console;

impl UserInterface for Console {
    fn write(&self, message: &str) -> Result<(), String> {
        println!("{}", message);
        Ok(())
    }
    fn read(&self) -> Result<String, String> {
        let mut buffer = String::new();
        match std::io::stdin().read_line(&mut buffer) {
            Ok(_) => return Ok(buffer),
            Err(e) => return Err(e.to_string()),
        };
    }
}

struct CommandRunner<T: UserInterface, U: DeviceOperations> {
    console: T,
    operations: U,
}

impl<T: UserInterface, U: DeviceOperations> CommandRunner<T, U> {
    fn run(&mut self, args: Vec<String>) {
        if args.len() < 2 {
            self.display_invalid_command();
            return;
        }

        match args[1].as_str() {
            "help" => self.display_help(),
            "--version" | "-v" => self.display_version(),
            "device" => self.run_device_command(args),
            _ => {
                self.display_invalid_command();
            }
        }
    }

    fn display_message(&self, message: &str) {
        self.console.write(message);
    }

    fn read_string(&self) -> Result<String, String> {
        self.console.read()
    }

    fn read_number(&self) -> Result<i32, String> {
        let input = self.console.read()?;
        input
            .trim()
            .parse::<i32>()
            .map_err(|_| "Invalid number".to_string())
    }

    fn ask_question(&self, question_type: &QuestionType, question_statement: &str) -> String {
        match question_type {
            QuestionType::String => self.ask_for_string(question_statement),
            _ => panic!("Unsupported question type"),
        }
    }

    fn ask_for_string(&self, message: &str) -> String {
        self.display_message(message);
        self.read_string()
            .map_err(|_| self.ask_for_string(message))
            .unwrap()
    }

    fn display_help(&self) {
        self.display_message(HELP);
    }

    fn display_version(&self) {
        self.display_message(VERSION);
    }

    fn display_invalid_command(&self) {
        self.display_message(INVALID_COMMAND);
    }

    fn run_device_command(&mut self, args: Vec<String>) {
        if args.len() < 3 {
            self.display_invalid_command();
            return;
        }

        match args[2].as_str() {
            "ls" | "list" => self.display_device_list(),
            "new" => self
                .find_device_factory_create_new_device(args)
                .map_err(|e| self.display_message(e.as_str()))
                .unwrap(),
            "rm" | "remove" => self.remove_device(args),
            _ => {
                self.display_invalid_command();
            }
        }
    }

    fn display_device_list(&mut self) {
        self.display_message("Device list:");
        let devices = self.operations.list();
        match devices {
            Ok(devices) => {
                for device in devices {
                    self.display_message(&format!("Device: {}", device.get_name()));
                }
            }
            Err(e) => {
                self.display_message(&e);
            }
        }
    }

    fn find_device_factory_create_new_device(&mut self, args: Vec<String>) -> Result<(), String> {
        if args.len() < 4 {
            self.display_invalid_command();
            return Ok(());
        }
        let device_key = args[3].as_str();
        match self
            .operations
            .get_available_device_factories()
            .iter()
            .find(|&key| key.key == device_key)
        {
            Some(key) => self.create_new_device(key),
            None => {
                self.display_message("No such device configuration exists");
                return Ok(());
            }
        }
    }

    fn create_new_device(&mut self, key: &DeviceFactoryKey) -> Result<(), String> {
        self.display_message("Creating new device of type:");
        let mut device_factory = self
            .operations
            .get_device_factory(key.key.clone())
            .ok_or("No such device configuration exists")?;
        while device_factory.has_next() {
            let question_type = device_factory.get_question_type();
            let question_statement = device_factory.get_question_statement();
            let answer = self.ask_question(&question_type, &question_statement);
            if let Some(device_factory) = Rc::get_mut(&mut device_factory) {
                device_factory.set_question_answer(answer);
            }
        }
        let device = device_factory
            .build()
            .map_err(|_| "Failed to build device")?;
        self.operations
            .add_device(device)
            .map_err(|_| "Failed to add device")?;
        self.display_message("Device created successfully");
        Ok(())
    }

    fn remove_device(&mut self, args: Vec<String>) {
        if args.len() < 4 {
            self.display_invalid_command();
            return;
        }
        let device_name = args[3].as_str();
        match self.operations.remove_by_name(device_name.to_string()) {
            Ok(_) => {
                self.display_message("Removed device successfully");
            }
            Err(e) => {
                self.display_message(&e);
            }
        }
    }
}

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
    let mut command_runner = CommandRunner {
        console: Console,
        operations,
    };
    command_runner.run(args);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::operations::device::MockDeviceOperations;
    use crate::models::secondary_device::{MockDevice, MockDeviceFactory};
    use mockall::predicate::eq;

    #[test]
    fn test_display_message() {
        let mut console = MockUserInterface::new();
        let message = "Hello, world!".to_string();
        console
            .expect_write()
            .times(1)
            .with(eq(message.clone()))
            .return_const(Ok(()));
        let operations = MockDeviceOperations::new();
        let command_runner = CommandRunner {
            console,
            operations,
        };
        command_runner.display_message(&message);
    }

    #[test]
    fn test_read_string() {
        let mut console = MockUserInterface::new();
        console
            .expect_read()
            .times(1)
            .returning(|| Ok("Hello, world!".to_string()));
        let operations = MockDeviceOperations::new();
        let command_runner = CommandRunner {
            console,
            operations,
        };
        let message = command_runner.read_string().unwrap();
        assert_eq!(message, "Hello, world!");
    }

    #[test]
    fn test_read_number() {
        let mut console = MockUserInterface::new();
        console
            .expect_read()
            .times(1)
            .returning(|| Ok("42".to_string()));
        let operations = MockDeviceOperations::new();
        let command_runner = CommandRunner {
            console,
            operations,
        };
        let message = command_runner.read_number().unwrap();
        assert_eq!(message, 42);
    }

    #[test]
    fn should_fail_for_a_number_with_letters() {
        let mut console = MockUserInterface::new();
        console
            .expect_read()
            .times(1)
            .returning(|| Ok("42a".to_string()));
        let operations = MockDeviceOperations::new();
        let command_runner = CommandRunner {
            console,
            operations,
        };
        let message = command_runner.read_number();
        assert!(message.is_err());
    }

    #[test]
    fn display_help() {
        let mut console = MockUserInterface::new();
        console
            .expect_write()
            .times(1)
            .with(eq(HELP.to_string()))
            .return_const(Ok(()));
        let operations = MockDeviceOperations::new();
        let command_runner = CommandRunner {
            console,
            operations,
        };
        command_runner.display_help();
    }

    #[test]
    fn display_help_when_running_with_help_command() {
        let mut console = MockUserInterface::new();
        console
            .expect_write()
            .times(1)
            .with(eq(HELP.to_string()))
            .return_const(Ok(()));
        let operations = MockDeviceOperations::new();
        let mut command_runner = CommandRunner {
            console,
            operations,
        };
        command_runner.run(vec!["/path/to/executable".to_string(), "help".to_string()]);
    }

    #[test]
    fn display_invalid_command() {
        let mut console = MockUserInterface::new();
        console
            .expect_write()
            .times(1)
            .with(eq(INVALID_COMMAND.to_string()))
            .return_const(Ok(()));
        let operations = MockDeviceOperations::new();
        let mut command_runner = CommandRunner {
            console,
            operations,
        };
        command_runner.run(vec![
            "/path/to/executable".to_string(),
            "invalid".to_string(),
        ]);
    }

    #[test]
    fn display_invalid_command_when_running_with_no_args() {
        let mut console = MockUserInterface::new();
        console
            .expect_write()
            .times(1)
            .with(eq(INVALID_COMMAND.to_string()))
            .return_const(Ok(()));
        let operations = MockDeviceOperations::new();
        let mut command_runner = CommandRunner {
            console,
            operations,
        };
        command_runner.run(vec!["/path/to/executable".to_string()]);
    }

    #[test]
    fn display_version_with_full_version_command() {
        let mut console = MockUserInterface::new();
        console
            .expect_write()
            .times(1)
            .with(eq(VERSION.to_string()))
            .return_const(Ok(()));
        let operations = MockDeviceOperations::new();
        let mut command_runner = CommandRunner {
            console,
            operations,
        };
        command_runner.run(vec![
            "/path/to/executable".to_string(),
            "--version".to_string(),
        ]);
    }

    #[test]
    fn display_version_with_short_version_command() {
        let mut console = MockUserInterface::new();
        console
            .expect_write()
            .times(1)
            .with(eq(VERSION.to_string()))
            .return_const(Ok(()));
        let operations = MockDeviceOperations::new();
        let mut command_runner = CommandRunner {
            console,
            operations,
        };
        command_runner.run(vec!["/path/to/executable".to_string(), "-v".to_string()]);
    }

    #[test]
    fn display_list_of_devices() {
        let mut console = MockUserInterface::new();
        let mut device_operations = MockDeviceOperations::new();

        device_operations.expect_list().times(1).returning(move || {
            let mut device = MockDevice::new();
            device
                .expect_get_name()
                .times(1)
                .returning(move || "USBkey".to_string());
            Ok(vec![Box::new(device)])
        });

        console
            .expect_write()
            .times(2)
            .withf(|msg| msg.contains("USBkey") || msg.contains("Device"))
            .return_const(Ok(()));

        let mut command_runner = CommandRunner {
            console,
            operations: device_operations,
        };
        command_runner.run(vec![
            "/path/to/executable".to_string(),
            "device".to_string(),
            "list".to_string(),
        ]);
    }

    #[test]
    fn display_invalid_command_when_running_with_device_command_and_no_subcommand() {
        let mut console = MockUserInterface::new();
        console
            .expect_write()
            .times(1)
            .with(eq(INVALID_COMMAND.to_string()))
            .return_const(Ok(()));
        let operations = MockDeviceOperations::new();
        let mut command_runner = CommandRunner {
            console,
            operations,
        };
        command_runner.run(vec![
            "/path/to/executable".to_string(),
            "device".to_string(),
        ]);
    }

    #[test]
    fn creating_a_new_usb_key() {
        let question = "What is the name of the device?";
        let friendly_name = "USB key";
        let mut console = MockUserInterface::new();
        console
            .expect_write()
            .times(3)
            .withf(move |msg| {
                msg.contains(&question)
                    || msg.contains(NEW_DEVICE_INIT)
                    || msg.contains("Creating new device of type:")
                    || msg.contains("Device created successfully")
            })
            .return_const(Ok(()));
        console
            .expect_read()
            .times(1)
            .returning(|| Ok(friendly_name.to_string()));

        let mut operations = MockDeviceOperations::new();
        operations
            .expect_get_available_device_factories()
            .times(1)
            .returning(|| {
                vec![DeviceFactoryKey {
                    key: "mounted_folder".to_string(),
                    readable_name: "Mounted folder".to_string(),
                }]
            });
        operations
            .expect_get_device_factory()
            .times(1)
            .with(eq("mounted_folder".to_string()))
            .returning(|_| {
                let mut device_factory = MockDeviceFactory::new();
                device_factory.expect_has_next().times(1).returning(|| true);
                device_factory
                    .expect_has_next()
                    .times(1)
                    .returning(|| false);
                device_factory
                    .expect_get_question_type()
                    .times(1)
                    .return_const(QuestionType::String);
                device_factory
                    .expect_get_question_statement()
                    .times(1)
                    .return_const(question.to_string());
                device_factory
                    .expect_set_question_answer()
                    .times(1)
                    .with(eq(friendly_name.to_string()))
                    .return_const(Ok(()));
                device_factory
                    .expect_build()
                    .times(1)
                    .returning(|| Ok(Box::new(MockDevice::new())));
                Some(Rc::new(device_factory))
            });
        operations.expect_add_device().times(1).return_const(Ok(()));

        let mut command_runner = CommandRunner {
            console,
            operations,
        };

        command_runner.run(vec![
            "/path/to/executable".to_string(),
            "device".to_string(),
            "new".to_string(),
            "mounted_folder".to_string(),
        ]);
    }

    #[test]
    fn deleting_a_usb_key() {
        let mut console = MockUserInterface::new();
        console
            .expect_write()
            .times(1)
            .with(eq("Removed device successfully"))
            .return_const(Ok(()));
        let mut device_operations = MockDeviceOperations::new();
        device_operations
            .expect_remove_by_name()
            .times(1)
            .with(eq("USBkey".to_string()))
            .return_const(Ok(()));

        let mut command_runner = CommandRunner {
            console,
            operations: device_operations,
        };
        command_runner.run(vec![
            "/path/to/executable".to_string(),
            "device".to_string(),
            "remove".to_string(),
            "USBkey".to_string(),
        ]);
    }
}
