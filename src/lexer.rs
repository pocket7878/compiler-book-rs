mod local_var_env;
mod node;

pub use node::{Node, NodeKind};

use crate::tokenizer::{TokenKind, TokenList};

use self::local_var_env::LocalVarEnvironment;

pub struct Lexer<'a> {
    token_list: TokenList<'a>,
    local_var_environment: LocalVarEnvironment,
}

impl<'a> Lexer<'a> {
    pub fn new(token_list: TokenList<'a>) -> Lexer<'a> {
        Self {
            token_list,
            local_var_environment: LocalVarEnvironment::new(),
        }
    }

    /* Lexing Programs */
    pub fn program(&mut self) -> Vec<Node> {
        let mut nodes = vec![];
        while !self.token_list.at_end() {
            nodes.push(self.stmt());
        }

        nodes
    }

    fn stmt(&mut self) -> Node {
        let node = self.expr();
        self.token_list.expect_kind(&TokenKind::Semicolon);

        node
    }

    fn expr(&mut self) -> Node {
        self.assign()
    }

    fn assign(&mut self) -> Node {
        let mut node = self.equality();
        if self.token_list.try_consume(&TokenKind::Assign).is_some() {
            node = Node::new_bin_op_node(NodeKind::Assign, Box::new(node), Box::new(self.assign()));
        }

        node
    }

    fn equality(&mut self) -> Node {
        let mut node = self.relational();

        loop {
            if self.token_list.try_consume(&TokenKind::Equal).is_some() {
                node = Node::new_bin_op_node(
                    NodeKind::Equal,
                    Box::new(node),
                    Box::new(self.relational()),
                );
            } else if self.token_list.try_consume(&TokenKind::NotEqual).is_some() {
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
            if self.token_list.try_consume(&TokenKind::LessThan).is_some() {
                node =
                    Node::new_bin_op_node(NodeKind::LessThan, Box::new(node), Box::new(self.add()));
            } else if self
                .token_list
                .try_consume(&TokenKind::LessThanOrEqual)
                .is_some()
            {
                node = Node::new_bin_op_node(
                    NodeKind::LessThanOrEqual,
                    Box::new(node),
                    Box::new(self.add()),
                );
            } else if self
                .token_list
                .try_consume(&TokenKind::GreaterThan)
                .is_some()
            {
                node =
                    Node::new_bin_op_node(NodeKind::LessThan, Box::new(self.add()), Box::new(node));
            } else if self
                .token_list
                .try_consume(&TokenKind::GreaterThanOrEqual)
                .is_some()
            {
                node = Node::new_bin_op_node(
                    NodeKind::LessThanOrEqual,
                    Box::new(self.add()),
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
            if self.token_list.try_consume(&TokenKind::Plus).is_some() {
                node = Node::new_bin_op_node(NodeKind::Add, Box::new(node), Box::new(self.mul()));
            } else if self.token_list.try_consume(&TokenKind::Minus).is_some() {
                node = Node::new_bin_op_node(NodeKind::Sub, Box::new(node), Box::new(self.mul()));
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Node {
        let mut node = self.unary();
        loop {
            if self.token_list.try_consume(&TokenKind::Mul).is_some() {
                node = Node::new_bin_op_node(NodeKind::Mul, Box::new(node), Box::new(self.unary()));
            } else if self.token_list.try_consume(&TokenKind::Div).is_some() {
                node = Node::new_bin_op_node(NodeKind::Div, Box::new(node), Box::new(self.unary()));
            } else {
                return node;
            }
        }
    }

    fn unary(&mut self) -> Node {
        if self.token_list.try_consume(&TokenKind::Plus).is_some() {
            return self.primary();
        }
        if self.token_list.try_consume(&TokenKind::Minus).is_some() {
            return Node::new_bin_op_node(
                NodeKind::Sub,
                Box::new(Node::new_num(0)),
                Box::new(self.primary()),
            );
        }
        self.primary()
    }

    fn primary(&mut self) -> Node {
        if self.token_list.try_consume(&TokenKind::LParen).is_some() {
            let node = self.expr();
            self.token_list.expect_kind(&TokenKind::RParen);
            return node;
        } else if let Some(ident_tok) = self.token_list.try_consume(&TokenKind::Ident) {
            let ident_name = ident_tok.str.unwrap();
            if let Some(offset) = self.local_var_environment.variable_offset(&ident_name) {
                return Node::new_local_var(*offset);
            } else {
                let offset = self.local_var_environment.intern_new_variable(&ident_name);
                return Node::new_local_var(offset);
            }
        }

        let n = self.token_list.expect_num();
        Node::new_num(n)
    }
}
