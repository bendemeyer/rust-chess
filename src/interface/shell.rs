use std::io::Write;

use crate::interface::arguments::{ArgumentParser, ParsedArgs};
use crate::util::errors::InputError;


pub struct InteractiveShell {
    prompt: String,
    parser: ArgumentParser,
}


impl InteractiveShell {
    pub fn new(prompt: Option<&str>, parser: ArgumentParser) -> InteractiveShell {
        let default_prompt = ">>> ";
        return InteractiveShell {
            prompt: String::from( match prompt { Some(x) => x, None => default_prompt } ),
            parser: parser,
        }
    }

    pub fn get_command(&self) -> Result<ParsedArgs, InputError> {
        let mut line = String::new();
        print!("{}", self.prompt);
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut line).expect("Error: Could not read a line");

        return self.parser.parse(&line.trim().to_string());
    }
}
