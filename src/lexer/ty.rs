#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Ty {
    Int,
    Ptr(Box<Ty>),
    Array(Box<Ty>, i32),
}

impl Ty {
    pub fn size(&self) -> i32 {
        match self {
            Ty::Int => 4,
            Ty::Ptr(_) => 8,
            Ty::Array(ty, len) => {
                let ty_size = ty.size();
                ty_size * len
            }
        }
    }

    pub fn deref_type(&self) -> Ty {
        match self {
            Ty::Ptr(ty) => *ty.clone(),
            _ => panic!("deref_type called on non-pointer type: {:?}", self),
        }
    }
}
