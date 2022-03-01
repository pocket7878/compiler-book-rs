use crate::lexer::{BinOpType, Node};

const FRAME_POINTER_REGISTER: &str = "x29";
const STACK_ALIGNMENT: i32 = 16;

pub struct CodeGenerator {
    pub program: Vec<Node>,
}

impl CodeGenerator {
    pub fn new(program: Vec<Node>) -> Self {
        Self { program }
    }

    pub fn generate(&self) {
        self.generate_program_opening();
        for stmt in self.program.iter() {
            self.gen(stmt);
            self.generate_pop_register_from_stack("x0");
        }
        self.generate_program_ending();
    }

    fn gen(&self, node: &Node) {
        match node {
            Node::Num(n) => {
                self.generate_comment(&format!("num: {}", n));
                println!("\tmov x2, #{}", n);
                self.generate_push_register_to_stack("x2");
                return;
            }
            Node::LocalVar(offset) => {
                self.generate_comment(&format!("local var offset: {}", offset));

                self.generate_comment("\t local var push address to stack");
                self.generate_local_var(node);

                self.generate_comment("\t local var pop address from stack");
                self.generate_pop_register_from_stack("x0");

                self.generate_comment("\t local var read address content to register");
                println!("\tldr x0, [x0]");

                self.generate_comment("\t local var push value to stack");
                self.generate_push_register_to_stack("x0");
                return;
            }
            Node::Assign(lhs, rhs) => {
                self.generate_comment(&format!("assign"));

                self.generate_comment(&format!("\tassign push lhs(address)"));
                self.generate_local_var(lhs.as_ref());

                self.generate_comment(&format!("\tassign push rhs(value)"));
                self.gen(rhs.as_ref());

                self.generate_comment(&format!("\tassign pop value and address"));
                self.generate_pop_register_from_stack("x1");
                self.generate_pop_register_from_stack("x0");

                self.generate_comment(&format!("\tassign store values to address"));
                println!("\tstr x1, [x0]");

                // Cでは代入式は代入された値を返す
                self.generate_comment(&format!("\tassign push assigned value to stack"));
                self.generate_push_register_to_stack("x1");
                return;
            }
            Node::Return(value) => {
                self.generate_comment("return");
                self.gen(value.as_ref());
                self.generate_pop_register_from_stack("x0");
                println!("\tmov sp, {}", FRAME_POINTER_REGISTER);
                self.generate_pop_register_from_stack(FRAME_POINTER_REGISTER);
                println!("\tret");
                return;
            }
            Node::If(condition, then_body, else_body) => {
                self.generate_comment("if");
                self.generate_comment("\tif condition");
                self.gen(condition.as_ref());
                self.generate_pop_register_from_stack("x0");
                println!("\tcmp x0, #0");
                if let Some(else_body) = &else_body {
                    println!("\tb.eq .Lelse0");
                    self.generate_comment("\tif then-body");
                    self.gen(then_body.as_ref());
                    println!("\tb .Lend0");
                    println!(".Lelse0:");
                    self.generate_comment("\tif then-else-body");
                    self.gen(else_body);
                } else {
                    println!("\tb.eq .Lend0");
                    self.gen(then_body.as_ref());
                }
                println!(".Lend0:");
                return;
            }
            Node::While(condition, body) => {
                println!(".Lbegin0:");
                self.gen(condition.as_ref());
                self.generate_pop_register_from_stack("x0");
                println!("\tcmp x0, #0");
                println!("\tb.eq .Lend0");
                self.gen(body.as_ref());
                println!("\tb .Lbegin0");
                println!(".Lend0:");
                return;
            }
            Node::BinOp(op, lhs, rhs) => {
                self.gen(lhs.as_ref());
                self.gen(rhs.as_ref());

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

    fn generate_local_var(&self, node: &Node) {
        match node {
            Node::LocalVar(offset) => {
                println!("\tmov x0, {}", FRAME_POINTER_REGISTER);
                println!("\tsub x0, x0, #{}", offset);
                self.generate_push_register_to_stack("x0");
            }
            _ => {
                panic!("Node: {:?} is not local var", node);
            }
        }
    }

    fn generate_program_opening(&self) {
        println!(".globl	_main");
        println!("_main:");
        println!("\tmov {}, sp", FRAME_POINTER_REGISTER);
        println!("\tsub sp, sp, #{}", STACK_ALIGNMENT * 26);
    }

    fn generate_program_ending(&self) {
        println!("\tmov sp, {}", FRAME_POINTER_REGISTER);
        self.generate_pop_register_from_stack(FRAME_POINTER_REGISTER);
        println!("\tret");
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
