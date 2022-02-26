use std::env;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        println!("Usage: {} <ret-code>", args[0]);
        return;
    }

    let ret_code = args[1].parse::<i32>().unwrap();
    println!("\t.section	__TEXT,__text,regular,pure_instructions");
    println!(".globl	_main");
    println!("_main:");
    println!("\tmov x0, #{}", ret_code);
    println!("\tret");
}
