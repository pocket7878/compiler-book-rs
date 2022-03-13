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

    fn fundef(&mut self) -> Node {
        self.token_list.expect_kind(&TokenKind::Int);
        let fn_name = self.token_list.expect_kind(&TokenKind::Ident).str.unwrap();
        // assign offsets to local variables
        // スタックのトップには、FPとLRが保存されているので、-16以降が変数領域
        let args = self.fundef_args();
        let mut function_scope_local_var_env = LocalVarEnvironment::new_with_base_offset(16);
        for arg in args.iter() {
            function_scope_local_var_env.intern(&arg.1, arg.0.clone());
        }
        let body = self.fundef_body(&mut function_scope_local_var_env);

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

    fn fundef_body(&mut self, local_var_env: &mut LocalVarEnvironment) -> Vec<Node> {
        self.token_list.expect_kind(&TokenKind::LBrace);
        let mut stmts = vec![];
        while self.token_list.try_consume(&TokenKind::RBrace).is_none() {
            stmts.push(self.stmt(local_var_env));
        }

        stmts
    }

    fn stmt(&mut self, local_var_env: &mut LocalVarEnvironment) -> Node {
        if self.token_list.try_consume(&TokenKind::Int).is_some() {
            let mut ty = Ty::Int;
            while self.token_list.try_consume(&TokenKind::Star).is_some() {
                ty = Ty::Ptr(Box::new(ty));
            }
            let ident_name = self.token_list.expect_kind(&TokenKind::Ident).str.unwrap();
            let mut array_dimens = vec![];
            while self.token_list.try_consume(&TokenKind::LBracket).is_some() {
                let dimen = self.token_list.expect_num();
                array_dimens.push(dimen);
                self.token_list.expect_kind(&TokenKind::RBracket);
            }
            for dimen in array_dimens.iter().rev() {
                ty = Ty::Array(Box::new(ty), *dimen);
            }
            self.token_list.expect_kind(&TokenKind::Semicolon);
            local_var_env.intern(&ident_name, ty.clone());

            Node::new(Ast::VarDef(ident_name, ty), None)
        } else if self.token_list.try_consume(&TokenKind::Return).is_some() {
            let return_value = self.expr(local_var_env);
            self.token_list.expect_kind(&TokenKind::Semicolon);

            Node::new(Ast::Return(Box::new(return_value)), None)
        } else if self.token_list.try_consume(&TokenKind::If).is_some() {
            self.token_list.expect_kind(&TokenKind::LParen);
            let condition = self.expr(local_var_env);
            self.token_list.expect_kind(&TokenKind::RParen);
            let then_body = self.stmt(local_var_env);

            if self.token_list.try_consume(&TokenKind::Else).is_some() {
                let else_body = self.stmt(local_var_env);

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
            let condition = self.expr(local_var_env);
            self.token_list.expect_kind(&TokenKind::RParen);
            let body = self.stmt(local_var_env);

            Node::new(Ast::While(Box::new(condition), Box::new(body)), None)
        } else if self.token_list.try_consume(&TokenKind::For).is_some() {
            // forの後には、for (初期化; 条件; 更新) 本体
            // ただし、初期化, 条件, 更新はどれも省略可能
            self.token_list.expect_kind(&TokenKind::LParen);
            let init = if self.token_list.try_consume(&TokenKind::Semicolon).is_some() {
                None
            } else {
                let node = self.expr(local_var_env);
                self.token_list.expect_kind(&TokenKind::Semicolon);

                Some(Box::new(node))
            };
            let check = if self.token_list.try_consume(&TokenKind::Semicolon).is_some() {
                None
            } else {
                let node = self.expr(local_var_env);
                self.token_list.expect_kind(&TokenKind::Semicolon);

                Some(Box::new(node))
            };
            let update = if self.token_list.try_consume(&TokenKind::RParen).is_some() {
                None
            } else {
                let node = self.expr(local_var_env);
                self.token_list.expect_kind(&TokenKind::RParen);

                Some(Box::new(node))
            };

            let body = self.stmt(local_var_env);
            Node::new(Ast::For(init, check, update, Box::new(body)), None)
        } else if self.token_list.try_consume(&TokenKind::LBrace).is_some() {
            let mut stmts = vec![];
            while self.token_list.try_consume(&TokenKind::RBrace).is_none() {
                stmts.push(self.stmt(local_var_env));
            }

            Node::new(Ast::Block(stmts), None)
        } else {
            let expr = self.expr(local_var_env);
            self.token_list.expect_kind(&TokenKind::Semicolon);

            expr
        }
    }

    fn expr(&mut self, local_var_env: &mut LocalVarEnvironment) -> Node {
        self.assign(local_var_env)
    }

    fn assign(&mut self, local_var_env: &mut LocalVarEnvironment) -> Node {
        let mut node = self.equality(local_var_env);
        if self.token_list.try_consume(&TokenKind::Assign).is_some() {
            let lhs = node;
            if let Some(Ty::Array(..)) = lhs.ty {
                panic!("{:?} is not a lvalue", lhs)
            }
            let lhs_ty = lhs.ty.clone();
            let rhs = self.assign(local_var_env);
            node = Node::new(Ast::Assign(Box::new(lhs), Box::new(rhs)), lhs_ty);
        }

        node
    }

    fn equality(&mut self, local_var_env: &mut LocalVarEnvironment) -> Node {
        let mut node = self.relational(local_var_env);

        loop {
            if self.token_list.try_consume(&TokenKind::Equal).is_some() {
                let lhs = node;
                let rhs = self.relational(local_var_env);
                node = Node::new(
                    Ast::BinOp(BinOpType::Equal, Box::new(lhs), Box::new(rhs)),
                    Some(Ty::Int),
                );
            } else if self.token_list.try_consume(&TokenKind::NotEqual).is_some() {
                let lhs = node;
                let rhs = self.relational(local_var_env);
                node = Node::new(
                    Ast::BinOp(BinOpType::NotEqual, Box::new(lhs), Box::new(rhs)),
                    Some(Ty::Int),
                );
            } else {
                return node;
            }
        }
    }

    fn relational(&mut self, local_var_env: &mut LocalVarEnvironment) -> Node {
        let mut node = self.add(local_var_env);

        loop {
            if self.token_list.try_consume(&TokenKind::LessThan).is_some() {
                let lhs = node;
                let rhs = self.add(local_var_env);
                node = Node::new(
                    Ast::BinOp(BinOpType::LessThan, Box::new(lhs), Box::new(rhs)),
                    Some(Ty::Int),
                );
            } else if self
                .token_list
                .try_consume(&TokenKind::LessThanOrEqual)
                .is_some()
            {
                let lhs = node;
                let rhs = self.add(local_var_env);
                node = Node::new(
                    Ast::BinOp(BinOpType::LessThanOrEqual, Box::new(lhs), Box::new(rhs)),
                    Some(Ty::Int),
                );
            } else if self
                .token_list
                .try_consume(&TokenKind::GreaterThan)
                .is_some()
            {
                let lhs = self.add(local_var_env);
                let rhs = node;
                node = Node::new(
                    Ast::BinOp(BinOpType::LessThan, Box::new(lhs), Box::new(rhs)),
                    Some(Ty::Int),
                );
            } else if self
                .token_list
                .try_consume(&TokenKind::GreaterThanOrEqual)
                .is_some()
            {
                let lhs = self.add(local_var_env);
                let rhs = node;
                node = Node::new(
                    Ast::BinOp(BinOpType::LessThanOrEqual, Box::new(lhs), Box::new(rhs)),
                    Some(Ty::Int),
                );
            } else {
                return node;
            }
        }
    }

    fn add(&mut self, local_var_env: &mut LocalVarEnvironment) -> Node {
        let mut node = self.mul(local_var_env);

        loop {
            if self.token_list.try_consume(&TokenKind::Plus).is_some() {
                let lhs = node;
                let lhs_ty = lhs.ty.clone();
                let mut rhs = self.mul(local_var_env);
                match &lhs_ty {
                    Some(Ty::Int) => {
                        node = Node::new(
                            Ast::BinOp(BinOpType::Add, Box::new(lhs), Box::new(rhs)),
                            lhs_ty.clone(),
                        );
                    }
                    //何かの値の参照をしている型は、参照先の型のサイズに応じてスケールする必要があるので欠け算のノードを挟んでおく
                    Some(refrence_type) => {
                        rhs = Node::new(
                            Ast::BinOp(
                                BinOpType::Mul,
                                Box::new(rhs),
                                Box::new(Node::new(
                                    Ast::Num(refrence_type.base_ty().size()),
                                    Some(Ty::Int),
                                )),
                            ),
                            Some(Ty::Int),
                        );
                        node = Node::new(
                            Ast::BinOp(BinOpType::Add, Box::new(lhs), Box::new(rhs)),
                            lhs_ty.clone(),
                        );
                    }
                    None => unreachable!(),
                }
            } else if self.token_list.try_consume(&TokenKind::Minus).is_some() {
                let lhs = node;
                let lhs_ty = lhs.ty.clone();
                let mut rhs = self.mul(local_var_env);
                match &lhs_ty {
                    Some(Ty::Int) => {
                        node = Node::new(
                            Ast::BinOp(BinOpType::Sub, Box::new(lhs), Box::new(rhs)),
                            lhs_ty.clone(),
                        );
                    }
                    //何かの値の参照をしている型は、参照先の型のサイズに応じてスケールする必要があるので欠け算のノードを挟んでおく
                    Some(refrence_type) => {
                        rhs = Node::new(
                            Ast::BinOp(
                                BinOpType::Mul,
                                Box::new(rhs),
                                Box::new(Node::new(
                                    Ast::Num(refrence_type.base_ty().size()),
                                    Some(Ty::Int),
                                )),
                            ),
                            Some(Ty::Int),
                        );
                        node = Node::new(
                            Ast::BinOp(BinOpType::Sub, Box::new(lhs), Box::new(rhs)),
                            lhs_ty.clone(),
                        );
                    }
                    None => unreachable!(),
                }
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self, local_var_env: &mut LocalVarEnvironment) -> Node {
        let mut node = self.unary(local_var_env);
        let mut node_ty = node.ty.clone();

        loop {
            if self.token_list.try_consume(&TokenKind::Star).is_some() {
                let lhs = node;
                let rhs = self.unary(local_var_env);
                node = Node::new(
                    Ast::BinOp(BinOpType::Mul, Box::new(lhs), Box::new(rhs)),
                    node_ty,
                );
                node_ty = node.ty.clone();
            } else if self.token_list.try_consume(&TokenKind::Div).is_some() {
                let lhs = node;
                let rhs = self.unary(local_var_env);
                node = Node::new(
                    Ast::BinOp(BinOpType::Div, Box::new(lhs), Box::new(rhs)),
                    node_ty,
                );
                node_ty = node.ty.clone();
            } else {
                return node;
            }
        }
    }

    fn unary(&mut self, local_var_env: &mut LocalVarEnvironment) -> Node {
        if self.token_list.try_consume(&TokenKind::SizeOf).is_some() {
            let node = self.unary(local_var_env);
            let node_ty = node.ty.unwrap();
            return Node::new(Ast::Num(node_ty.size()), Some(Ty::Int));
        }
        if self.token_list.try_consume(&TokenKind::Plus).is_some() {
            // TODO: should check to_ptr_if_array?
            return self.primary(local_var_env);
        }
        if self.token_list.try_consume(&TokenKind::Minus).is_some() {
            let rhs = self.primary(local_var_env);
            return Node::new(
                Ast::BinOp(
                    BinOpType::Sub,
                    Box::new(Node::new(Ast::Num(0), Some(Ty::Int))),
                    Box::new(rhs),
                ),
                Some(Ty::Int),
            );
        }
        if self.token_list.try_consume(&TokenKind::Star).is_some() {
            let base = self.unary(local_var_env);
            let base_ty = base.ty.clone();
            return Node::new(Ast::Deref(Box::new(base)), Some(base_ty.unwrap().base_ty()));
        }
        if self.token_list.try_consume(&TokenKind::Ampersand).is_some() {
            let base = self.unary(local_var_env);
            let base_ty = base.ty.clone().unwrap();
            match base_ty {
                Ty::Array(item_ty, ..) => {
                    return Node::new(Ast::Addr(Box::new(base)), Some(Ty::Ptr(item_ty)));
                }
                _ => {
                    return Node::new(Ast::Addr(Box::new(base)), Some(Ty::Ptr(Box::new(base_ty))));
                }
            }
        }
        self.primary(local_var_env)
    }

    fn primary(&mut self, local_var_env: &mut LocalVarEnvironment) -> Node {
        if self.token_list.try_consume(&TokenKind::LParen).is_some() {
            let node = self.expr(local_var_env);
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
                        let arg = self.expr(local_var_env);
                        args.push(arg);
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
            } else if let Some(var_info) = local_var_env.get_var_info(&ident_name) {
                let mut node = Node::new(
                    Ast::LocalVar {
                        name: ident_name,
                        offset: var_info.offset,
                    },
                    Some(var_info.ty.clone()),
                );
                let mut ty = var_info.ty.clone();
                let mut array_dimens = vec![];
                while self.token_list.try_consume(&TokenKind::LBracket).is_some() {
                    let dimen = self.token_list.expect_num();
                    array_dimens.push(dimen);
                    self.token_list.expect_kind(&TokenKind::RBracket);
                }
                for dimen in array_dimens.iter().rev() {
                    node = Node::new(
                        Ast::Deref(Box::new(Node::new(
                            Ast::BinOp(
                                BinOpType::Add,
                                Box::new(node),
                                Box::new(Node::new(Ast::Num(*dimen), Some(Ty::Int))),
                            ),
                            Some(ty.clone()),
                        ))),
                        Some(ty.base_ty()),
                    );
                    ty = ty.base_ty();
                }
                return node;
            } else {
                panic!("undefined variable: {}", ident_name);
            }
        }

        let n = self.token_list.expect_num();
        Node::new(Ast::Num(n), Some(Ty::Int))
    }
}
