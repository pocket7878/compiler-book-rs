use super::ty::Ty;

#[derive(PartialEq, Eq, Debug)]

pub enum BinOpType {
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Ast {
    BinOp(BinOpType, Box<Node>, Box<Node>),
    Assign(Box<Node>, Box<Node>),
    LocalVar {
        name: String,
        offset: i32,
    },
    Num(i32),
    Return(Box<Node>),
    If(Box<Node>, Box<Node>, Option<Box<Node>>),
    While(Box<Node>, Box<Node>),
    For(
        Option<Box<Node>>,
        Option<Box<Node>>,
        Option<Box<Node>>,
        Box<Node>,
    ),
    Block(Vec<Node>),
    Funcall(String, Vec<Node>),
    Fundef {
        name: String,
        args: Vec<Node>,
        body: Vec<Node>,
        stack_size: i32,
    },
    Addr(Box<Node>),
    Deref(Box<Node>),
    VarDef(String, Ty),
}

#[derive(PartialEq, Eq, Debug)]
pub struct Node {
    pub ast: Ast,
    pub ty: Option<Ty>,
}

impl Node {
    pub fn new(ast: Ast, ty: Option<Ty>) -> Self {
        Self { ast, ty }
    }
}
