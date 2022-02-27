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
}

#[derive(PartialEq, Eq, Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub val: Option<i32>,    // Only for Num NodeKind
    pub offset: Option<i32>, // Only for LocalVar NodeKind
}

impl Node {
    pub fn new_bin_op_node(kind: NodeKind, lhs: Box<Node>, rhs: Box<Node>) -> Self {
        Self {
            kind,
            lhs: Some(lhs),
            rhs: Some(rhs),
            val: None,
            offset: None,
        }
    }

    pub fn new_num(val: i32) -> Self {
        Self {
            kind: NodeKind::Num,
            lhs: None,
            rhs: None,
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
            offset: Some(offset),
        }
    }
}
