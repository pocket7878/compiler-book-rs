mod ast;
mod codegen;
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
    let program_node = Lexer::new(token_list).expr();
    let code_generator = codegen::CodeGenerator::new(program_node);
    code_generator.generate();
}
