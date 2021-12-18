use std::fmt;

pub struct InputError {
    pub msg: String,
}

impl InputError {
    pub fn new(msg: &str) -> InputError {
        return InputError {
            msg: String::from(msg)
        }
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid input: {}", self.msg)
    }
}

impl fmt::Debug for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid input: {}", self.msg)
    }
}


pub struct ValueError {
    pub msg: String,
}

impl ValueError {
    pub fn new(msg: &str) -> ValueError {
        return ValueError {
            msg: String::from(msg)
        }
    }
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid argument: {}", self.msg)
    }
}

impl fmt::Debug for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid argument: {}", self.msg)
    }
}