#[derive(Clone, PartialEq, Eq, Debug)]
pub enum VarType {
    Int,
    Ptr(Box<VarType>),
}

impl VarType {
    pub fn size(&self) -> i32 {
        match self {
            VarType::Int => 4,
            VarType::Ptr(_) => 8,
        }
    }
}
