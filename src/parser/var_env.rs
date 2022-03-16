use super::ty::Ty;
use std::collections::HashMap;

const STACK_ALIGNMENT: i32 = 16;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct LocalVarInfo {
    pub ty: Ty,
    pub offset: i32,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct GlobalVarInfo {
    pub ty: Ty,
    pub label: String,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum VarInfo {
    Local(LocalVarInfo),
    Global(GlobalVarInfo),
}

#[derive(Clone)]
pub struct VarEnvironment {
    stack_offset: i32,
    local_variables: HashMap<String, LocalVarInfo>,
    global_variables: HashMap<String, GlobalVarInfo>,
}

impl VarEnvironment {
    pub fn new() -> Self {
        Self {
            stack_offset: 16,
            local_variables: HashMap::new(),
            global_variables: HashMap::new(),
        }
    }

    // スタックのアラインメントに現在のoffsetをアラインした数を返す
    // たとえば5なら16, 16なら16, 17なら32
    pub fn stack_size(&self) -> i32 {
        (self.stack_offset + STACK_ALIGNMENT - 1) / STACK_ALIGNMENT * STACK_ALIGNMENT
    }

    pub fn add_local_var(&mut self, name: &str, ty: Ty) -> LocalVarInfo {
        if let Some(var_info) = self.local_variables.get(name) {
            var_info.clone()
        } else {
            let size = ty.size();
            self.stack_offset += size;
            let var_info = LocalVarInfo {
                ty,
                offset: self.stack_offset,
            };
            self.local_variables
                .insert(name.to_string(), var_info.clone());
            var_info
        }
    }

    pub fn add_global_var(&mut self, name: &str, ty: Ty) -> GlobalVarInfo {
        if let Some(var_info) = self.global_variables.get(name) {
            var_info.clone()
        } else {
            let var_info = GlobalVarInfo {
                ty,
                label: name.to_owned(),
            };
            self.global_variables
                .insert(name.to_string(), var_info.clone());
            var_info
        }
    }

    // 変数の名前を解決する
    // ローカル変数は同名のグローバル変数をシャドーイングするので、まずはローカルの変数を探す
    pub fn resolve(&self, name: &str) -> Option<VarInfo> {
        if let Some(local_var_info) = self.local_variables.get(name) {
            Some(VarInfo::Local(local_var_info.clone()))
        } else {
            self.global_variables
                .get(name)
                .map(|global_var_info| VarInfo::Global(global_var_info.clone()))
        }
    }

    pub fn new_local_scope(&self) -> Self {
        Self {
            stack_offset: 16,
            local_variables: HashMap::new(),
            global_variables: self.global_variables.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{GlobalVarInfo, LocalVarInfo, VarEnvironment, VarInfo};
    use crate::parser::Ty;

    #[test]
    fn add_local_var() {
        let mut var_env = VarEnvironment::new();
        assert_eq!(
            var_env.add_local_var("x", Ty::Int),
            LocalVarInfo {
                ty: Ty::Int,
                offset: 20,
            }
        );
    }

    #[test]
    fn add_global_var() {
        let mut var_env = VarEnvironment::new();
        assert_eq!(
            var_env.add_global_var("x", Ty::Int),
            GlobalVarInfo {
                ty: Ty::Int,
                label: "x".to_owned(),
            }
        );
    }

    #[test]
    fn resolve_local_var() {
        let mut var_env = VarEnvironment::new();
        var_env.add_local_var("x", Ty::Int);
        assert_eq!(
            var_env.resolve("x"),
            Some(VarInfo::Local(LocalVarInfo {
                ty: Ty::Int,
                offset: 20,
            }))
        );
    }

    #[test]
    fn resolve_global_var() {
        let mut var_env = VarEnvironment::new();
        var_env.add_global_var("x", Ty::Int);
        assert_eq!(
            var_env.resolve("x"),
            Some(VarInfo::Global(GlobalVarInfo {
                ty: Ty::Int,
                label: "x".to_owned(),
            }))
        );
    }

    #[test]
    fn lobal_var_shadowing_global_var() {
        let mut var_env = VarEnvironment::new();
        var_env.add_global_var("x", Ty::Int);
        var_env.add_local_var("x", Ty::Int);
        assert_eq!(
            var_env.resolve("x"),
            Some(VarInfo::Local(LocalVarInfo {
                ty: Ty::Int,
                offset: 20,
            }))
        );
    }
}
