use crate::tokenizer::{TokenKind, TokenList};

#[derive(PartialEq, Eq, Debug)]
pub enum NodeKind {
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
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
        self.equality()
    }

    fn equality(&mut self) -> Node {
        let mut node = self.relational();

        loop {
            if self.token_list.try_consume(&TokenKind::Equal) {
                node = Node::new_bin_op_node(
                    NodeKind::Equal,
                    Box::new(node),
                    Box::new(self.relational()),
                );
            } else if self.token_list.try_consume(&TokenKind::NotEqual) {
                node = Node::new_bin_op_node(
                    NodeKind::NotEqual,
                    Box::new(node),
                    Box::new(self.relational()),
                );
            } else {
                return node;
            }
        }
    }

    fn relational(&mut self) -> Node {
        let mut node = self.add();

        loop {
            if self.token_list.try_consume(&TokenKind::LessThan) {
                node = Node::new_bin_op_node(
                    NodeKind::LessThan,
                    Box::new(node),
                    Box::new(self.relational()),
                );
            } else if self.token_list.try_consume(&TokenKind::LessThanOrEqual) {
                node = Node::new_bin_op_node(
                    NodeKind::LessThanOrEqual,
                    Box::new(node),
                    Box::new(self.relational()),
                );
            } else if self.token_list.try_consume(&TokenKind::GreaterThan) {
                node = Node::new_bin_op_node(
                    NodeKind::LessThan,
                    Box::new(self.relational()),
                    Box::new(node),
                );
            } else if self.token_list.try_consume(&TokenKind::GreaterThanOrEqual) {
                node = Node::new_bin_op_node(
                    NodeKind::LessThanOrEqual,
                    Box::new(self.relational()),
                    Box::new(node),
                );
            } else {
                return node;
            }
        }
    }

    fn add(&mut self) -> Node {
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
        let mut node = self.unary();
        loop {
            if self.token_list.try_consume(&TokenKind::Mul) {
                node = Node::new_bin_op_node(NodeKind::Mul, Box::new(node), Box::new(self.unary()));
            } else if self.token_list.try_consume(&TokenKind::Div) {
                node = Node::new_bin_op_node(NodeKind::Div, Box::new(node), Box::new(self.unary()));
            } else {
                return node;
            }
        }
    }

    fn unary(&mut self) -> Node {
        if self.token_list.try_consume(&TokenKind::Plus) {
            return self.primary();
        }
        if self.token_list.try_consume(&TokenKind::Minus) {
            return Node::new_bin_op_node(
                NodeKind::Sub,
                Box::new(Node::new_num(0)),
                Box::new(self.primary()),
            );
        }
        return self.primary();
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
