#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier,
    String,
    Number,
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Error,
    EOF,
}

#[derive(Clone, Debug)]
pub struct Token<'a> {
    pub token_type: TokenType,
    pub length: usize,
    pub line: usize,
    pub start: &'a str,
}

impl<'a> Token<'a> {
    pub fn new(token_type: TokenType, length: usize, line: usize, start: &'a str) -> Self {
        Token {
            token_type,
            length,
            line,
            start,
        }
    }
}
