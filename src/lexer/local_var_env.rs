use super::var_type::VarType;
use std::collections::HashMap;

#[derive(Clone)]
struct VarInfo {
    ty: VarType,
    offset: i32,
}

#[derive(Clone)]
pub struct LocalVarEnvironment {
    offset: i32,
    variables: HashMap<String, VarInfo>,
}

impl LocalVarEnvironment {
    pub fn new_with_base_offset(base_offset: i32) -> Self {
        Self {
            offset: base_offset,
            variables: HashMap::new(),
        }
    }

    pub fn is_interned(&self, var_name: &str) -> bool {
        self.variables.contains_key(var_name)
    }

    pub fn intern(&mut self, name: &str, ty: VarType) -> i32 {
        if let Some(var_info) = self.variables.get(name) {
            var_info.offset
        } else {
            let var_offset = self.offset;
            self.variables.insert(
                name.to_string(),
                VarInfo {
                    ty: ty,
                    offset: var_offset,
                },
            );
            self.offset += 16;

            var_offset
        }
    }
}
