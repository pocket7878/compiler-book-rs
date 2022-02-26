use crate::tokenizer::{TokenKind, TokenList};

#[derive(PartialEq, Eq, Debug)]
pub enum NodeKind {
    Add,
    Sub,
    Mul,
    Div,
    Num,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub val: Option<i32>, // Only for Num NodeKind
}

impl Node {
    pub fn new_bin_op_node(kind: NodeKind, lhs: Box<Node>, rhs: Box<Node>) -> Self {
        Self {
            kind,
            lhs: Some(lhs),
            rhs: Some(rhs),
            val: None,
        }
    }

    pub fn new_num(val: i32) -> Self {
        Self {
            kind: NodeKind::Num,
            lhs: None,
            rhs: None,
            val: Some(val),
        }
    }
}

pub struct Lexer<'a> {
    token_list: TokenList<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(token_list: TokenList<'a>) -> Lexer<'a> {
        Self { token_list }
    }

    pub fn expr(&mut self) -> Node {
        let mut node = self.mul();

        loop {
            if self.token_list.try_consume(&TokenKind::Plus) {
                node = Node::new_bin_op_node(NodeKind::Add, Box::new(node), Box::new(self.mul()));
            } else if self.token_list.try_consume(&TokenKind::Minus) {
                node = Node::new_bin_op_node(NodeKind::Sub, Box::new(node), Box::new(self.mul()));
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Node {
        let mut node = self.primary();
        loop {
            if self.token_list.try_consume(&TokenKind::Mul) {
                node =
                    Node::new_bin_op_node(NodeKind::Mul, Box::new(node), Box::new(self.primary()));
            } else if self.token_list.try_consume(&TokenKind::Div) {
                node =
                    Node::new_bin_op_node(NodeKind::Div, Box::new(node), Box::new(self.primary()));
            } else {
                return node;
            }
        }
    }

    fn primary(&mut self) -> Node {
        if self.token_list.try_consume(&TokenKind::LParen) {
            let node = self.expr();
            self.token_list.expect_kind(&TokenKind::RParen);
            return node;
        }

        let n = self.token_list.expect_num();
        Node::new_num(n)
    }
}
