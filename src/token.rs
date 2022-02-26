#[derive(PartialEq, Eq, Debug)]
pub enum Token {
    Reserved { position: usize, word: String },
    Num { position: usize, value: i32 },
}

impl Token {
    pub fn position(&self) -> usize {
        match self {
            Token::Reserved { position, .. } => *position,
            Token::Num { position, .. } => *position,
        }
    }
}
