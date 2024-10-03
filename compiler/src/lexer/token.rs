#[derive(Clone, Debug)]
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
    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub literal: Option<String>,
    pub source: String,
}
