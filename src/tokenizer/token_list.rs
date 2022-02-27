use std::process::exit;

use crate::error_report::error_at;

use super::token::{Token, TokenKind};

#[derive(Debug)]
pub struct TokenList<'a> {
    original_input: &'a str,
    tokens: Vec<Token>,
    pos: usize,
}

impl<'a> TokenList<'a> {
    pub fn new(original_input: &'a str, tokens: Vec<Token>) -> Self {
        Self {
            original_input,
            tokens,
            pos: 0,
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        if self.pos >= self.tokens.len() {
            None
        } else {
            let token = self.tokens[self.pos].clone();
            self.advance();
            Some(token)
        }
    }

    pub fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    pub fn peek(&self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            Some(self.tokens[self.pos].clone())
        } else {
            None
        }
    }

    pub fn try_consume(&mut self, kind: &TokenKind) -> Option<Token> {
        let next = self.peek();
        if let Some(next) = next {
            if next.kind == *kind {
                self.next()
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn expect_kind(&mut self, kind: &TokenKind) {
        let next = self.peek();
        if let Some(next) = next {
            if next.kind == *kind {
                self.advance();
            } else {
                self.exit_with_unexpected_token(next.position, &format!("Expected {:?}", kind));
            }
        } else {
            self.exit_with_unexpected_eof(&format!("Expected {:?}", kind));
        }
    }

    pub fn expect_num(&mut self) -> i32 {
        let next_token = self.peek();
        match next_token {
            Some(token) => {
                if token.kind == TokenKind::Num {
                    self.advance();
                    token.num.unwrap()
                } else {
                    self.exit_with_unexpected_token(token.position, "Expected number");
                }
            }
            None => {
                self.exit_with_unexpected_eof("Expected number");
            }
        }
    }

    fn exit_with_unexpected_token(&self, position: usize, additional_message: &str) -> ! {
        if additional_message.is_empty() {
            error_at(self.original_input, position, "Unexpected Token");
        } else {
            error_at(
                self.original_input,
                position,
                &format!("Unexpected Token, {}", additional_message),
            );
        }
        exit(1)
    }

    fn exit_with_unexpected_eof(&self, additional_message: &str) -> ! {
        if additional_message.is_empty() {
            error_at(
                self.original_input,
                self.original_input.len(),
                "Unexpected EOF",
            );
        } else {
            error_at(
                self.original_input,
                self.original_input.len(),
                &format!("Unexpected EOF, {}", additional_message),
            );
        }
        exit(1)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }
}
