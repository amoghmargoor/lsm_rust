struct SetCommand {
    key: String,
    value: String
}

struct RmCommand {
    key: String,
    value: String
}

trait Command {
    fn getKey(&self) -> String;
    fn getValue(&self) -> String;
}

impl Command for SetCommand {
    fn getKey(&self) -> String {
        return &self.key;        
    }
    fn getValue(&self) -> String {
        return &self.value;
    }
}

impl Command for RmCommand {
    fn getKey(&self) -> String {
        return &self.key;        
    }
    fn getValue(&self) -> String {
        return &self.value;
    }
}

