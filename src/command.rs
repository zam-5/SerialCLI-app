use crate::communicator::Communicator;

pub struct Command {
    pub name: String,
    exec_func: fn(&Vec<String>, &mut Communicator),
}

impl Command {
    pub fn new(name: &str, exec_func: fn(&Vec<String>, &mut Communicator)) -> Self {
        Self {
            name: name.to_string(),
            exec_func,
        }
    }

    pub fn exec(&self, argv: &Vec<String>, communicator: &mut Communicator) {
        (self.exec_func)(argv, communicator);
    }
}
