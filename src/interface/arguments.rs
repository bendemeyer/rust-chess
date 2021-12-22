use std::collections::{HashSet, HashMap, VecDeque};

use crate::util::errors::InputError;


pub enum ParsedArgs {
    SubCommand(SubCommand),
    Arguments(Arguments),
}


#[derive(Clone)]
struct Argument {
    pub name: String,
    pub is_required: bool,
    pub keys: HashSet<String>,
    pub is_narg: bool,
    pub is_flag: bool,
}


pub struct SubCommand {
    pub name: String,
    pub args: Box<ParsedArgs>,
}

#[derive(Default)]
pub struct Arguments {
    pub args: HashMap<String, String>,
    pub nargs: HashMap<String, Vec<String>>,
    pub flags: HashSet<String>,
}

impl Arguments {
    pub fn get_arg(&self, name: &str) -> Option<String> {
        return match self.args.get(name) {
            Some(arg) => Some(String::from(arg)),
            None => None
        };
    }

    pub fn get_narg(&self, name: &str) -> Option<Vec<String>> {
        return match self.nargs.get(name) {
            Some(arg) => Some(Vec::from_iter(arg.iter().map(|x| String::from(x)))),
            None => None
        };
    }

    pub fn get_flag(&self, name: &str) -> bool {
        return self.flags.contains(name)
    }
}


#[derive(Default)]
pub struct ArgumentParserBuilder {
    sub_commands: HashMap<String, ArgumentParserBuilder>,
    required: HashSet<String>,
    positional: Vec<Argument>,
    named: HashMap<String, Argument>,
    keys: HashMap<String, String>,
    key_set: HashSet<String>,
    has_optional_positional: bool,
    has_positional_narg: bool,
}


impl ArgumentParserBuilder {
    fn new() -> ArgumentParserBuilder {
        return Default::default();
    }

    pub fn add_subcommand(&mut self, name: &str) -> Result<&mut ArgumentParserBuilder, ()> {
        let sub_builder: ArgumentParserBuilder = Default::default();
        self.sub_commands.insert(String::from(name), sub_builder);
        return Ok(self.sub_commands.get_mut(name).unwrap());

    }

    pub fn add_positional_arg(&mut self, name: &str, required: bool, narg: bool) -> Result<&mut ArgumentParserBuilder, InputError> {
        if self.has_positional_narg {
            return Err(InputError::new("Additional positional arguments cannot be added after a positional narg."))
        }
        if self.has_optional_positional && required {
            return Err(InputError::new("Required positional arguments cannot be added after optional ones."))
        }
        if self.has_optional_positional && narg {
            return Err(InputError::new("Positional narg arguments cannot be added after optional positional arguments."))
        }
        self.positional.push(Argument {
            name: String::from(name),
            is_required: required,
            keys: HashSet::new(),
            is_narg: narg,
            is_flag: false,
        });
        if required {
            self.required.insert(String::from(name));
        } else {
            self.has_optional_positional = true;
        }
        if narg {
            self.has_positional_narg = true;
        }
        return Ok(self);
    }

    pub fn add_named_arg(&mut self, name: &str, keys: HashSet<&str>, required: bool, narg: bool) -> Result<&mut ArgumentParserBuilder, InputError> {
        if keys.is_empty() {
            return Err(InputError::new("Named arguments must supply at least one key."))
        }
        if !self.key_set.is_disjoint(&HashSet::from_iter(keys.iter().map(|x| String::from(*x) ))) {
            return Err(InputError::new("Some of the provided keys already belong to other arguments."))
        }
        self.named.insert(String::from(name), Argument {
            name: String::from(name),
            is_required: required,
            keys: HashSet::from_iter(keys.iter().map(|x| { String::from(*x) })),
            is_narg: narg,
            is_flag: false,
        });
        for key in keys {
            self.keys.insert(String::from(key), String::from(name));
            self.key_set.insert(String::from(key));
        }
        if required {
            self.required.insert(String::from(name));
        }
        return Ok(self);
    }

    pub fn add_flag_arg(&mut self, name: &str, keys: HashSet<&str>) -> Result<&mut ArgumentParserBuilder, InputError> {
        if keys.is_empty() {
            return Err(InputError::new("Flag arguments must supply at least one key."))
        }
        if !self.key_set.is_disjoint(&HashSet::from_iter(keys.iter().map(|x| String::from(*x) ))) {
            return Err(InputError::new("Some of the provided keys already belong to other arguments."))
        }
        self.named.insert(String::from(name), Argument {
            name: String::from(name),
            is_required: false,
            keys: HashSet::from_iter(keys.iter().map(|x| { String::from(*x) })),
            is_narg: false,
            is_flag: true,
        });
        for key in keys {
            self.keys.insert(String::from(key), String::from(name));
            self.key_set.insert(String::from(key));
        }
        return Ok(self);
    }

    pub fn build(&self) -> ArgumentParser {
        return ArgumentParser {
            sub_commands: self.sub_commands.iter().map(|(n, b)| {
                (String::from(n), b.build())
            }).collect(),
            required: self.required.clone(),
            positional: self.positional.clone(),
            named: self.named.clone(),
            keys: self.keys.clone(),
        }
    }
}


pub struct ArgumentParser {
    sub_commands: HashMap<String, ArgumentParser>,
    required: HashSet<String>,
    positional: Vec<Argument>,
    named: HashMap<String, Argument>,
    keys: HashMap<String, String>,
}

impl ArgumentParser {
    pub fn builder() -> ArgumentParserBuilder {
        return ArgumentParserBuilder::new();
    }

    pub fn parse(&self, input: &str) -> Result<ParsedArgs, InputError> {
        let mut args: VecDeque<String> = match shell_words::split(input) {
            Err(e) => return Err(InputError::new(&format!("Could not parse input into valid argments: {}", e))),
            Ok(a) => VecDeque::from_iter(a.into_iter())
        };
        match args.pop_front() {
            Some(s) => {
                if self.sub_commands.contains_key(&s) {
                    let remaining_input = shell_words::join(args);
                    return Ok(ParsedArgs::SubCommand(SubCommand {
                        name: String::from(&s),
                        args: match self.sub_commands.get(&s).unwrap().parse(&remaining_input) {
                            Ok(args) => Box::new(args),
                            Err(e) => return Err(e),
                        }
                    }));
                } else {
                    args.push_front(s);
                }
            },
            None => return Ok(ParsedArgs::Arguments(Default::default()))
        };
        let mut positional_queue = VecDeque::from_iter(self.positional.iter());
        let mut required_fields: HashSet<&String> = HashSet::from_iter(self.required.iter());
        let mut return_args: HashMap<String, String> = HashMap::new();
        let mut return_nargs: HashMap<String, Vec<String>> = HashMap::new();
        let mut return_flags: HashSet<String> = HashSet::new();
        while !positional_queue.is_empty() {
            let parg = positional_queue.pop_front().unwrap();
            match args.pop_front() {
                None => break,
                Some(mut arg) => {
                    if !parg.is_narg {
                        if parg.is_required {
                            required_fields.remove(&parg.name);
                            return_args.insert(String::from(&parg.name), arg);
                        } else {
                            if self.keys.contains_key(&arg) {
                                break;
                            } else {
                                return_args.insert(String::from(&parg.name), arg);
                            }
                        }
                    } else {
                        if parg.is_required || !self.keys.contains_key(&arg) {
                            let mut arg_vec: Vec<String> = Vec::new();
                            loop {
                                arg_vec.push(arg);
                                if args.is_empty() {
                                    break;
                                }
                                arg = args.pop_front().unwrap();
                                if self.keys.contains_key(&arg) {
                                    break;
                                }
                            }
                            if parg.is_required {
                                required_fields.remove(&parg.name);
                            }
                            return_nargs.insert(String::from(&parg.name), arg_vec);
                        }
                    }
                }
            }
        }
        if !positional_queue.is_empty() && positional_queue.pop_front().unwrap().is_required {
            return Err(InputError::new("Some required positional arguments were not provided."))
        }
        while !args.is_empty() {
            let arg = args.pop_front().unwrap();
            if !self.keys.contains_key(&arg) {
                return Err(InputError::new(&format!("Unexpected argument encountered: {}.", arg)));
            }
            let name = self.keys.get(&arg).unwrap();
            match self.named.get(name) {
                None => return Err(InputError::new(&format!("Could not find a known argument matching key {}.", arg))),
                Some(named_arg) => {
                    if named_arg.is_flag {
                        return_flags.insert(String::from(&named_arg.name));
                    } else {
                        if named_arg.is_required {
                            required_fields.remove(&named_arg.name);
                        }
                        if !named_arg.is_narg {
                            match args.pop_front() {
                                None => return Err(InputError::new(&format!("No value was specified for named arg {}.", named_arg.name))),
                                Some(val) => return_args.insert(String::from(&named_arg.name), val)
                            };
                        } else {
                            let mut arg_vec: Vec<String> = Vec::new();
                            let mut did_one = false;
                            loop {
                                match args.pop_front() {
                                    None => break,
                                    Some(val) => {
                                        if did_one && self.keys.contains_key(&val) {
                                            break;
                                        } else {
                                            arg_vec.push(val);
                                            did_one = true;
                                        }
                                    }
                                };
                                if args.is_empty() {
                                    break;
                                }
                            }
                            if arg_vec.is_empty() {
                                return Err(InputError::new(&format!("No value was specified for named arg {}.", named_arg.name)));
                            }
                            return_nargs.insert(String::from(&named_arg.name), arg_vec);
                        }
                    }
                }
            };
        }
        if !required_fields.is_empty() {
            return Err(InputError::new("Some required arguments were not provided."))
        }
        return Ok(ParsedArgs::Arguments(Arguments {
            args: HashMap::from_iter(return_args.iter().map(|(k,v)| (String::from(k), String::from(v)))),
            nargs: HashMap::from_iter(return_nargs.iter().map(|(k,v)| (String::from(k), Vec::from_iter(v.iter().map(|x| String::from(x)))))),
            flags: HashSet::from_iter(return_flags.iter().map(|x| String::from(x))),
        }));
    }
}
