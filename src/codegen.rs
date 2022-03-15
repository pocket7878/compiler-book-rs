use crate::lexer::{Ast, BinOpType, Node, Ty};

const FRAME_POINTER_REGISTER: &str = "x29";
const LINK_REGISTER: &str = "x30";
const STACK_ALIGNMENT: i32 = 16;

pub struct CodeGenerator {
    pub program: Vec<Node>,
}

impl CodeGenerator {
    pub fn new(program: Vec<Node>) -> Self {
        Self { program }
    }

    pub fn generate(&self) {
        // プログラム中でユニークなラベルを生成するため
        let mut label_index = 0;
        for stmt in self.program.iter() {
            match stmt {
                Node {
                    ast: Ast::Fundef { name, .. },
                    ..
                } => {
                    self.gen(stmt, &mut label_index, Some(name));
                }
                Node {
                    ast: Ast::GlobalVarDef(name, ty),
                    ..
                } => {
                    println!(".comm _{},{}", name, ty.size());
                }
                _ => {
                    panic!("Unsupported toplevel node: {:?}", stmt);
                }
            }
        }
    }

    fn gen(&self, node: &Node, label_index: &mut i32, current_fn_name: Option<&str>) {
        self.generate_comment(&format!("Compiling: {:?}", node));
        match &node.ast {
            Ast::Num(n) => {
                self.generate_comment(&format!("num: {}", n));
                println!("\tmov x2, #{}", n);
                self.generate_push_register_to_stack("x2");
            }
            Ast::LocalVarDef(_, _) => {}
            Ast::GlobalVarDef(..) => {}
            Ast::LocalVar { name, offset, .. } => {
                self.generate_comment(&format!("local var {} at {}", name, offset));

                self.generate_comment("\t local var push address to stack");
                self.generate_var(node);

                self.generate_comment("\t local var pop address from stack");
                self.generate_pop_register_from_stack("x0");

                self.generate_comment("\t local var read address content to register");
                self.load(&node.ty);

                self.generate_comment("\t local var push value to stack");
                self.generate_push_register_to_stack("x0");
            }
            Ast::GlobalVar { name } => {
                self.generate_comment(&format!("global var {}", name));

                self.generate_comment("\t global var push address to stack");
                self.generate_var(node);

                self.generate_comment("\t global var pop address from stack");
                self.generate_pop_register_from_stack("x0");

                self.generate_comment("\t global var read address content to register");
                self.load(&node.ty);

                self.generate_comment("\t global var push value to stack");
                self.generate_push_register_to_stack("x0");
            }
            Ast::Addr(base_node) => {
                self.generate_var(base_node);
            }
            Ast::Deref(base_node) => {
                self.gen(base_node, label_index, current_fn_name);
                self.generate_pop_register_from_stack("x0");
                self.load(&node.ty);
                self.generate_push_register_to_stack("x0");
            }
            Ast::Assign(lhs, rhs) => {
                self.generate_comment("assign");

                self.generate_comment("\tassign push lhs(address)");
                if let Node {
                    ast: Ast::Deref(derefed_lhs),
                    ..
                } = &**lhs
                {
                    self.gen(derefed_lhs, label_index, current_fn_name);
                } else {
                    self.generate_var(lhs.as_ref());
                }

                self.generate_comment("\tassign push rhs(value)");
                self.gen(rhs.as_ref(), label_index, current_fn_name);

                self.generate_comment("\tassign pop value and address");
                self.generate_pop_register_from_stack("x1");
                self.generate_pop_register_from_stack("x0");

                self.generate_comment("\tassign store values to address");
                self.store(&rhs.as_ref().ty);

                // Cでは代入式は代入された値を返す
                self.generate_comment("\tassign push assigned value to stack");
                self.generate_push_register_to_stack("x1");
            }
            Ast::If(condition, then_body, else_body) => {
                self.generate_comment("if");
                self.generate_comment("\tif condition");
                self.gen(condition.as_ref(), label_index, current_fn_name);
                self.generate_pop_register_from_stack("x0");
                println!("\tcmp x0, #0");

                let idx = self.increment_label_index(label_index);
                if let Some(else_body) = &else_body {
                    println!("\tb.eq .Lelse{}", idx);
                    self.generate_comment("\tif then-body");
                    self.gen(then_body.as_ref(), label_index, current_fn_name);
                    println!("\tb .Lend{}", idx);
                    println!(".Lelse{}:", idx);
                    self.generate_comment("\tif then-else-body");
                    self.gen(else_body, label_index, current_fn_name);
                } else {
                    println!("\tb.eq .Lend{}", idx);
                    self.gen(then_body.as_ref(), label_index, current_fn_name);
                }

                println!(".Lend{}:", idx);
            }
            Ast::While(condition, body) => {
                let idx = self.increment_label_index(label_index);
                println!(".Lbegin{}:", idx);
                self.gen(condition.as_ref(), label_index, current_fn_name);
                self.generate_pop_register_from_stack("x0");
                println!("\tcmp x0, #0");
                println!("\tb.eq .Lend{}", idx);
                self.gen(body.as_ref(), label_index, current_fn_name);
                println!("\tb .Lbegin{}", idx);
                println!(".Lend{}:", idx);
            }
            Ast::For(init, check, update, body) => {
                let idx = self.increment_label_index(label_index);
                if let Some(init) = init {
                    self.gen(init.as_ref(), label_index, current_fn_name);
                }
                println!(".Lbegin{}:", idx);
                if let Some(check) = check {
                    self.gen(check.as_ref(), label_index, current_fn_name);
                } else {
                    // checkがない場合は常にtrueにする
                    self.gen(
                        &Node::new(Ast::Num(1), Some(Ty::Int)),
                        label_index,
                        current_fn_name,
                    );
                }
                self.generate_pop_register_from_stack("x0");
                println!("\tcmp x0, #0");
                println!("\tb.eq .Lend{}", idx);
                self.gen(body.as_ref(), label_index, current_fn_name);
                if let Some(update) = update {
                    self.gen(update.as_ref(), label_index, current_fn_name);
                }
                println!("\tb .Lbegin{}", idx);
                println!(".Lend{}:", idx);
            }
            Ast::Block(stmts) => {
                for s in stmts {
                    self.gen(s, label_index, current_fn_name);
                    self.generate_pop_register_from_stack("x0");
                }
            }
            Ast::Funcall(name, args) => {
                for a in args {
                    self.gen(a, label_index, current_fn_name);
                }
                for i in (0..args.len()).rev() {
                    self.generate_pop_register_from_stack(&format!("x{}", i));
                }
                println!("\tbl _{}", name);
                // 関数の戻り値はx0に入っている
                self.generate_push_register_to_stack("x0");
            }
            Ast::Return(value) => {
                self.generate_comment("return");
                self.gen(value.as_ref(), label_index, current_fn_name);
                self.generate_pop_register_from_stack("x0");
                println!("\tb .L.return_{}", current_fn_name.unwrap());
            }
            Ast::Fundef {
                name,
                args,
                body,
                stack_size,
            } => {
                println!("\t.globl _{}", name);
                println!("\t.p2align 2");
                println!("_{}:", name);
                self.generate_comment("Store FP & LR to stack");
                println!(
                    "\tstp {}, {}, [sp, #-16]!",
                    FRAME_POINTER_REGISTER, LINK_REGISTER
                );
                self.generate_comment("Update FP");
                println!("\tmov {}, sp", FRAME_POINTER_REGISTER);
                self.generate_comment("Allocate stack space for local variables & arguments");
                println!("\tsub sp, sp, #{}", stack_size);
                self.generate_comment("Copy arguments into stack");
                for arg in args.iter().enumerate() {
                    if let Ast::LocalVar { offset, .. } = arg.1.ast {
                        println!(
                            "\tstur x{}, [{}, #-{}]",
                            arg.0, FRAME_POINTER_REGISTER, offset,
                        );
                    } else {
                        panic!("unexpected function arg ast: {:?}", arg.1.ast);
                    }
                }
                for s in body {
                    self.gen(s, label_index, Some(name));
                }
                println!(".L.return_{}:", name);
                self.generate_comment("Restore FP & LR from stack");
                println!("\tmov sp, {}", FRAME_POINTER_REGISTER);
                println!(
                    "\tldp {}, {}, [sp], #16",
                    FRAME_POINTER_REGISTER, LINK_REGISTER
                );
                println!("\tret")
            }
            Ast::BinOp(op, lhs, rhs) => {
                self.gen(lhs.as_ref(), label_index, current_fn_name);
                self.gen(rhs.as_ref(), label_index, current_fn_name);
                self.generate_pop_register_from_stack("x1");
                self.generate_pop_register_from_stack("x0");

                match *op {
                    BinOpType::Add => println!("\tadd x0, x0, x1"),
                    BinOpType::Sub => println!("\tsub x0, x0, x1"),
                    BinOpType::Mul => println!("\tmul x0, x0, x1"),
                    BinOpType::Div => println!("\tsdiv x0, x0, x1"),
                    BinOpType::Equal => {
                        println!("\tcmp x0, x1");
                        println!("\tcset x0, EQ");
                    }
                    BinOpType::NotEqual => {
                        println!("\tcmp x0, x1");
                        println!("\tcset x0, NE");
                    }
                    BinOpType::LessThan => {
                        println!("\tcmp x0, x1");
                        println!("\tcset x0, LT");
                    }
                    BinOpType::LessThanOrEqual => {
                        println!("\tcmp x0, x1");
                        println!("\tcset x0, LE");
                    }
                }
                self.generate_push_register_to_stack("x0");
            }
        }
    }

    fn increment_label_index(&self, label_index: &mut i32) -> i32 {
        let idx = *label_index;
        *label_index += 1;

        idx
    }

    fn generate_var(&self, node: &Node) {
        match &node.ast {
            Ast::LocalVar { offset, .. } => {
                println!("\tmov x0, {}", FRAME_POINTER_REGISTER);
                println!("\tsub x0, x0, #{}", offset);
                self.generate_push_register_to_stack("x0");
            }
            Ast::GlobalVar { name } => {
                println!("\tadrp x0, _{}@GOTPAGE", name);
                self.generate_push_register_to_stack("x0");
            }
            _ => {
                panic!("Node: {:?} is not local var", node);
            }
        }
    }

    fn load(&self, ty: &Option<Ty>) {
        self.generate_comment(&format!("Load {:?} type value from x0", ty));
        match *ty {
            Some(Ty::Array(..)) => {
                // 配列は先頭要素へのポインターとして扱うので、アドレスからロードはしない
                self.generate_comment("Treat array as pointer");
            }
            Some(ref non_array_ty) => match non_array_ty.size() {
                4 => println!("\tldrsw x0, [x0]"),
                8 => println!("\tldr x0, [x0]"),
                _ => panic!("ty: {:?} is not supported", non_array_ty),
            },
            None => {
                panic!("ty is None");
            }
        }
    }

    fn store(&self, ty: &Option<Ty>) {
        self.generate_comment(&format!("Store {:?} type value from x1", ty));
        match *ty {
            Some(Ty::Array(..)) => {
                // Store array as pointer to head element
                println!("\tstr x1, [x0]");
            }
            Some(ref non_array_ty) => match non_array_ty.size() {
                4 => println!("\tstr w1, [x0]"),
                8 => println!("\tstr x1, [x0]"),
                _ => panic!("ty: {:?} is not supported", non_array_ty),
            },
            None => {
                panic!("ty is None");
            }
        }
    }

    fn generate_push_register_to_stack(&self, register: &str) {
        println!("\tstr {}, [sp, #-{}]!", register, STACK_ALIGNMENT);
    }

    fn generate_pop_register_from_stack(&self, register: &str) {
        println!("\tldr {}, [sp], #{}", register, STACK_ALIGNMENT);
    }

    fn generate_comment(&self, comment: &str) {
        println!("\t; {}", comment);
    }
}
