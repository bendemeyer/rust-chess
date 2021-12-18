use std::collections::{HashSet, HashMap, VecDeque};

use crate::util::errors::InputError;


enum ArgumentTypes {
    Positional,
    PositionalN,
    Named,
    NamedN,
    Flag,
}


#[derive(Clone)]
struct Argument {
    pub name: String,
    pub is_required: bool,
    pub keys: HashSet<String>,
    pub is_narg: bool,
    pub is_flag: bool,
}


pub struct ParsedArgs {
    args: HashMap<String, String>,
    nargs: HashMap<String, Vec<String>>,
    flags: HashSet<String>,
}

impl ParsedArgs {
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

    pub fn add_positional_arg(&mut self, name: &str, required: bool, narg: bool) -> Result<(), InputError> {
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
        return Ok(());
    }

    pub fn add_named_arg(&mut self, name: &str, keys: HashSet<&str>, required: bool, narg: bool) -> Result<(), InputError> {
        if keys.is_empty() {
            return Err(InputError::new("Named arguments must supply at least one key."))
        }
        if self.key_set.is_disjoint(&HashSet::from_iter(keys.iter().map(|x| String::from(*x) ))) {
            return Err(InputError::new("Some of the provided keys already belong to other arguments."))
        }
        self.named.insert(String::from(name), Argument {
            name: String::from(name),
            is_required: required,
            keys: HashSet::from_iter(keys.iter().map(|x| { String::from(*x) })),
            is_narg: narg,
            is_flag: false,
        });
        self.key_set.extend(keys.iter().map(|x| { String::from(*x) }));
        if required {
            self.required.insert(String::from(name));
        }
        return Ok(());
    }

    pub fn add_flag_arg(&mut self, name: &str, keys: HashSet<&str>) -> Result<(), InputError> {
        if keys.is_empty() {
            return Err(InputError::new("Flag arguments must supply at least one key."))
        }
        if self.key_set.is_disjoint(&HashSet::from_iter(keys.iter().map(|x| String::from(*x) ))) {
            return Err(InputError::new("Some of the provided keys already belong to other arguments."))
        }
        self.named.insert(String::from(name), Argument {
            name: String::from(name),
            is_required: false,
            keys: HashSet::from_iter(keys.iter().map(|x| { String::from(*x) })),
            is_narg: false,
            is_flag: true,
        });
        self.key_set.extend(keys.iter().map(|x| { String::from(*x) }));
        return Ok(());
    }

    pub fn build(&self) -> ArgumentParser {
        return ArgumentParser {
            required: self.required.clone(),
            positional: self.positional.clone(),
            named: self.named.clone(),
            keys: self.keys.clone(),
        }
    }
}


pub struct ArgumentParser {
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
        let mut positional_queue = VecDeque::from_iter(self.positional.iter());
        let mut required_fields: HashSet<&String> = HashSet::from_iter(self.required.iter());
        let mut return_args: HashMap<String, String> = HashMap::new();
        let mut return_nargs: HashMap<String, Vec<String>> = HashMap::new();
        let mut return_flags: HashSet<String> = HashSet::new();
        let mut args= VecDeque::from_iter(input.split(" ").map(|x| { String::from(x) }));
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
        return Ok(ParsedArgs{
            args: HashMap::from_iter(return_args.iter().map(|(k,v)| (String::from(k), String::from(v)))),
            nargs: HashMap::from_iter(return_nargs.iter().map(|(k,v)| (String::from(k), Vec::from_iter(v.iter().map(|x| String::from(x)))))),
            flags: HashSet::from_iter(return_flags.iter().map(|x| String::from(x))),
        });
    }
}
