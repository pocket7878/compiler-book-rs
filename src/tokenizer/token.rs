#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TokenKind {
    Plus,
    Minus,
    Mul,
    Div,
    Num,
    LParen,
    RParen,
}

impl TokenKind {
    fn is_op(&self) -> bool {
        match self {
            TokenKind::Plus | TokenKind::Minus | TokenKind::Mul | TokenKind::Div => true,
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
    pub fn new_op(position: usize, kind: TokenKind) -> Self {
        if !kind.is_op() {
            panic!("{:?} is not an operator TokenKind", kind);
        }

        Self {
            position,
            kind,
            num: None,
        }
    }

    pub fn new_paren(position: usize, kind: TokenKind) -> Self {
        if kind == TokenKind::LParen || kind == TokenKind::RParen {
            return Self {
                position,
                kind,
                num: None,
            };
        }

        panic!("{:?} is not an paren TokenKind", kind);
    }

    pub fn new_num(position: usize, num: i32) -> Self {
        Self {
            position,
            kind: TokenKind::Num,
            num: Some(num),
        }
    }
}
