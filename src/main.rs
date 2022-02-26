mod error_report;
mod token;
mod token_iterator;
use std::{env, error, process::exit};

use crate::{
    token::Token,
    token_iterator::{TokenIterator, TokenizeError},
};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        println!("Usage: {} <ret-code>", args[0]);
        return;
    }

    let original_program = args[1].clone();
    let program = original_program.clone();

    println!("\t.section	__TEXT,__text,regular,pure_instructions");
    println!(".globl	_main");
    println!("_main:");
    println!("\tmov x0, #0");

    let mut token_iter = TokenIterator::new(&program);
    let head_number = token_iter.expect_num();
    println!("\tadd x0, x0, #{}", head_number);

    while let Some(token) = token_iter.next() {
        if token.is_err() {
            match token.unwrap_err() {
                e @ TokenizeError::UnknownToken { position } => {
                    error_report::error_at(&original_program, position, &e.to_string());
                }
            }
            exit(1)
        }

        match token.unwrap() {
            Token::Reserved { word, position } => {
                if word == "+" {
                    let n = token_iter.expect_num();
                    println!("\tadd x0, x0, #{}", n);
                } else if word == "-" {
                    let n = token_iter.expect_num();
                    println!("\tsub x0, x0, #{}", n);
                }
            }
            n => {
                error_report::error_at(&original_program, n.position(), "Unexpected token");
                exit(1);
            }
        }
    }

    println!("\tret");
}
