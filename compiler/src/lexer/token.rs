#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Identifier,
    String,
    Number,
    Boolean,
    Nil,
    Comment,
    Symbol,
    Else,
    Eof,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub literal: Option<String>,
    pub source: String,
}
