trait Console {
    fn write(&mut self, message: String) -> Result<usize, std::io::Error>;
    fn read(&mut self) -> Result<String, std::io::Error>;
}

struct TerminalConsole {}

impl Console for TerminalConsole {
    fn write(&mut self, message: String) -> Result<usize, std::io::Error> {
        println!("{}", message);
        Ok(message.len())
    }
    fn read(&mut self) -> Result<String, std::io::Error> {
        Ok("".to_string())
    }
}

pub fn run() {
    let mut console = TerminalConsole {};
    display_message(&mut console, &"HibernaCLI".to_string())
}

fn display_message(console: &mut dyn Console, message: &String) {
    let _ = console.write(message.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Read, Write};

    struct ConsoleMock {
        output: io::Cursor<Vec<u8>>,
        input: io::Cursor<Vec<u8>>,
    }

    impl Console for ConsoleMock {
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
        let message = "Hello, world!".to_string();
        let _ = display_message(&mut console, &message);
        assert_eq!(console.output.get_ref(), message.as_bytes());
    }
}
