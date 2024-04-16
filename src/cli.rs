#[cfg(test)]
use mockall::automock;

const HELP: &str = r#"
HibernaCLI
Usage:
    hibernacli [command] [options]

Commands:
    help              Display this help message
    --version or -v   Display the version of the application
    device [cmd]      Manage devices
    config [cmd]      Manage configuration
"#;

const INVALID_COMMAND: &str = "Invalid command, use 'help' to display available commands";
const VERSION: &str = env!("CARGO_PKG_VERSION");

trait UserInterface {
    fn write(&mut self, message: String) -> Result<(), String>;
    fn read(&mut self) -> Result<String, String>;
}

struct Console;

#[cfg_attr(test, automock)]
impl UserInterface for Console {
    fn write(&mut self, message: String) -> Result<(), String> {
        println!("{}", message);
        Ok(())
    }
    fn read(&mut self) -> Result<String, String> {
        let mut buffer = String::new();
        match std::io::stdin().read_line(&mut buffer) {
            Ok(_) => return Ok(buffer),
            Err(e) => return Err(e.to_string()),
        };
    }
}

struct CommandRunner<T: UserInterface> {
    console: T,
}

impl<T: UserInterface> CommandRunner<T> {
    fn run(&mut self, args: Vec<String>) {
        if args.len() < 2 {
            self.display_invalid_command();
            return;
        }

        match args[1].as_str() {
            "help" => self.display_help(),
            "--version" | "-v" => self.display_version(),
            _ => {
                self.display_invalid_command();
            }
        }
    }

    fn display_message(&mut self, message: &String) {
        let _ = self.console.write(message.to_string());
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
}

pub fn run(args: Vec<String>) {
    let mut command_runner = CommandRunner { console: Console };
    command_runner.run(args);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{core::test_utils::mocks::MockDeviceFactory, models::secondary_device::Device};
    use mockall::predicate::eq;

    #[test]
    fn test_display_message() {
        let mut console = MockConsole::new();
        let mut message = "Hello, world!".to_string();
        console
            .expect_write()
            .times(1)
            .with(eq(message.clone()))
            .return_const(Ok(()));
        let mut command_runner = CommandRunner { console };
        let _ = command_runner.display_message(&mut message);
    }

    #[test]
    fn test_read_string() {
        let mut console = MockConsole::new();
        console
            .expect_read()
            .times(1)
            .returning(|| Ok("Hello, world!".to_string()));
        let mut command_runner = CommandRunner { console };
        let message = command_runner.read_string().unwrap();
        assert_eq!(message, "Hello, world!");
    }

    #[test]
    fn test_read_number() {
        let mut console = MockConsole::new();
        console
            .expect_read()
            .times(1)
            .returning(|| Ok("42".to_string()));
        let mut command_runner = CommandRunner { console };
        let message = command_runner.read_number().unwrap();
        assert_eq!(message, 42);
    }

    #[test]
    fn should_fail_for_a_number_with_letters() {
        let mut console = MockConsole::new();
        console
            .expect_read()
            .times(1)
            .returning(|| Ok("42a".to_string()));
        let mut command_runner = CommandRunner { console };
        let message = command_runner.read_number();
        assert!(message.is_err());
    }

    #[test]
    fn display_help() {
        let mut console = MockConsole::new();
        console
            .expect_write()
            .times(1)
            .with(eq(HELP.to_string()))
            .return_const(Ok(()));
        let mut command_runner = CommandRunner { console };
        command_runner.display_help();
    }

    #[test]
    fn display_help_when_running_with_help_command() {
        let mut console = MockConsole::new();
        console
            .expect_write()
            .times(1)
            .with(eq(HELP.to_string()))
            .return_const(Ok(()));
        let mut command_runner = CommandRunner { console };
        command_runner.run(vec!["/path/to/executable".to_string(), "help".to_string()]);
    }

    #[test]
    fn display_invalid_command() {
        let mut console = MockConsole::new();
        console
            .expect_write()
            .times(1)
            .with(eq(INVALID_COMMAND.to_string()))
            .return_const(Ok(()));
        let mut command_runner = CommandRunner { console };
        command_runner.run(vec![
            "/path/to/executable".to_string(),
            "invalid".to_string(),
        ]);
    }

    #[test]
    fn display_invalid_command_when_running_with_no_args() {
        let mut console = MockConsole::new();
        console
            .expect_write()
            .times(1)
            .with(eq(INVALID_COMMAND.to_string()))
            .return_const(Ok(()));
        let mut command_runner = CommandRunner { console };
        command_runner.run(vec!["/path/to/executable".to_string()]);
    }

    #[test]
    fn display_version_with_full_version_command() {
        let mut console = MockConsole::new();
        console
            .expect_write()
            .times(1)
            .with(eq(VERSION.to_string()))
            .return_const(Ok(()));
        let mut command_runner = CommandRunner { console };
        command_runner.run(vec![
            "/path/to/executable".to_string(),
            "--version".to_string(),
        ]);
    }

    #[test]
    fn display_version_with_short_version_command() {
        let mut console = MockConsole::new();
        console
            .expect_write()
            .times(1)
            .with(eq(VERSION.to_string()))
            .return_const(Ok(()));
        let mut command_runner = CommandRunner { console };
        command_runner.run(vec!["/path/to/executable".to_string(), "-v".to_string()]);
    }

    fn when_asked_to_create_device_gets_the_question_from_device_factory() {
        let mut device_factory = MockDeviceFactory {};
        let mut console = MockConsole::new();
        // to be continued...
    }
}
