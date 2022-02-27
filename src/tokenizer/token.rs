#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TokenKind {
    Plus,
    Minus,
    Mul,
    Div,
    Num,
    LParen,
    RParen,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
}

impl TokenKind {
    fn is_op(&self) -> bool {
        match self {
            TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Mul
            | TokenKind::Div
            | TokenKind::LessThan
            | TokenKind::GreaterThan
            | TokenKind::GreaterThanOrEqual
            | TokenKind::Equal
            | TokenKind::NotEqual
            | TokenKind::LessThanOrEqual => true,
            _ => false,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Token {
    pub position: usize,
    pub kind: TokenKind,
    pub num: Option<i32>, // Number
}

impl Token {
    pub fn new_syntax_item(position: usize, kind: TokenKind) -> Self {
        Self {
            position,
            kind,
            num: None,
        }
    }

    pub fn new_num(position: usize, num: i32) -> Self {
        Self {
            position,
            kind: TokenKind::Num,
            num: Some(num),
        }
    }
}
