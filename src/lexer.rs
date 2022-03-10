mod local_var_env;
mod node;
mod ty;

pub use node::{Ast, BinOpType, Node};

use self::local_var_env::LocalVarEnvironment;
use crate::tokenizer::{TokenKind, TokenList};
pub use ty::Ty;

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
            nodes.push(self.fundef());
        }

        nodes
    }

    fn resolve_local_var(&self, node: &mut Node, local_var_env: &mut LocalVarEnvironment) {
        match &mut node.ast {
            Ast::VarDef(name, ty) => {
                local_var_env.intern(name, ty.clone());
            }
            Ast::LocalVar { name, offset } => {
                if offset.is_none() {
                    if let Some(var_info) = local_var_env.get_var_info(name) {
                        *offset = Some(var_info.offset);
                        node.ty = Some(var_info.ty.clone());
                    } else {
                        panic!("Undefined variable: {}", name);
                    }
                } else {
                    unreachable!()
                }
            }
            Ast::BinOp(_, lhs, rhs) => {
                self.resolve_local_var(lhs, local_var_env);
                self.resolve_local_var(rhs, local_var_env);
            }
            Ast::Assign(lhs, rhs) => {
                self.resolve_local_var(lhs, local_var_env);
                self.resolve_local_var(rhs, local_var_env);
            }
            Ast::Num(_) => {}
            Ast::Addr(base_node) => {
                self.resolve_local_var(base_node, local_var_env);
                let base_node_ty = base_node.ty.clone().unwrap();
                node.ty = Some(Ty::Ptr(Box::new(base_node_ty)));
            }
            Ast::Deref(base_node) => {
                self.resolve_local_var(base_node, local_var_env);
                let base_node_ty = base_node.ty.clone().unwrap();
                node.ty = Some(base_node_ty.deref_type().clone());
            }
            Ast::Return(return_value_node) => {
                self.resolve_local_var(return_value_node, local_var_env);
            }
            Ast::If(condition, then_body, else_body) => {
                self.resolve_local_var(condition, local_var_env);
                self.resolve_local_var(then_body, local_var_env);
                if let Some(else_body) = else_body {
                    self.resolve_local_var(else_body, local_var_env);
                }
            }
            Ast::While(condition, body) => {
                self.resolve_local_var(condition, local_var_env);
                self.resolve_local_var(body, local_var_env);
            }
            Ast::For(init, check, update, body) => {
                if let Some(init) = init {
                    self.resolve_local_var(init, local_var_env);
                }
                if let Some(check) = check {
                    self.resolve_local_var(check, local_var_env);
                }
                if let Some(update) = update {
                    self.resolve_local_var(update, local_var_env);
                }
                self.resolve_local_var(body, local_var_env);
            }
            Ast::Block(stmts) => {
                for stmt in stmts.iter_mut() {
                    self.resolve_local_var(stmt, local_var_env);
                }
            }
            Ast::Funcall(_, args) => {
                for arg in args.iter_mut() {
                    self.resolve_local_var(arg, local_var_env);
                }
            }
            Ast::Fundef { .. } => {}
        }
    }

    fn fundef(&mut self) -> Node {
        self.token_list.expect_kind(&TokenKind::Int);
        let fn_name = self.token_list.expect_kind(&TokenKind::Ident).str.unwrap();
        let args = self.fundef_args();
        let mut body = self.fundef_body();

        // assign offsets to local variables
        // スタックのトップには、FPとLRが保存されているので、-16以降が変数領域
        let mut function_scope_local_var_env = LocalVarEnvironment::new_with_base_offset(16);
        for arg in args.iter() {
            function_scope_local_var_env.intern(&arg.1, arg.0.clone());
        }
        for b in body.iter_mut() {
            self.resolve_local_var(b, &mut function_scope_local_var_env)
        }
        let stack_size = function_scope_local_var_env.stack_size();

        Node::new(
            Ast::Fundef {
                name: fn_name,
                args,
                body,
                stack_size,
            },
            None,
        )
    }

    fn fundef_args(&mut self) -> Vec<(Ty, String)> {
        let mut args: Vec<(Ty, String)> = vec![];

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

    fn fundef_arg(&mut self) -> (Ty, String) {
        self.token_list.expect_kind(&TokenKind::Int);
        let name = self.token_list.expect_kind(&TokenKind::Ident).str.unwrap();

        (Ty::Int, name)
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
            let mut ty = Ty::Int;
            while self.token_list.try_consume(&TokenKind::Star).is_some() {
                ty = Ty::Ptr(Box::new(ty));
            }
            let ident_name = self.token_list.expect_kind(&TokenKind::Ident).str.unwrap();
            self.token_list.expect_kind(&TokenKind::Semicolon);

            Node::new(Ast::VarDef(ident_name, ty), None)
        } else if self.token_list.try_consume(&TokenKind::Return).is_some() {
            let return_value = self.expr();
            self.token_list.expect_kind(&TokenKind::Semicolon);

            Node::new(Ast::Return(Box::new(return_value)), None)
        } else if self.token_list.try_consume(&TokenKind::If).is_some() {
            self.token_list.expect_kind(&TokenKind::LParen);
            let condition = self.expr();
            self.token_list.expect_kind(&TokenKind::RParen);
            let then_body = self.stmt();

            if self.token_list.try_consume(&TokenKind::Else).is_some() {
                let else_body = self.stmt();

                return Node::new(
                    Ast::If(
                        Box::new(condition),
                        Box::new(then_body),
                        Some(Box::new(else_body)),
                    ),
                    None,
                );
            }

            Node::new(
                Ast::If(Box::new(condition), Box::new(then_body), None),
                None,
            )
        } else if self.token_list.try_consume(&TokenKind::While).is_some() {
            self.token_list.expect_kind(&TokenKind::LParen);
            let condition = self.expr();
            self.token_list.expect_kind(&TokenKind::RParen);
            let body = self.stmt();

            Node::new(Ast::While(Box::new(condition), Box::new(body)), None)
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
            Node::new(Ast::For(init, check, update, Box::new(body)), None)
        } else if self.token_list.try_consume(&TokenKind::LBrace).is_some() {
            let mut stmts = vec![];
            while self.token_list.try_consume(&TokenKind::RBrace).is_none() {
                stmts.push(self.stmt());
            }

            Node::new(Ast::Block(stmts), None)
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
            let rhs = self.assign();
            let rhs_ty = rhs.ty.clone();
            node = Node::new(Ast::Assign(Box::new(node), Box::new(rhs)), rhs_ty);
        }

        node
    }

    fn equality(&mut self) -> Node {
        let mut node = self.relational();

        loop {
            if self.token_list.try_consume(&TokenKind::Equal).is_some() {
                node = Node::new(
                    Ast::BinOp(
                        BinOpType::Equal,
                        Box::new(node),
                        Box::new(self.relational()),
                    ),
                    Some(Ty::Int),
                );
            } else if self.token_list.try_consume(&TokenKind::NotEqual).is_some() {
                node = Node::new(
                    Ast::BinOp(
                        BinOpType::NotEqual,
                        Box::new(node),
                        Box::new(self.relational()),
                    ),
                    Some(Ty::Int),
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
                node = Node::new(
                    Ast::BinOp(BinOpType::LessThan, Box::new(node), Box::new(self.add())),
                    Some(Ty::Int),
                );
            } else if self
                .token_list
                .try_consume(&TokenKind::LessThanOrEqual)
                .is_some()
            {
                node = Node::new(
                    Ast::BinOp(
                        BinOpType::LessThanOrEqual,
                        Box::new(node),
                        Box::new(self.add()),
                    ),
                    Some(Ty::Int),
                );
            } else if self
                .token_list
                .try_consume(&TokenKind::GreaterThan)
                .is_some()
            {
                node = Node::new(
                    Ast::BinOp(BinOpType::LessThan, Box::new(self.add()), Box::new(node)),
                    Some(Ty::Int),
                );
            } else if self
                .token_list
                .try_consume(&TokenKind::GreaterThanOrEqual)
                .is_some()
            {
                node = Node::new(
                    Ast::BinOp(
                        BinOpType::LessThanOrEqual,
                        Box::new(self.add()),
                        Box::new(node),
                    ),
                    Some(Ty::Int),
                );
            } else {
                return node;
            }
        }
    }

    fn add(&mut self) -> Node {
        let mut node = self.mul();
        let mut node_ty = node.ty.clone();

        loop {
            if self.token_list.try_consume(&TokenKind::Plus).is_some() {
                node = Node::new(
                    Ast::BinOp(BinOpType::Add, Box::new(node), Box::new(self.mul())),
                    node_ty,
                );
                node_ty = node.ty.clone();
            } else if self.token_list.try_consume(&TokenKind::Minus).is_some() {
                node = Node::new(
                    Ast::BinOp(BinOpType::Sub, Box::new(node), Box::new(self.mul())),
                    node_ty,
                );
                node_ty = node.ty.clone();
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Node {
        let mut node = self.unary();
        let mut node_ty = node.ty.clone();

        loop {
            if self.token_list.try_consume(&TokenKind::Star).is_some() {
                node = Node::new(
                    Ast::BinOp(BinOpType::Mul, Box::new(node), Box::new(self.unary())),
                    node_ty,
                );
                node_ty = node.ty.clone();
            } else if self.token_list.try_consume(&TokenKind::Div).is_some() {
                node = Node::new(
                    Ast::BinOp(BinOpType::Div, Box::new(node), Box::new(self.unary())),
                    node_ty,
                );
                node_ty = node.ty.clone();
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
            return Node::new(
                Ast::BinOp(
                    BinOpType::Sub,
                    Box::new(Node::new(Ast::Num(0), Some(Ty::Int))),
                    Box::new(self.primary()),
                ),
                Some(Ty::Int),
            );
        }
        if self.token_list.try_consume(&TokenKind::Star).is_some() {
            let base = self.unary();
            // パースする段階では、型が決まっていないので、型は決めない
            return Node::new(Ast::Deref(Box::new(base)), None);
        }
        if self.token_list.try_consume(&TokenKind::Ampersand).is_some() {
            let base = self.unary();
            // パースする段階では、型が決まっていないので、型は決めない
            return Node::new(Ast::Addr(Box::new(base)), None);
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
                // 今はint <function>しかサポートしていない
                return Node::new(Ast::Funcall(ident_name, args), Some(Ty::Int));
            } else {
                // パースの時点では型もoffsetも決められていないので、後で埋める
                return Node::new(
                    Ast::LocalVar {
                        name: ident_name,
                        offset: None,
                    },
                    None,
                );
            }
        }

        let n = self.token_list.expect_num();
        Node::new(Ast::Num(n), Some(Ty::Int))
    }
}
