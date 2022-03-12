use super::ty::Ty;
use std::collections::HashMap;

#[derive(Clone)]
pub struct VarInfo {
    pub ty: Ty,
    pub offset: i32,
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

    pub fn stack_size(&self) -> i32 {
        self.offset - 16
    }

    pub fn get_var_info(&self, var_name: &str) -> Option<&VarInfo> {
        self.variables.get(var_name)
    }

    pub fn intern(&mut self, name: &str, ty: Ty) -> VarInfo {
        if let Some(var_info) = self.variables.get(name) {
            var_info.clone()
        } else {
            let var_offset = self.offset;
            let var_info = VarInfo {
                ty,
                offset: var_offset,
            };
            self.variables.insert(name.to_string(), var_info.clone());
            self.offset += 16;
            var_info
        }
    }
}
