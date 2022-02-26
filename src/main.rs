mod ast;
mod error_report;
mod tokenizer;
use std::env;

use ast::{Node, NodeKind};

use crate::{ast::Lexer, tokenizer::Tokenizer};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        println!("Usage: {} <code>", args[0]);
        return;
    }

    let program = args[1].clone();
    let token_list = Tokenizer::new(&program).tokenize();
    let root = Lexer::new(token_list).expr();

    println!(".globl	_main");
    println!("_main:");
    println!("\tmov x0, xzr");

    gen(&root);

    println!("\tldr x0, [sp], #16");
    println!("\tret");
}

fn gen(expr_node: &Node) {
    if expr_node.kind == NodeKind::Num {
        println!("\tmov x2, #{}", expr_node.val.unwrap(),);
        println!("\tstr x2, [sp, #-16]!");
        return;
    }

    gen(expr_node.lhs.as_ref().unwrap());
    gen(expr_node.rhs.as_ref().unwrap());

    println!("\tldr x1, [sp], 16");
    println!("\tldr x0, [sp], 16");

    match expr_node.kind {
        NodeKind::Add => println!("\tadd x0, x0, x1"),
        NodeKind::Sub => println!("\tsub x0, x0, x1"),
        NodeKind::Mul => println!("\tmul x0, x0, x1"),
        NodeKind::Div => println!("\tsdiv x0, x0, x1"),
        _ => unreachable!(),
    }

    println!("\tstr x0, [sp, #-16]!");
}
