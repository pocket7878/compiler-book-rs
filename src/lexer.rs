mod local_var_env;
mod node;

pub use node::{BinOpType, Node};

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
        if self.token_list.try_consume(&TokenKind::Return).is_some() {
            let return_value = self.expr();
            self.token_list.expect_kind(&TokenKind::Semicolon);

            Node::Return(Box::new(return_value))
        } else if self.token_list.try_consume(&TokenKind::If).is_some() {
            self.token_list.expect_kind(&TokenKind::LParen);
            let condition = self.expr();
            self.token_list.expect_kind(&TokenKind::RParen);
            let then_body = self.stmt();

            if self.token_list.try_consume(&TokenKind::Else).is_some() {
                let else_body = self.stmt();

                return Node::If(
                    Box::new(condition),
                    Box::new(then_body),
                    Some(Box::new(else_body)),
                );
            }

            Node::If(Box::new(condition), Box::new(then_body), None)
        } else if self.token_list.try_consume(&TokenKind::While).is_some() {
            self.token_list.expect_kind(&TokenKind::LParen);
            let condition = self.expr();
            self.token_list.expect_kind(&TokenKind::RParen);
            let body = self.stmt();

            Node::While(Box::new(condition), Box::new(body))
        } else if self.token_list.try_consume(&TokenKind::For).is_some() {
            // forの後には、for (初期化; 条件; 更新) 本体
            // ただし、初期化, 条件, 更新はどれも省略可能
            self.token_list.expect_kind(&TokenKind::LParen);
            let init = if self.token_list.try_consume(&TokenKind::Semicolon).is_some() {
                None
            } else {
                let node = self.expr();
                self.token_list.expect_kind(&TokenKind::Semicolon);

                Some(Box::new(node))
            };
            let check = if self.token_list.try_consume(&TokenKind::Semicolon).is_some() {
                None
            } else {
                let node = self.expr();
                self.token_list.expect_kind(&TokenKind::Semicolon);

                Some(Box::new(node))
            };
            let update = if self.token_list.try_consume(&TokenKind::RParen).is_some() {
                None
            } else {
                let node = self.expr();
                self.token_list.expect_kind(&TokenKind::RParen);

                Some(Box::new(node))
            };

            let body = self.stmt();
            Node::For(init, check, update, Box::new(body))
        } else if self.token_list.try_consume(&TokenKind::LBrace).is_some() {
            let mut stmts = vec![];
            while self.token_list.try_consume(&TokenKind::RBrace).is_none() {
                stmts.push(self.stmt());
            }

            Node::Block(stmts)
        } else {
            let expr = self.expr();
            self.token_list.expect_kind(&TokenKind::Semicolon);

            expr
        }
    }

    fn expr(&mut self) -> Node {
        self.assign()
    }

    fn assign(&mut self) -> Node {
        let mut node = self.equality();
        if self.token_list.try_consume(&TokenKind::Assign).is_some() {
            node = Node::Assign(Box::new(node), Box::new(self.assign()));
        }

        node
    }

    fn equality(&mut self) -> Node {
        let mut node = self.relational();

        loop {
            if self.token_list.try_consume(&TokenKind::Equal).is_some() {
                node = Node::BinOp(
                    BinOpType::Equal,
                    Box::new(node),
                    Box::new(self.relational()),
                );
            } else if self.token_list.try_consume(&TokenKind::NotEqual).is_some() {
                node = Node::BinOp(
                    BinOpType::NotEqual,
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
                node = Node::BinOp(BinOpType::LessThan, Box::new(node), Box::new(self.add()));
            } else if self
                .token_list
                .try_consume(&TokenKind::LessThanOrEqual)
                .is_some()
            {
                node = Node::BinOp(
                    BinOpType::LessThanOrEqual,
                    Box::new(node),
                    Box::new(self.add()),
                );
            } else if self
                .token_list
                .try_consume(&TokenKind::GreaterThan)
                .is_some()
            {
                node = Node::BinOp(BinOpType::LessThan, Box::new(self.add()), Box::new(node));
            } else if self
                .token_list
                .try_consume(&TokenKind::GreaterThanOrEqual)
                .is_some()
            {
                node = Node::BinOp(
                    BinOpType::LessThanOrEqual,
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
                node = Node::BinOp(BinOpType::Add, Box::new(node), Box::new(self.mul()));
            } else if self.token_list.try_consume(&TokenKind::Minus).is_some() {
                node = Node::BinOp(BinOpType::Sub, Box::new(node), Box::new(self.mul()));
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Node {
        let mut node = self.unary();
        loop {
            if self.token_list.try_consume(&TokenKind::Mul).is_some() {
                node = Node::BinOp(BinOpType::Mul, Box::new(node), Box::new(self.unary()));
            } else if self.token_list.try_consume(&TokenKind::Div).is_some() {
                node = Node::BinOp(BinOpType::Div, Box::new(node), Box::new(self.unary()));
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
            return Node::BinOp(
                BinOpType::Sub,
                Box::new(Node::Num(0)),
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
            if self.token_list.try_consume(&TokenKind::LParen).is_some() {
                self.token_list.expect_kind(&TokenKind::RParen);
                return Node::Funcall(ident_name);
            } else if let Some(offset) = self.local_var_environment.variable_offset(&ident_name) {
                return Node::LocalVar(*offset);
            } else {
                let offset = self.local_var_environment.intern_new_variable(&ident_name);
                return Node::LocalVar(offset);
            }
        }

        let n = self.token_list.expect_num();
        Node::Num(n)
    }
}
