use std::fmt::Display;

use crate::source::SourceSpan;

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: SourceSpan,
}

impl Token {
    pub fn new(kind: TokenKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Number(f64),
    Duration { value: f64, unit: DurationUnit },
    Identifier(String),
    Let,
    Render,
    Float,
    Sig,
    True,
    False,
    Type,
    Match,
    Mod,
    Use,
    Export,
    Package,
    From,
    As,
    Feedback,
    Event,
    Mut,
    Where,
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    Equal,
    FatArrow,
    PipeGreater,
    Semicolon,
    Eof,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Number(value) => f.write_str(value.to_string().as_str()),
            TokenKind::Duration { value, unit } => {
                let duration = format!("{}{}", value, unit);

                f.write_str(duration.as_str())
            }
            TokenKind::Identifier(value) => f.write_str(value),
            TokenKind::Let => f.write_str("let"),
            TokenKind::Render => f.write_str("render"),
            TokenKind::Float => f.write_str("Float"),
            TokenKind::Sig => f.write_str("Sig"),
            TokenKind::True => f.write_str("true"),
            TokenKind::False => f.write_str("false"),
            TokenKind::Type => f.write_str("type"),
            TokenKind::Match => f.write_str("match"),
            TokenKind::Mod => f.write_str("mod"),
            TokenKind::Use => f.write_str("use"),
            TokenKind::Export => f.write_str("export"),
            TokenKind::Package => f.write_str("package"),
            TokenKind::From => f.write_str("from"),
            TokenKind::As => f.write_str("as"),
            TokenKind::Feedback => f.write_str("feedback"),
            TokenKind::Event => f.write_str("event"),
            TokenKind::Mut => f.write_str("mut"),
            TokenKind::Where => f.write_str("where"),
            TokenKind::Plus => f.write_str("+"),
            TokenKind::Minus => f.write_str("-"),
            TokenKind::Star => f.write_str("*"),
            TokenKind::Slash => f.write_str("/"),
            TokenKind::LeftParen => f.write_str("("),
            TokenKind::RightParen => f.write_str(")"),
            TokenKind::LeftBrace => f.write_str("{"),
            TokenKind::RightBrace => f.write_str("}"),
            TokenKind::Comma => f.write_str(","),
            TokenKind::Colon => f.write_str(":"),
            TokenKind::Equal => f.write_str("="),
            TokenKind::FatArrow => f.write_str("=>"),
            TokenKind::PipeGreater => f.write_str("|>"),
            TokenKind::Semicolon => f.write_str(";"),
            TokenKind::Eof => f.write_str("\n"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurationUnit {
    Milliseconds,
    Seconds,
}

impl DurationUnit {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Milliseconds => "ms",
            Self::Seconds => "s",
        }
    }
}

impl Display for DurationUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
