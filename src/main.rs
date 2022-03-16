mod codegen;
mod error_report;
mod parser;
mod tokenizer;

use crate::{parser::Parser, tokenizer::Tokenizer};
use std::{env, io::Read, path::PathBuf};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        println!("Usage: {} <c-file>", args[0]);
        return;
    }

    let program = read_program(args[1].clone());
    let token_list = Tokenizer::new(&program).tokenize();
    let (program_node, string_literals) = Parser::new(token_list).program();
    let code_generator = codegen::CodeGenerator::new(program_node, string_literals);
    code_generator.generate();
}

fn read_program<P: Into<PathBuf>>(path: P) -> String {
    let mut file = std::fs::File::open(path.into()).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    if !contents.ends_with("\n") {
        contents.push('\n');
    }
    contents
}
