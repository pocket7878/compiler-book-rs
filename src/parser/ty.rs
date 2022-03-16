#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Ty {
    Int,
    Char,
    Ptr(Box<Ty>),
    Array(Box<Ty>, i32),
}

impl Ty {
    pub fn size(&self) -> i32 {
        match self {
            Ty::Char => 1,
            Ty::Int => 4,
            Ty::Ptr(_) => 8,
            Ty::Array(ty, len) => {
                let ty_size = ty.size();
                ty_size * len
            }
        }
    }

    pub fn is_reference_type(&self) -> bool {
        matches!(self, Ty::Ptr(_) | Ty::Array(..))
    }

    pub fn base_ty(&self) -> Ty {
        match self {
            Ty::Ptr(ty) => *ty.clone(),
            Ty::Array(ty, _) => *ty.clone(),
            _ => panic!("{:?} is not refrence type", self),
        }
    }
}
