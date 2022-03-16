mod codegen;
mod error_report;
mod parser;
mod tokenizer;

use crate::{parser::Parser, tokenizer::Tokenizer};
use std::env;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        println!("Usage: {} <code>", args[0]);
        return;
    }

    let program = args[1].clone();
    let token_list = Tokenizer::new(&program).tokenize();
    let program_node = Parser::new(token_list).program();
    let code_generator = codegen::CodeGenerator::new(program_node);
    code_generator.generate();
}
