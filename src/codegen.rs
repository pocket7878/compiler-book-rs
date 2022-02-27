use crate::ast::{Node, NodeKind};

pub struct CodeGenerator {
    pub program: Node,
}

impl CodeGenerator {
    pub fn new(program: Node) -> Self {
        Self { program }
    }

    pub fn generate(&self) {
        self.generate_program_opening();
        self.gen(&self.program);
        self.generate_program_ending();
    }

    pub fn gen(&self, node: &Node) {
        if node.kind == NodeKind::Num {
            println!("\tmov x2, #{}", node.val.unwrap(),);
            println!("\tstr x2, [sp, #-16]!");
            return;
        }

        self.gen(node.lhs.as_ref().unwrap());
        self.gen(node.rhs.as_ref().unwrap());

        println!("\tldr x1, [sp], 16");
        println!("\tldr x0, [sp], 16");

        match node.kind {
            NodeKind::Add => println!("\tadd x0, x0, x1"),
            NodeKind::Sub => println!("\tsub x0, x0, x1"),
            NodeKind::Mul => println!("\tmul x0, x0, x1"),
            NodeKind::Div => println!("\tsdiv x0, x0, x1"),
            _ => unreachable!(),
        }

        println!("\tstr x0, [sp, #-16]!");
    }

    fn generate_program_opening(&self) {
        println!(".globl	_main");
        println!("_main:");
        println!("\tmov x0, xzr");
    }

    fn generate_program_ending(&self) {
        println!("\tldr x0, [sp], #16");
        println!("\tret");
    }
}
