trait UserInterface {
    fn write(&mut self, message: String) -> Result<usize, std::io::Error>;
    fn read(&mut self) -> Result<String, std::io::Error>;
}

struct Console;

impl UserInterface for Console {
    fn write(&mut self, message: String) -> Result<usize, std::io::Error> {
        println!("{}", message);
        Ok(message.len())
    }
    fn read(&mut self) -> Result<String, std::io::Error> {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer)?;
        Ok(buffer)
    }
}

trait CommandInterface {
    fn run(&mut self, args: Vec<String>);
    fn display_message(&mut self, message: &String);
    fn read_string(&mut self) -> Result<String, std::io::Error>;
    fn read_number(&mut self) -> Result<i32, std::io::Error>;
}

struct CommandRunner<'a> {
    console: &'a mut dyn UserInterface,
}

impl CommandInterface for CommandRunner<'_> {
    fn run(&mut self, args: Vec<String>) {
        self.display_message(&"HibernaCLI".to_string());

        for arg in args.iter() {
            let _ = self.display_message(arg);
        }
    }

    fn display_message(&mut self, message: &String) {
        let _ = self.console.write(message.to_string());
    }

    fn read_string(&mut self) -> Result<String, std::io::Error> {
        self.console.read()
    }

    fn read_number(&mut self) -> Result<i32, std::io::Error> {
        let input = self.console.read()?;
        match input.trim().parse::<i32>() {
            Ok(number) => Ok(number),
            Err(_) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid number",
            )),
        }
    }
}

pub fn run(args: Vec<String>) {
    let mut command_runner = CommandRunner {
        console: &mut Console,
    };
    command_runner.run(args);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Read, Write};

    struct ConsoleMock {
        output: io::Cursor<Vec<u8>>,
        input: io::Cursor<Vec<u8>>,
    }

    impl UserInterface for ConsoleMock {
        fn write(&mut self, message: String) -> Result<usize, std::io::Error> {
            self.output.write(message.as_bytes())
        }
        fn read(&mut self) -> Result<String, std::io::Error> {
            let mut buffer = String::new();
            match self.input.read_to_string(&mut buffer) {
                Ok(_) => Ok(buffer),
                Err(e) => Err(e),
            }
        }
    }

    #[test]
    fn test_display_message() {
        let mut console = ConsoleMock {
            output: io::Cursor::new(vec![]),
            input: io::Cursor::new(vec![]),
        };
        let mut command_runner = CommandRunner {
            console: &mut console,
        };
        let message = "Hello, world!".to_string();
        let _ = command_runner.display_message(&message);
        assert_eq!(console.output.get_ref(), message.as_bytes());
    }

    #[test]
    fn test_read_string() {
        let mut console = ConsoleMock {
            output: io::Cursor::new(vec![]),
            input: io::Cursor::new("Hello, world!".as_bytes().to_vec()),
        };
        let mut command_runner = CommandRunner {
            console: &mut console,
        };
        let message = command_runner.read_string().unwrap();
        assert_eq!(message, "Hello, world!");
    }

    #[test]
    fn test_read_number() {
        let mut console = ConsoleMock {
            output: io::Cursor::new(vec![]),
            input: io::Cursor::new("42".as_bytes().to_vec()),
        };
        let mut command_runner = CommandRunner {
            console: &mut console,
        };
        let message = command_runner.read_number().unwrap();
        assert_eq!(message, 42);
    }

    #[test]
    fn should_fail_for_a_number_with_letters() {
        let mut console = ConsoleMock {
            output: io::Cursor::new(vec![]),
            input: io::Cursor::new("42a".as_bytes().to_vec()),
        };
        let mut command_runner = CommandRunner {
            console: &mut console,
        };
        let message = command_runner.read_number();
        assert!(message.is_err());
    }

    #[test]
    fn display_help_when_no_argument() {
        let args = vec![];
        run(args);
    }
}
