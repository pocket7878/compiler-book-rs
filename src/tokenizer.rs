mod token;
mod token_list;
use std::process::exit;

use crate::error_report::error_at;

pub use self::{
    token::{Token, TokenKind},
    token_list::TokenList,
};

pub struct Tokenizer<'a> {
    original_input: &'a str,
    input: &'a str,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Tokenizer<'a> {
        Self {
            original_input: input,
            input,
            pos: 0,
        }
    }

    pub fn tokenize(mut self) -> TokenList<'a> {
        let mut tokens = vec![];

        loop {
            self.skip_whitespace();

            if self.input.is_empty() {
                break;
            }

            let current_position = self.pos;
            if self.try_consume('+') {
                tokens.push(Token::new_op(current_position, TokenKind::Plus));
                continue;
            }
            if self.try_consume('-') {
                tokens.push(Token::new_op(current_position, TokenKind::Minus));
                continue;
            }
            if self.try_consume('*') {
                tokens.push(Token::new_op(current_position, TokenKind::Mul));
                continue;
            }
            if self.try_consume('/') {
                tokens.push(Token::new_op(current_position, TokenKind::Div));
                continue;
            }
            if self.try_consume('(') {
                tokens.push(Token::new_paren(current_position, TokenKind::LParen));
                continue;
            }
            if self.try_consume(')') {
                tokens.push(Token::new_paren(current_position, TokenKind::RParen));
                continue;
            }
            if let Some(num) = self.try_consume_digits() {
                tokens.push(Token::new_num(current_position, num));
                continue;
            }

            // 単純化のため、トークン化できなかったら即終了させる
            error_at(self.original_input, current_position, "Unrecognized token");
            exit(1)
        }

        TokenList::new(self.original_input, tokens)
    }

    fn skip_whitespace(&mut self) {
        use itertools::Itertools;

        let mut chars = self.input.chars();
        let spaces = chars
            .take_while_ref(|c| c.is_whitespace())
            .collect::<String>();
        self.pos += spaces.chars().count();
        self.input = chars.as_str();
    }

    fn try_consume(&mut self, c: char) -> bool {
        if self.input.starts_with(c) {
            self.pos += 1;
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
                self.pos += digit_str.chars().count();
                Some(num)
            }
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn tokenize_single_digit_num() {
        let expr = "1";
        let mut token_list = super::Tokenizer::new(expr).tokenize();
        assert_eq!(token_list.peek().unwrap().kind, super::TokenKind::Num);
        assert_eq!(token_list.peek().unwrap().num.unwrap(), 1);
        token_list.next();
        assert_eq!(token_list.peek().is_none(), true);
    }

    #[test]
    fn tokenize_multiple_digit_num() {
        let expr = "1234";
        let mut token_list = super::Tokenizer::new(expr).tokenize();
        assert_eq!(token_list.peek().unwrap().kind, super::TokenKind::Num);
        assert_eq!(token_list.peek().unwrap().num.unwrap(), 1234);
        token_list.next();
        assert_eq!(token_list.peek().is_none(), true);
    }

    #[test]
    fn tokenize_operators() {
        let expr = "+-*/";
        let mut token_list = super::Tokenizer::new(expr).tokenize();
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Plus);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Minus);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Mul);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Div);
    }

    #[test]
    fn tokenize_parens() {
        let expr = "()";
        let mut token_list = super::Tokenizer::new(expr).tokenize();
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::LParen);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::RParen);
    }

    #[test]
    fn skip_whitespaces() {
        let expr = "5 + 20 - 4";
        let mut token_list = super::Tokenizer::new(expr).tokenize();
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Num);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Plus);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Num);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Minus);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Num);
    }
}
