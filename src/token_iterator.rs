use std::{error, fmt, process::exit};

use crate::{error_report::error_at, token::Token};

#[derive(Debug)]
pub enum TokenizeError {
    UnknownToken { position: usize },
}

impl fmt::Display for TokenizeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unknown token")
    }
}

impl error::Error for TokenizeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

pub struct TokenIterator<'a> {
    pub original_input: &'a str,
    input: &'a str,
    pos: usize,
}

impl<'a> TokenIterator<'a> {
    pub fn new(input: &'a str) -> TokenIterator<'a> {
        TokenIterator {
            original_input: input,
            input,
            pos: 0,
        }
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
            self.input = &self.input[1..];
            self.pos += 1;
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

    pub fn expect_num(&mut self) -> i32 {
        let next_token = self.next();
        if let Some(t) = next_token {
            match t {
                Ok(t) => match t {
                    Token::Num { value, .. } => value,
                    _ => {
                        error_at(
                            self.original_input,
                            t.position(),
                            "Unexpected token. Number expected",
                        );
                        exit(1);
                    }
                },
                Err(err) => {
                    error_at(self.original_input, self.pos, &err.to_string());
                    exit(1);
                }
            }
        } else {
            error_at(
                self.original_input,
                self.pos,
                "Unexpected EOF expected number",
            );
            exit(1);
        }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Result<Token, TokenizeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let current_position = self.pos;
        if self.input.is_empty() {
            return None;
        }

        if self.try_consume('+') {
            return Some(Ok(Token::Reserved {
                position: current_position,
                word: "+".to_string(),
            }));
        }

        if self.try_consume('-') {
            return Some(Ok(Token::Reserved {
                position: current_position,
                word: "-".to_string(),
            }));
        }

        if let Some(num) = self.try_consume_digits() {
            return Some(Ok(Token::Num {
                position: current_position,
                value: num,
            }));
        }

        Some(Err(TokenizeError::UnknownToken {
            position: current_position,
        }
        .into()))
    }
}
