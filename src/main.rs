use std::{env, process::exit};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        println!("Usage: {} <ret-code>", args[0]);
        return;
    }

    let program = args[1].clone();

    println!("\t.section	__TEXT,__text,regular,pure_instructions");
    println!(".globl	_main");
    println!("_main:");
    println!("\tmov x0, #0");

    let mut rest_program: &str = &program;
    let (lhs, rest) = split_digit(&rest_program);
    println!("\tadd x0, x0, #{}", lhs);
    rest_program = rest;

    while !rest_program.is_empty() {
        if rest_program.starts_with("+") {
            let (lhs, rest) = split_digit(&rest_program[1..]);
            println!("\tadd x0, x0, #{}", lhs);
            rest_program = rest;
            continue;
        }

        if rest_program.starts_with("-") {
            let (lhs, rest) = split_digit(&rest_program[1..]);
            println!("\tsub x0, x0, #{}", lhs);
            rest_program = rest;
            continue;
        }

        eprint!(
            "Unexpected character: {}",
            rest_program.chars().next().unwrap()
        );
        exit(1);
    }

    println!("\tret");
}

// https://qiita.com/AtsukiTak/items/0819ee57af2639891ecf
fn split_digit(s: &str) -> (&str, &str) {
    let first_non_num = s.find(|c| !char::is_numeric(c)).unwrap_or(s.len());
    s.split_at(first_non_num)
}
