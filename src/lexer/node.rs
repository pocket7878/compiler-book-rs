#[derive(PartialEq, Eq, Debug)]
pub enum NodeKind {
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    NotEqual,
    Assign,
    LessThan,
    LessThanOrEqual,
    LocalVar,
    Num,
    Return,
    If,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub else_body: Option<Box<Node>>, // Only for If
    pub val: Option<i32>,             // Only for Num NodeKind
    pub offset: Option<i32>,          // Only for LocalVar NodeKind
}

impl Node {
    pub fn new_bin_op_node(kind: NodeKind, lhs: Box<Node>, rhs: Box<Node>) -> Self {
        Self {
            kind,
            lhs: Some(lhs),
            rhs: Some(rhs),
            val: None,
            else_body: None,
            offset: None,
        }
    }

    pub fn new_num(val: i32) -> Self {
        Self {
            kind: NodeKind::Num,
            lhs: None,
            rhs: None,
            else_body: None,
            val: Some(val),
            offset: None,
        }
    }

    pub fn new_local_var(offset: i32) -> Self {
        Self {
            kind: NodeKind::LocalVar,
            lhs: None,
            rhs: None,
            val: None,
            else_body: None,
            offset: Some(offset),
        }
    }

    pub fn new_return(expr: Box<Node>) -> Self {
        Self {
            kind: NodeKind::Return,
            lhs: Some(expr),
            rhs: None,
            val: None,
            else_body: None,
            offset: None,
        }
    }

    pub fn new_if(
        condition: Box<Node>,
        then_body: Box<Node>,
        else_body: Option<Box<Node>>,
    ) -> Self {
        Self {
            kind: NodeKind::If,
            lhs: Some(condition),
            rhs: Some(then_body),
            else_body: else_body,
            val: None,
            offset: None,
        }
    }
}
