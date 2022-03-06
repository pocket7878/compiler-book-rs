#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TokenKind {
    Plus,
    Minus,
    Star,
    Div,
    Num,
    Ident,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Assign,
    Equal,
    NotEqual,
    Semicolon,
    Return,
    If,
    Else,
    While,
    For,
    Comma,
    Ampersand,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Token {
    pub position: usize,
    pub kind: TokenKind,
    pub num: Option<i32>,    // Number
    pub str: Option<String>, // Identifier
}

impl Token {
    pub fn new_syntax_item(position: usize, kind: TokenKind) -> Self {
        Self {
            position,
            kind,
            num: None,
            str: None,
        }
    }

    pub fn new_num(position: usize, num: i32) -> Self {
        Self {
            position,
            kind: TokenKind::Num,
            num: Some(num),
            str: None,
        }
    }

    pub fn new_ident(position: usize, str: &str) -> Self {
        Self {
            position,
            kind: TokenKind::Ident,
            num: None,
            str: Some(str.to_string()),
        }
    }
}
