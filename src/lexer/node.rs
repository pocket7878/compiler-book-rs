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
    LocalVar(i32),
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
}
