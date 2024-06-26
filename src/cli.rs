#[cfg(test)]
use mockall::automock;

use crate::adapters::operations::device::DeviceOperations;
use crate::adapters::operations::project::{AddProjectArgs, ProjectOperations};
use crate::models::question::QuestionType;
use crate::models::secondary_device::DeviceFactoryKey;

const HELP: &str = r#"
HibernaCLI
Usage:
    hibernacli [command] [options]

Commands:
    help                        Display this help message
    
    --version or -v             Display the version of the application
    
    device [opt]                Manage devices
        ls or list                  List all devices
        new MountedFolder           Create a new mounted folder device
        rm or remove [device_name]  Remove a device
    
    project [opt]               Manage projects
        ls or list                  List all projects
        new                         Create a new project
"#;

const INVALID_COMMAND: &str = "Invalid command, use 'help' to display available commands";
const TODO_COMMAND: &str = "This command is not implemented yet, but it will be soon!";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(test, automock)]
pub trait UserInterface {
    fn write(&self, message: &str) -> ();
    fn read(&self) -> Result<String, String>;
}

pub struct Console;

impl UserInterface for Console {
    fn write(&self, message: &str) -> () {
        println!("{}", message);
    }
    fn read(&self) -> Result<String, String> {
        let mut buffer = String::new();
        match std::io::stdin().read_line(&mut buffer) {
            Ok(_) => return Ok(buffer),
            Err(e) => return Err(e.to_string()),
        };
    }
}

pub struct CommandRunner<'a, T: UserInterface, U: DeviceOperations, V: ProjectOperations> {
    console: T,
    device_operations: &'a U,
    project_operations: &'a V,
}

impl<'a, T: UserInterface, U: DeviceOperations, V: ProjectOperations> CommandRunner<'a, T, U, V> {
    pub fn new(console: T, device_operations: &'a U, project_operations: &'a V) -> Self {
        CommandRunner {
            console,
            device_operations,
            project_operations,
        }
    }

    pub fn run(&self, args: Vec<String>) {
        if args.len() < 2 {
            self.display_invalid_command();
            return;
        }

        match args[1].as_str() {
            "help" => self.display_help(),
            "--version" | "-v" => self.display_version(),
            "device" => self.run_device_command(args),
            "project" => self.run_project_command(args),
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
            QuestionType::UnixPath => self.ask_for_unix_path(question_statement),
            _ => panic!("Unsupported question type"),
        }
    }

    fn ask_for_string(&self, message: &str) -> String {
        self.display_message(message);
        self.read_string()
            .map_err(|_| self.ask_for_string(message))
            .map(|s| s.trim().to_string())
            .unwrap()
    }

    fn ask_for_unix_path(&self, message: &str) -> String {
        self.display_message(message);
        self.display_message("Enter a valid Unix path");
        self.read_string()
            .map_err(|_| self.ask_for_unix_path(message))
            .map(|s| s.trim().to_string())
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

    fn display_todo(&self) {
        self.display_message(TODO_COMMAND);
    }

    fn run_device_command(&self, args: Vec<String>) {
        if args.len() < 3 {
            self.display_invalid_command();
            return;
        }

        let result = match args[2].as_str() {
            "ls" | "list" => self.display_device_list(),
            "new" => self.find_device_factory_create_new_device(args),
            "rm" | "remove" => self.remove_device(args),
            _ => Ok(self.display_invalid_command()),
        };

        result.unwrap_or_else(|e| self.display_message(&e));
    }

    fn display_device_list(&self) -> Result<(), String> {
        self.display_message("Device list:");
        let devices = self.device_operations.list().map_err(|e| e.to_string())?;
        for device in devices {
            self.display_message(&format!("Device: {}", device.get_name()));
        }
        Ok(())
    }

    fn find_device_factory_create_new_device(&self, args: Vec<String>) -> Result<(), String> {
        if args.len() < 4 {
            self.display_invalid_command();
            return Ok(());
        }
        let device_key = args[3].as_str();
        self.device_operations
            .get_available_device_factories()
            .iter()
            .find(|&key| key.key == device_key)
            .map(|key| self.create_new_device(key))
            .unwrap_or_else(|| Err("Device factory not found".to_string()))
    }

    fn create_new_device(&self, key: &DeviceFactoryKey) -> Result<(), String> {
        self.display_message("Creating new device of type:");
        let mut device_factory = self
            .device_operations
            .get_device_factory(key.key.clone())
            .ok_or("No such device configuration exists")?;

        while device_factory.has_next() {
            let question_type = device_factory.get_question_type();
            let question_statement = device_factory.get_question_statement();
            let answer = self.ask_question(&question_type, &question_statement);
            device_factory
                .set_question_answer(answer)
                .map_err(|_| "Failed to set answer")?;
        }
        let device = device_factory
            .build()
            .map_err(|_| "Failed to build device")?;
        self.device_operations
            .add_device(device)
            .map_err(|_| "Failed to add device")?;
        self.display_message("Device created successfully");
        Ok(())
    }

    fn remove_device(&self, args: Vec<String>) -> Result<(), String> {
        if args.len() < 4 {
            return Ok(self.display_invalid_command());
        }
        let device_name = args[3].as_str();
        self.device_operations
            .remove_by_name(device_name.to_string())
            .map_err(|e| e.to_string())
            .unwrap();

        self.display_message("Removed device successfully");
        Ok(())
    }

    fn run_project_command(&self, args: Vec<String>) {
        if args.len() < 3 {
            self.display_invalid_command();
            return;
        }

        match args[2].as_str() {
            "ls" | "list" => self
                .display_project_list()
                .unwrap_or_else(|e| self.display_message(&e)),
            "new" => self
                .add_project()
                .unwrap_or_else(|e| self.display_message(&e)),
            "rm" | "remove" => self.display_todo(),
            _ => {
                self.display_invalid_command();
            }
        }
    }

    fn display_project_list(&self) -> Result<(), String> {
        self.display_message("Project list:");
        let projects = self.project_operations.list_projects()?;
        for project in projects {
            self.display_message(&format!("Project: {}", project.get_name()));
        }
        Ok(())
    }

    fn add_project(&self) -> Result<(), String> {
        let project_name = self.ask_for_string("What is the name of the project?");
        let project_path = self.ask_for_unix_path("What is the path to the project?");
        self.project_operations
            .add_project(AddProjectArgs {
                name: project_name,
                location: project_path,
            })
            .map_err(|e| e.to_string())?;
        self.display_message("Project created successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::operations::device::MockDeviceOperations;
    use crate::adapters::operations::project::MockProjectOperations;
    use crate::models::secondary_device::{MockDevice, MockDeviceFactory};
    use mockall::predicate::eq;

    trait ExpectReadWrite {
        fn expect_to_read(self, read_value: &str) -> Self;
        fn expect_to_write(self, written_value: &str) -> Self;
    }

    impl ExpectReadWrite for MockUserInterface {
        fn expect_to_read(mut self, read_value: &str) -> Self {
            let r = read_value.to_string();
            self.expect_read().times(1).returning(move || Ok(r.clone()));
            self
        }

        fn expect_to_write(mut self, written_value: &str) -> Self {
            self.expect_write()
                .times(1)
                .with(eq(written_value.to_string()))
                .return_const(());
            self
        }
    }

    macro_rules! run_command {
        ($console:ident, $device_operations:ident, $project_operations:ident, $args: expr) => {{
            let command_runner =
                CommandRunner::new($console, &$device_operations, &$project_operations);
            let args_with_executable = format!("/path/to/executable {}", $args);
            let split_args: Vec<String> = args_with_executable
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            command_runner.run(split_args);
        }};
    }

    #[test]
    fn test_display_message() {
        let mut console = MockUserInterface::new();
        let project_operations = MockProjectOperations::new();
        let message = "Hello, world!".to_string();
        console
            .expect_write()
            .times(1)
            .with(eq(message.clone()))
            .return_const(());
        let device_operations = MockDeviceOperations::new();
        let command_runner = CommandRunner::new(console, &device_operations, &project_operations);
        command_runner.display_message(&message);
    }

    #[test]
    fn test_read_string() {
        let mut console = MockUserInterface::new();
        let project_operations = MockProjectOperations::new();
        console
            .expect_read()
            .times(1)
            .returning(|| Ok("Hello, world!".to_string()));
        let device_operations = MockDeviceOperations::new();
        let command_runner = CommandRunner::new(console, &device_operations, &project_operations);
        let message = command_runner.read_string().unwrap();
        assert_eq!(message, "Hello, world!");
    }

    #[test]
    fn test_read_number() {
        let mut console = MockUserInterface::new();
        let project_operations = MockProjectOperations::new();
        console
            .expect_read()
            .times(1)
            .returning(|| Ok("42".to_string()));
        let device_operations = MockDeviceOperations::new();
        let command_runner = CommandRunner::new(console, &device_operations, &project_operations);
        let message = command_runner.read_number().unwrap();
        assert_eq!(message, 42);
    }

    #[test]
    fn should_fail_for_a_number_with_letters() {
        let mut console = MockUserInterface::new();
        let project_operations = MockProjectOperations::new();
        console
            .expect_read()
            .times(1)
            .returning(|| Ok("42a".to_string()));
        let device_operations = MockDeviceOperations::new();
        let command_runner = CommandRunner::new(console, &device_operations, &project_operations);
        let message = command_runner.read_number();
        assert!(message.is_err());
    }

    #[test]
    fn display_help() {
        let mut console = MockUserInterface::new();
        let project_operations = MockProjectOperations::new();
        console
            .expect_write()
            .times(1)
            .with(eq(HELP.to_string()))
            .return_const(());
        let device_operations = MockDeviceOperations::new();
        let command_runner = CommandRunner::new(console, &device_operations, &project_operations);
        command_runner.display_help();
    }

    #[test]
    fn display_help_when_running_with_help_command() {
        let mut console = MockUserInterface::new();
        let project_operations = MockProjectOperations::new();
        console
            .expect_write()
            .times(1)
            .with(eq(HELP.to_string()))
            .return_const(());
        let device_operations = MockDeviceOperations::new();
        let command_runner = CommandRunner::new(console, &device_operations, &project_operations);
        command_runner.run(vec!["/path/to/executable".to_string(), "help".to_string()]);
    }

    #[test]
    fn display_invalid_command() {
        let mut console = MockUserInterface::new();
        let project_operations = MockProjectOperations::new();
        console
            .expect_write()
            .times(1)
            .with(eq(INVALID_COMMAND.to_string()))
            .return_const(());
        let device_operations = MockDeviceOperations::new();
        let command_runner = CommandRunner::new(console, &device_operations, &project_operations);
        command_runner.run(vec![
            "/path/to/executable".to_string(),
            "invalid".to_string(),
        ]);
    }

    #[test]
    fn display_invalid_command_when_running_with_no_args() {
        let mut console = MockUserInterface::new();
        let project_operations = MockProjectOperations::new();
        console
            .expect_write()
            .times(1)
            .with(eq(INVALID_COMMAND.to_string()))
            .return_const(());
        let device_operations = MockDeviceOperations::new();
        let command_runner = CommandRunner::new(console, &device_operations, &project_operations);
        command_runner.run(vec!["/path/to/executable".to_string()]);
    }

    #[test]
    fn display_version_with_full_version_command() {
        let mut console = MockUserInterface::new();
        let project_operations = MockProjectOperations::new();
        console
            .expect_write()
            .times(1)
            .with(eq(VERSION.to_string()))
            .return_const(());
        let device_operations = MockDeviceOperations::new();
        let command_runner = CommandRunner::new(console, &device_operations, &project_operations);
        command_runner.run(vec![
            "/path/to/executable".to_string(),
            "--version".to_string(),
        ]);
    }

    #[test]
    fn display_version_with_short_version_command() {
        let mut console = MockUserInterface::new();
        let project_operations = MockProjectOperations::new();
        console
            .expect_write()
            .times(1)
            .with(eq(VERSION.to_string()))
            .return_const(());
        let device_operations = MockDeviceOperations::new();
        let command_runner = CommandRunner::new(console, &device_operations, &project_operations);
        command_runner.run(vec!["/path/to/executable".to_string(), "-v".to_string()]);
    }

    #[test]
    fn display_list_of_devices() {
        let mut console = MockUserInterface::new();
        let project_operations = MockProjectOperations::new();
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
            .return_const(());

        let command_runner = CommandRunner::new(console, &device_operations, &project_operations);
        command_runner.run(vec![
            "/path/to/executable".to_string(),
            "device".to_string(),
            "list".to_string(),
        ]);
    }

    #[test]
    fn display_invalid_command_when_running_with_device_command_and_no_subcommand() {
        let mut console = MockUserInterface::new();
        let project_operations = MockProjectOperations::new();
        console
            .expect_write()
            .times(1)
            .with(eq(INVALID_COMMAND.to_string()))
            .return_const(());
        let device_operations = MockDeviceOperations::new();
        let command_runner = CommandRunner::new(console, &device_operations, &project_operations);
        command_runner.run(vec![
            "/path/to/executable".to_string(),
            "device".to_string(),
        ]);
    }

    #[test]
    fn creating_a_new_usb_key_with_a_string_question() {
        let question = "What is the name of the device?";
        let friendly_name = "USB key";
        let mut console = MockUserInterface::new();
        let project_operations = MockProjectOperations::new();
        console
            .expect_write()
            .times(3)
            .withf(move |msg| {
                msg.contains(&question)
                    || msg.contains("Creating new device of type:")
                    || msg.contains("Device created successfully")
            })
            .return_const(());
        console
            .expect_read()
            .times(1)
            .returning(|| Ok(friendly_name.to_string()));

        let mut device_operations = MockDeviceOperations::new();
        device_operations
            .expect_get_available_device_factories()
            .times(1)
            .returning(|| {
                vec![DeviceFactoryKey {
                    key: "mounted_folder".to_string(),
                    readable_name: "Mounted folder".to_string(),
                }]
            });
        device_operations
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
                Some(Box::new(device_factory))
            });
        device_operations
            .expect_add_device()
            .times(1)
            .return_const(Ok(()));

        let command_runner = CommandRunner::new(console, &device_operations, &project_operations);

        command_runner.run(vec![
            "/path/to/executable".to_string(),
            "device".to_string(),
            "new".to_string(),
            "mounted_folder".to_string(),
        ]);
    }

    #[test]
    fn creating_a_new_usb_key_with_a_unix_path_question() {
        let question = "What is the path to the device?";
        let mut console = MockUserInterface::new();
        let project_operations = MockProjectOperations::new();
        console
            .expect_write()
            .times(4)
            .withf(move |msg| {
                msg.contains(&question)
                    || msg.contains("Creating new device of type:")
                    || msg.contains("Device created successfully")
                    || msg.contains("Enter a valid Unix path")
            })
            .return_const(());
        console
            .expect_read()
            .times(1)
            .returning(|| Ok("/mnt/usbkey".to_string()));

        let mut device_operations = MockDeviceOperations::new();
        device_operations
            .expect_get_available_device_factories()
            .times(1)
            .returning(|| {
                vec![DeviceFactoryKey {
                    key: "mounted_folder".to_string(),
                    readable_name: "Mounted folder".to_string(),
                }]
            });
        device_operations
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
                    .return_const(QuestionType::UnixPath);
                device_factory
                    .expect_get_question_statement()
                    .times(1)
                    .return_const(question.to_string());
                device_factory
                    .expect_set_question_answer()
                    .times(1)
                    .with(eq("/mnt/usbkey".to_string()))
                    .return_const(Ok(()));
                device_factory
                    .expect_build()
                    .times(1)
                    .returning(|| Ok(Box::new(MockDevice::new())));
                Some(Box::new(device_factory))
            });
        device_operations
            .expect_add_device()
            .times(1)
            .return_const(Ok(()));

        let command_runner = CommandRunner::new(console, &device_operations, &project_operations);

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
        let project_operations = MockProjectOperations::new();
        console
            .expect_write()
            .times(1)
            .with(eq("Removed device successfully"))
            .return_const(());
        let mut device_operations = MockDeviceOperations::new();
        device_operations
            .expect_remove_by_name()
            .times(1)
            .with(eq("USBkey".to_string()))
            .return_const(Ok(()));

        let command_runner = CommandRunner::new(console, &device_operations, &project_operations);
        command_runner.run(vec![
            "/path/to/executable".to_string(),
            "device".to_string(),
            "remove".to_string(),
            "USBkey".to_string(),
        ]);
    }

    #[test]
    fn display_invalid_command_when_running_with_device_command_and_invalid_subcommand() {
        let mut console = MockUserInterface::new();
        let project_operations = MockProjectOperations::new();
        console
            .expect_write()
            .times(1)
            .with(eq(INVALID_COMMAND.to_string()))
            .return_const(());
        let device_operations = MockDeviceOperations::new();
        let command_runner = CommandRunner::new(console, &device_operations, &project_operations);
        command_runner.run(vec![
            "/path/to/executable".to_string(),
            "device".to_string(),
            "invalid".to_string(),
        ]);
    }

    #[test]
    fn display_list_of_projects() {
        let mut console = MockUserInterface::new();
        let mut project_operations = MockProjectOperations::new();
        project_operations
            .expect_list_projects()
            .times(1)
            .returning(|| Ok(vec![]));
        console
            .expect_write()
            .times(1)
            .with(eq("Project list:"))
            .return_const(());
        let device_operations = MockDeviceOperations::new();
        let command_runner = CommandRunner::new(console, &device_operations, &project_operations);
        command_runner.run(vec![
            "/path/to/executable".to_string(),
            "project".to_string(),
            "list".to_string(),
        ]);
    }

    #[test]
    fn display_invalid_command_when_running_with_project_command_and_invalid_subcommand() {
        let mut console = MockUserInterface::new();
        let project_operations = MockProjectOperations::new();
        console
            .expect_write()
            .times(1)
            .with(eq(INVALID_COMMAND.to_string()))
            .return_const(());
        let device_operations = MockDeviceOperations::new();
        let command_runner = CommandRunner::new(console, &device_operations, &project_operations);
        command_runner.run(vec![
            "/path/to/executable".to_string(),
            "project".to_string(),
            "invalid".to_string(),
        ]);
    }

    #[test]
    fn adding_a_new_project() {
        let mut project_operations = MockProjectOperations::new();
        project_operations
            .expect_add_project()
            .times(1)
            .with(eq(AddProjectArgs {
                name: "MyProject".to_string(),
                location: "/mnt/projects/myproject".to_string(),
            }))
            .return_const(Ok(()));

        let console = MockUserInterface::new()
            .expect_to_write("What is the name of the project?")
            .expect_to_read("MyProject")
            .expect_to_write("What is the path to the project?")
            .expect_to_write("Enter a valid Unix path")
            .expect_to_read("/mnt/projects/myproject")
            .expect_to_write("Project created successfully");

        let device_operations = MockDeviceOperations::new();
        run_command!(
            console,
            device_operations,
            project_operations,
            "project new"
        );
    }

    #[test]
    fn when_failing_to_add_a_project_it_shall_print_error_to_user() {
        let mut project_operations = MockProjectOperations::new();
        project_operations
            .expect_add_project()
            .times(1)
            .return_const(Err("Project already exists".to_string()));

        let console = MockUserInterface::new()
            .expect_to_write("What is the name of the project?")
            .expect_to_read("MyProject")
            .expect_to_write("What is the path to the project?")
            .expect_to_write("Enter a valid Unix path")
            .expect_to_read("/mnt/projects/myproject")
            .expect_to_write("Project already exists");

        let device_operations = MockDeviceOperations::new();
        run_command!(
            console,
            device_operations,
            project_operations,
            "project new"
        );
    }
}
