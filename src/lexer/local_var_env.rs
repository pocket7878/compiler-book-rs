use std::collections::HashMap;

pub struct LocalVarEnvironment {
    offset: i32,
    variables: HashMap<String, i32>,
}

impl LocalVarEnvironment {
    pub fn new() -> Self {
        Self {
            offset: 16,
            variables: HashMap::new(),
        }
    }

    pub fn intern_new_variable(&mut self, name: &str) -> i32 {
        let var_offset = self.offset;
        self.variables.insert(name.to_string(), var_offset);
        self.offset += 16;

        var_offset
    }

    pub fn variable_offset(&self, name: &str) -> Option<&i32> {
        self.variables.get(name)
    }
}
