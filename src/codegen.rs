use crate::lexer::{Node, NodeKind};

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
        match node.kind {
            NodeKind::Num => {
                self.generate_comment(&format!("num: {}", node.val.unwrap()));
                self.generate_num(node);
                return;
            }
            NodeKind::LocalVar => {
                self.generate_comment(&format!("local var offset: {}", node.offset.unwrap()));
                self.generate_comment("\t local var push address to stack");
                self.generate_local_var(node);
                self.generate_comment("\t local var pop address from stack");
                self.generate_pop_register_from_stack("x0");
                self.generate_comment("\t local var read address content to register");
                println!("\tldr x0, [x0]");
                self.generate_comment("\t local var value to stack");
                self.generate_push_register_to_stack("x0");
                return;
            }
            NodeKind::Assign => {
                self.generate_comment(&format!("assign local var"));
                self.generate_comment(&format!("\tassign local var push lhs(address)"));
                self.generate_local_var(node.lhs.as_ref().unwrap());
                self.generate_comment(&format!("\tassign local var push rhs(value)"));
                self.gen(node.rhs.as_ref().unwrap());
                self.generate_comment(&format!("\tassign pop value and address"));
                self.generate_pop_register_from_stack("x1");
                self.generate_pop_register_from_stack("x0");
                self.generate_comment(&format!("\tassign store to address"));
                println!("\tstr x1, [x0]");
                self.generate_push_register_to_stack("x1");
                return;
            }
            NodeKind::Return => {
                self.generate_comment("return");
                self.gen(node.lhs.as_ref().unwrap());
                self.generate_pop_register_from_stack("x0");
                println!("\tmov sp, {}", FRAME_POINTER_REGISTER);
                self.generate_pop_register_from_stack(FRAME_POINTER_REGISTER);
                println!("\tret");
                return;
            }
            NodeKind::If => {
                self.generate_comment("if");
                self.generate_comment("\tif condition");
                self.gen(node.lhs.as_ref().unwrap());
                self.generate_pop_register_from_stack("x0");
                println!("\tcmp x0, #0");
                if let Some(else_body) = &node.else_body {
                    println!("\tb.eq .Lelse0");
                    self.generate_comment("\tif then-body");
                    self.gen(node.rhs.as_ref().unwrap());
                    println!("\tb .Lend0");
                    println!(".Lelse0:");
                    self.generate_comment("\tif then-else-body");
                    self.gen(else_body);
                } else {
                    println!("\tb.eq .Lend0");
                    self.gen(node.rhs.as_ref().unwrap());
                }
                println!(".Lend0:");
                return;
            }
            NodeKind::While => {
                println!(".Lbegin0:");
                self.gen(node.lhs.as_ref().unwrap());
                self.generate_pop_register_from_stack("x0");
                println!("\tcmp x0, #0");
                println!("\tb.eq .Lend0");
                self.gen(node.rhs.as_ref().unwrap());
                println!("\tb .Lbegin0");
                println!(".Lend0:");
                return;
            }
            _ => { /* Nothing to DO */ }
        }

        self.gen(node.lhs.as_ref().unwrap());
        self.gen(node.rhs.as_ref().unwrap());

        self.generate_pop_register_from_stack("x1");
        self.generate_pop_register_from_stack("x0");

        match node.kind {
            NodeKind::Add => println!("\tadd x0, x0, x1"),
            NodeKind::Sub => println!("\tsub x0, x0, x1"),
            NodeKind::Mul => println!("\tmul x0, x0, x1"),
            NodeKind::Div => println!("\tsdiv x0, x0, x1"),
            NodeKind::Equal => {
                println!("\tcmp x0, x1");
                println!("\tcset x0, EQ");
            }
            NodeKind::NotEqual => {
                println!("\tcmp x0, x1");
                println!("\tcset x0, NE");
            }
            NodeKind::LessThan => {
                println!("\tcmp x0, x1");
                println!("\tcset x0, LT");
            }
            NodeKind::LessThanOrEqual => {
                println!("\tcmp x0, x1");
                println!("\tcset x0, LE");
            }
            _ => { /* Ignore */ }
        }

        self.generate_push_register_to_stack("x0");
    }

    fn generate_num(&self, node: &Node) {
        println!("\tmov x2, #{}", node.val.unwrap());
        self.generate_push_register_to_stack("x2");
    }

    fn generate_local_var(&self, node: &Node) {
        println!("\tmov x0, {}", FRAME_POINTER_REGISTER);
        println!("\tsub x0, x0, #{}", node.offset.unwrap());
        self.generate_push_register_to_stack("x0");
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
