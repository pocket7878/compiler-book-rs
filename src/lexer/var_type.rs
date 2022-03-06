#[derive(Clone, PartialEq, Eq, Debug)]
pub enum VarType {
    Int,
    Ptr(Box<VarType>),
}
