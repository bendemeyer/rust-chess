use crate::interface::shell::InteractiveShell;

use super::arguments::ArgumentParser;


pub struct Interface {
    shell: InteractiveShell,
}


impl Interface {
    pub fn new() -> Interface {
        let prompt = "chess > ";
        let mut builder = ArgumentParser::builder();
        builder.add_positional_arg("command", true, false);
        let parser = builder.build();
        return Interface {
            shell: InteractiveShell::new(Some(prompt), parser)
        }
    }

    pub fn init(&self) {
        loop {
            let result = self.shell.get_command();
            match result {
                Err(e) => {
                    println!("{}", e.msg);
                    break;
                },
                Ok(args) => match args.get_arg("command") {
                    None => println!("No command was given!"),
                    Some(val) => println!("Command {} was given!", val)
                }
            };
            println!();
        }
    }
}
