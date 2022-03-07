use super::var_type::VarType;

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
pub enum Node {
    BinOp(BinOpType, Box<Node>, Box<Node>),
    Assign(Box<Node>, Box<Node>),
    LocalVar(String, Option<i32>),
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
        args: Vec<(VarType, String)>,
        body: Vec<Node>,
        stack_size: i32,
    },
    Addr(Box<Node>),
    Deref(Box<Node>),
    VarDef(String, VarType),
}
