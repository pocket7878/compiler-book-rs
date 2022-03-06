use std::collections::HashMap;

#[derive(Clone)]
pub struct LocalVarEnvironment {
    offset: i32,
    variables: HashMap<String, i32>,
}

impl LocalVarEnvironment {
    pub fn new() -> Self {
        Self {
            offset: 0,
            variables: HashMap::new(),
        }
    }

    pub fn new_with_base_offset(base_offset: i32) -> Self {
        Self {
            offset: base_offset,
            variables: HashMap::new(),
        }
    }

    pub fn intern(&mut self, name: &str) -> i32 {
        if let Some(offset) = self.variables.get(name) {
            *offset
        } else {
            let var_offset = self.offset;
            self.variables.insert(name.to_string(), var_offset);
            self.offset += 16;

            var_offset
        }
    }
}
