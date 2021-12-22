use std::fs::OpenOptions;

use rustyline::{Editor, Config, Cmd, KeyEvent, KeyCode, Modifiers};

use crate::interface::arguments::{ArgumentParser, ParsedArgs};
use crate::util::errors::InputError;


static HISTORY_FILE: &str = "history.txt";


pub struct InteractiveShell {
    editor: Editor<()>,
    prompt: String,
    parser: ArgumentParser,
}


impl InteractiveShell {
    pub fn new(prompt: Option<&str>, parser: ArgumentParser) -> InteractiveShell {
        let default_prompt = ">>> ";
        let mut shell = InteractiveShell {
            editor: Editor::with_config(Config::builder().auto_add_history(false).build()),
            prompt: String::from( match prompt { Some(x) => x, None => default_prompt } ),
            parser: parser,
        };

        let _ = OpenOptions::new().create_new(true).open(HISTORY_FILE);
        shell.editor.load_history(HISTORY_FILE).unwrap();
        shell.editor.bind_sequence(KeyEvent { 0: KeyCode::Up, 1: Modifiers::NONE }, Cmd::PreviousHistory);
        shell.editor.bind_sequence(KeyEvent { 0: KeyCode::Down, 1: Modifiers::NONE }, Cmd::NextHistory);

        return shell
    }

    pub fn empty_line(&self) {
        println!();
    }

    pub fn output(&self, output: &str) {
        println!("{}", output);
    }

    pub fn input(&self, prompt: &str) -> String {
        return Editor::<()>::new().readline(prompt).unwrap();
    }

    pub fn get_command(&mut self) -> Result<ParsedArgs, InputError> {
        let input = self.editor.readline(&self.prompt).unwrap();
        self.editor.add_history_entry(input.clone());
        self.editor.append_history(HISTORY_FILE).unwrap();
        return self.parser.parse(&input);
    }
}
