use super::ty::Ty;
use std::collections::HashMap;

const STACK_ALIGNMENT: i32 = 16;

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
        self.offset - STACK_ALIGNMENT
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
                ty: ty.clone(),
                offset: var_offset,
            };
            self.variables.insert(name.to_string(), var_info.clone());
            self.offset += align_to_stack(var_offset + ty.size(), STACK_ALIGNMENT);
            var_info
        }
    }
}

fn align_to_stack(v: i32, alignment: i32) -> i32 {
    let result = v / alignment * alignment;
    if v % alignment != 0 {
        result + alignment
    } else {
        result
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_align_to_stack() {
        assert_eq!(16, super::align_to_stack(1, 16));
        assert_eq!(16, super::align_to_stack(16, 16));
        assert_eq!(32, super::align_to_stack(17, 16));
    }
}
