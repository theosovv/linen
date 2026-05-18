//! Lexer for the Linen programming language.
//!
//! Uses the `logos` crate for fast tokenization with automatic span tracking.

pub use logos::Logos;

use linen_core::span::Span;

/// A token in the Linen language.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"--[^\n]*")]
pub enum Token {
    // Keywords
    #[token("let")]
    Let,
    #[token("fn")]
    Fn,
    #[token("type")]
    Type,
    #[token("match")]
    Match,
    #[token("with")]
    With,
    #[token("import")]
    Import,
    #[token("export")]
    Export,
    #[token("as")]
    As,
    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("true")]
    True,
    #[token("false")]
    False,

    // Operators
    #[token("|")]
    Pipe,
    #[token("|>")]
    Pipeline,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("==")]
    Eq,
    #[token("!=")]
    Neq,
    #[token("<=")]
    Lte,
    #[token(">=")]
    Gte,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("&&")]
    And,
    #[token("||")]
    Or,

    // Delimiters
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token("=")]
    Equals,
    #[token(":")]
    Colon,
    #[token(".")]
    Dot,

    // Literals
    #[regex(r"[0-9]+", |lex| lex.slice().parse().ok())]
    Int(i64),
    #[regex(r"[0-9]+\.[0-9]*([eE][+-]?[0-9]+)?", |lex| lex.slice().parse().ok())]
    Float(f64),
    #[regex("\"([^\"\\\\]|\\\\.)*\"", |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    String(String),

    // Identifiers (including `_`)
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    // Block comments (skipped manually in lex())
    #[regex(r"\{-(?:[^-]|-[^}])*-\}")]
    BlockComment,
}

/// A token with its source span.
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

/// Lex a source string into a vector of spanned tokens.
pub fn lex(input: &str) -> Result<Vec<SpannedToken>, linen_core::error::LexError> {
    let mut lexer = Token::lexer(input);
    let mut tokens = Vec::new();

    while let Some(result) = lexer.next() {
        let span = Span::new(lexer.span().start, lexer.span().end);
        match result {
            Ok(Token::BlockComment) => {
                // Skip block comments
                continue;
            }
            Ok(token) => tokens.push(SpannedToken { token, span }),
            Err(()) => {
                let ch = input.chars().nth(span.start).unwrap_or('\0');
                return Err(linen_core::error::LexError::UnexpectedChar(ch, span));
            }
        }
    }

    Ok(tokens)
}
