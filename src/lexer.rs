mod local_var_env;
mod node;
mod var_type;

pub use node::{BinOpType, Node};

use self::local_var_env::LocalVarEnvironment;
use crate::tokenizer::{TokenKind, TokenList};
use var_type::VarType;

pub struct Lexer<'a> {
    token_list: TokenList<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(token_list: TokenList<'a>) -> Lexer<'a> {
        Self { token_list }
    }

    /* Lexing Programs */
    pub fn program(&mut self) -> Vec<Node> {
        let mut nodes = vec![];
        while !self.token_list.at_end() {
            // 関数の本体の変数のoffsetは引数に指定されているかどうかで変化するので、一度読みこんだ後に計算する
            let mut function = self.fundef();
            if let Node::Fundef(_, ref args, ref mut body) = function {
                // スタックのトップには、FPとLRが保存されているので、-16以降が変数領域
                let mut function_scope_local_var_env =
                    LocalVarEnvironment::new_with_base_offset(16);
                for arg in args {
                    function_scope_local_var_env.intern(&arg.1, arg.0.clone());
                }
                for b in body.iter_mut() {
                    self.assign_local_var_offset(b, &mut function_scope_local_var_env)
                }
                nodes.push(function);
            } else {
                unreachable!()
            }
        }

        nodes
    }

    fn assign_local_var_offset(&self, node: &mut Node, local_var_env: &mut LocalVarEnvironment) {
        match node {
            Node::VarDef(name) => {
                local_var_env.intern(name, VarType::Int);
            }
            Node::LocalVar(name, offset) => {
                if offset.is_none() {
                    if local_var_env.is_interned(name) {
                        *offset = Some(local_var_env.intern(name, VarType::Int));
                    } else {
                        panic!("Undefined variable: {}", name);
                    }
                } else {
                    unreachable!()
                }
            }
            Node::BinOp(_, lhs, rhs) => {
                self.assign_local_var_offset(lhs, local_var_env);
                self.assign_local_var_offset(rhs, local_var_env);
            }
            Node::Assign(lhs, rhs) => {
                self.assign_local_var_offset(lhs, local_var_env);
                self.assign_local_var_offset(rhs, local_var_env);
            }
            Node::Num(_) => {}
            Node::Addr(node) => {
                self.assign_local_var_offset(node, local_var_env);
            }
            Node::Deref(node) => {
                self.assign_local_var_offset(node, local_var_env);
            }
            Node::Return(return_value_node) => {
                self.assign_local_var_offset(return_value_node, local_var_env);
            }
            Node::If(condition, then_body, else_body) => {
                self.assign_local_var_offset(condition, local_var_env);
                self.assign_local_var_offset(then_body, local_var_env);
                if let Some(else_body) = else_body {
                    self.assign_local_var_offset(else_body, local_var_env);
                }
            }
            Node::While(condition, body) => {
                self.assign_local_var_offset(condition, local_var_env);
                self.assign_local_var_offset(body, local_var_env);
            }
            Node::For(init, check, update, body) => {
                if let Some(init) = init {
                    self.assign_local_var_offset(init, local_var_env);
                }
                if let Some(check) = check {
                    self.assign_local_var_offset(check, local_var_env);
                }
                if let Some(update) = update {
                    self.assign_local_var_offset(update, local_var_env);
                }
                self.assign_local_var_offset(body, local_var_env);
            }
            Node::Block(stmts) => {
                for stmt in stmts.iter_mut() {
                    self.assign_local_var_offset(stmt, local_var_env);
                }
            }
            Node::Funcall(_, args) => {
                for arg in args.iter_mut() {
                    self.assign_local_var_offset(arg, local_var_env);
                }
            }
            Node::Fundef(_, _, _) => {}
        }
    }

    fn fundef(&mut self) -> Node {
        self.token_list.expect_kind(&TokenKind::Int);
        let fn_name = self.token_list.expect_kind(&TokenKind::Ident).str.unwrap();
        let args = self.fundef_args();
        let body = self.fundef_body();
        Node::Fundef(fn_name, args, body)
    }

    fn fundef_args(&mut self) -> Vec<(VarType, String)> {
        let mut args: Vec<(VarType, String)> = vec![];

        self.token_list.expect_kind(&TokenKind::LParen);
        // 最大6つまでの引数をサポートする
        let mut paren_consumed = false;
        for _ in 1..=6 {
            if self.token_list.try_consume(&TokenKind::RParen).is_none() {
                args.push(self.fundef_arg());
                if self.token_list.try_consume(&TokenKind::RParen).is_none() {
                    self.token_list.expect_kind(&TokenKind::Comma);
                } else {
                    paren_consumed = true;
                    break;
                }
            } else {
                paren_consumed = true;
                break;
            }
        }
        if !paren_consumed {
            self.token_list.expect_kind(&TokenKind::RParen);
        }

        args
    }

    fn fundef_arg(&mut self) -> (VarType, String) {
        self.token_list.expect_kind(&TokenKind::Int);
        let name = self.token_list.expect_kind(&TokenKind::Ident).str.unwrap();

        return (VarType::Int, name);
    }

    fn fundef_body(&mut self) -> Vec<Node> {
        self.token_list.expect_kind(&TokenKind::LBrace);
        let mut stmts = vec![];
        while self.token_list.try_consume(&TokenKind::RBrace).is_none() {
            stmts.push(self.stmt());
        }

        stmts
    }

    fn stmt(&mut self) -> Node {
        if self.token_list.try_consume(&TokenKind::Int).is_some() {
            let ident_name = self.token_list.expect_kind(&TokenKind::Ident).str.unwrap();
            self.token_list.expect_kind(&TokenKind::Semicolon);

            Node::VarDef(ident_name)
        } else if self.token_list.try_consume(&TokenKind::Return).is_some() {
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
            if self.token_list.try_consume(&TokenKind::Star).is_some() {
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
        if self.token_list.try_consume(&TokenKind::Star).is_some() {
            return Node::Deref(Box::new(self.unary()));
        }
        if self.token_list.try_consume(&TokenKind::Ampersand).is_some() {
            return Node::Addr(Box::new(self.unary()));
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
                // 最大6つまでの引数をサポートする
                let mut args = vec![];
                let mut paren_consumed = false;
                for _ in 1..=6 {
                    if self.token_list.try_consume(&TokenKind::RParen).is_none() {
                        args.push(self.expr());
                        if self.token_list.try_consume(&TokenKind::RParen).is_none() {
                            self.token_list.expect_kind(&TokenKind::Comma);
                        } else {
                            paren_consumed = true;
                            break;
                        }
                    } else {
                        paren_consumed = true;
                        break;
                    }
                }
                if !paren_consumed {
                    self.token_list.expect_kind(&TokenKind::RParen);
                }
                return Node::Funcall(ident_name, args);
            } else {
                // パースする段階ではOffsetは未確定
                return Node::LocalVar(ident_name, None);
            }
        }

        let n = self.token_list.expect_num();
        Node::Num(n)
    }
}
