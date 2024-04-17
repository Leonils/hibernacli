#[cfg(test)]
use mockall::automock;

use crate::adapters::operations::device::DeviceOperations;
use crate::core::operations::Operations;
use crate::devices::local_file_storage::{
    LocalFileStorage, StandardFileSystem, StandardPathProvider,
};

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

const DEFAULT_CONFIG: &str = r#"[[devices]]
    name = "MyPersonalDevice"
    type = "MockDevice"
"#;

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

    fn display_message(&mut self, message: &String) {
        let _ = self.console.write(message);
    }

    fn read_string(&mut self) -> Result<String, String> {
        self.console.read()
    }

    fn read_number(&mut self) -> Result<i32, String> {
        let input = self.console.read()?;
        match input.trim().parse::<i32>() {
            Ok(number) => Ok(number),
            Err(_) => Err("Invalid number".to_string()),
        }
    }

    fn display_help(&mut self) {
        let _ = self.display_message(&HELP.to_string());
    }

    fn display_version(&mut self) {
        let _ = self.display_message(&VERSION.to_string());
    }

    fn display_invalid_command(&mut self) {
        let _ = self.display_message(&INVALID_COMMAND.to_string());
    }

    fn run_device_command(&mut self, args: Vec<String>) {
        if args.len() < 3 {
            self.display_invalid_command();
            return;
        }

        match args[2].as_str() {
            "ls" | "list" => self.display_device_list(),
            _ => {
                self.display_invalid_command();
            }
        }
    }

    fn display_device_list(&mut self) {
        let _ = self.display_message(&"Device list:".to_string());
        let devices = self.operations.list();
        match devices {
            Ok(devices) => {
                for device in devices {
                    let _ = self.display_message(&format!("Device: {}", device.get_name()));
                }
            }
            Err(e) => {
                let _ = self.display_message(&e);
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
    let operations = Operations::new(Box::new(local_file_storage));
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
    use crate::models::secondary_device::MockDevice;
    use mockall::predicate::eq;

    #[test]
    fn test_display_message() {
        let mut console = MockUserInterface::new();
        let mut message = "Hello, world!".to_string();
        console
            .expect_write()
            .times(1)
            .with(eq(message.clone()))
            .return_const(Ok(()));
        let operations = MockDeviceOperations::new();
        let mut command_runner = CommandRunner {
            console,
            operations,
        };
        let _ = command_runner.display_message(&mut message);
    }

    #[test]
    fn test_read_string() {
        let mut console = MockUserInterface::new();
        console
            .expect_read()
            .times(1)
            .returning(|| Ok("Hello, world!".to_string()));
        let operations = MockDeviceOperations::new();
        let mut command_runner = CommandRunner {
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
        let mut command_runner = CommandRunner {
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
        let mut command_runner = CommandRunner {
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
        let mut command_runner = CommandRunner {
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
        let mut device = MockDevice::new();

        device
            .expect_get_name()
            .times(1)
            .returning(move || "USBkey".to_string());

        device_operations
            .expect_list()
            .times(1)
            .returning(|| Ok(vec![Box::new(MockDevice::new())]));

        console
            .expect_write()
            .times(2)
            .withf(|msg| msg.contains("USBkey") || msg.contains("Device"))
            .return_const(Ok(()));

        let operations = MockDeviceOperations::new();
        let mut command_runner = CommandRunner {
            console,
            operations,
        };
        command_runner.run(vec![
            "/path/to/executable".to_string(),
            "device".to_string(),
            "ls".to_string(),
        ]);
    }
}
