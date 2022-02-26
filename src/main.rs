use std::env;

#[derive(PartialEq, Eq, Debug)]
enum Token {
    Reserved(String),
    Num(i32),
    Eof,
}

struct TokenIterator<'a> {
    input: &'a str,
}

impl<'a> TokenIterator<'a> {
    fn new(input: &'a str) -> TokenIterator<'a> {
        TokenIterator { input }
    }

    fn skip_whitespace(&mut self) {
        self.input = self.input.trim_start();
    }

    fn try_consume(&mut self, c: char) -> bool {
        if self.input.starts_with(c) {
            self.input = &self.input[1..];
            true
        } else {
            false
        }
    }

    fn try_consume_digits(&mut self) -> Option<i32> {
        let first_non_num = self
            .input
            .find(|c| !char::is_numeric(c))
            .unwrap_or(self.input.len());
        let (digit_str, rest_input) = self.input.split_at(first_non_num);

        match digit_str.parse::<i32>() {
            Ok(num) => {
                self.input = rest_input;
                Some(num)
            }
            Err(_) => None,
        }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        if self.input.is_empty() {
            return Some(Token::Eof);
        }

        if self.try_consume('+') {
            return Some(Token::Reserved("+".to_string()));
        }

        if self.try_consume('-') {
            return Some(Token::Reserved("-".to_string()));
        }

        if let Some(num) = self.try_consume_digits() {
            return Some(Token::Num(num));
        }

        eprintln!(
            "Unexpected character '{}'",
            self.input.chars().next().unwrap()
        );
        None
    }
}

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

    let mut token_iter = TokenIterator::new(&program);
    let head_token = token_iter.next();
    if let Some(Token::Num(n)) = head_token {
        println!("\tadd x0, x0, #{}", n);
    } else {
        eprintln!("Unexpected first token: {:?}", head_token);
    }

    while let Some(token) = token_iter.next() {
        match token {
            Token::Reserved(s) => {
                if s == "+" {
                    match token_iter.next() {
                        Some(Token::Num(n)) => {
                            println!("\tadd x0, x0, #{}", n);
                        }
                        n => {
                            eprintln!("Unexpected token after +: {:?}", n);
                        }
                    }
                } else if s == "-" {
                    match token_iter.next() {
                        Some(Token::Num(n)) => {
                            println!("\tsub x0, x0, #{}", n);
                        }
                        n => {
                            eprintln!("Unexpected token after -: {:?}", n);
                        }
                    }
                } else {
                    eprintln!("Unexpected reserved token: {}", s);
                }
            }
            Token::Eof => break,
            _ => panic!("Unexpected token: {:?}", token),
        }
    }

    println!("\tret");
}
