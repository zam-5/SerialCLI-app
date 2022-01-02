pub struct Command {
    pub name: String,
    exec_func: fn(&Vec<String>),
}

impl Command {
    pub fn new(name: &str, exec_func: fn(&Vec<String>)) -> Self {
        Self {
            name: name.to_string(),
            exec_func,
        }
    }

    pub fn exec(&self, argv: &Vec<String>) {
        (self.exec_func)(argv);
    }
}
