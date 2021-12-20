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

    pub fn empty_line(&self) {
        println!();
    }

    pub fn output(&self, output: &str) {
        println!("{}", output);
    }

    pub fn input(&self, prompt: &str) -> String {
        let mut line = String::new();
        print!("{}", prompt);
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut line).expect("Error: Could not read a line");
        return String::from(line.trim());
    }

    pub fn get_command(&self) -> Result<ParsedArgs, InputError> {
        return self.parser.parse(&self.input(&self.prompt));
    }
}
