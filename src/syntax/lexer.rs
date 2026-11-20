use crate::{
    Diagnostic, DiagnosticLabel, ErrorCode,
    source::{SourceId, SourceSpan},
    syntax::{DurationUnit, TokenKind},
};

use super::Token;

pub fn lex(source: &str, source_id: SourceId) -> Result<Vec<Token>, Diagnostic> {
    Lexer::new(source, source_id).tokenize()
}

pub struct Lexer<'source> {
    source: &'source str,
    source_id: SourceId,
    start: usize,
    offset: usize,
    tokens: Vec<Token>,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str, source_id: SourceId) -> Self {
        Self {
            source,
            source_id,
            start: 0,
            offset: 0,
            tokens: Vec::new(),
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, Diagnostic> {
        while let Some(ch) = self.peek() {
            self.start = self.offset;

            match ch {
                ' ' | '\t' | '\r' | '\n' => {
                    self.advance();
                }
                '0'..='9' => {
                    self.number()?;
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    self.identifier()?;
                }
                '+' => {
                    self.advance();
                    self.add_token(TokenKind::Plus);
                }
                '-' => {
                    self.advance();
                    self.add_token(TokenKind::Minus);
                }
                '*' => {
                    self.advance();
                    self.add_token(TokenKind::Star);
                }
                '/' => {
                    self.advance();

                    if self.consume_if('/') {
                        while self.peek().is_some_and(|ch| ch != '\n') {
                            self.advance();
                        }
                    } else {
                        self.add_token(TokenKind::Slash);
                    }
                }
                '(' => {
                    self.advance();
                    self.add_token(TokenKind::LeftParen);
                }
                ')' => {
                    self.advance();
                    self.add_token(TokenKind::RightParen);
                }
                ',' => {
                    self.advance();
                    self.add_token(TokenKind::Comma);
                }
                ':' => {
                    self.advance();
                    self.add_token(TokenKind::Colon);
                }
                '=' => {
                    self.advance();
                    self.add_token(TokenKind::Equal);
                }
                ';' => {
                    self.advance();
                    self.add_token(TokenKind::Semicolon);
                }
                _ => {
                    self.advance();

                    return Err(self.error(
                        ErrorCode::UnknownCharacter,
                        format!("unexpected character {ch:?}"),
                        "this character is not valid in linen source",
                    ));
                }
            }
        }

        self.tokens.push(Token::new(
            TokenKind::Eof,
            SourceSpan::empty(self.source_id, self.offset),
        ));

        Ok(self.tokens)
    }

    fn identifier(&mut self) -> Result<(), Diagnostic> {
        while self.peek().is_some_and(Self::is_identifier_continue) {
            self.advance();
        }

        let identifier = &self.source[self.start..self.offset];
        let kind = match identifier {
            "let" => TokenKind::Let,
            "render" => TokenKind::Render,
            "Float" => TokenKind::Float,
            "Sig" => TokenKind::Sig,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "type" => TokenKind::Type,
            "match" => TokenKind::Match,
            "mod" => TokenKind::Mod,
            "use" => TokenKind::Use,
            "export" => TokenKind::Export,
            "package" => TokenKind::Package,
            "from" => TokenKind::From,
            "as" => TokenKind::As,
            "feedback" => TokenKind::Feedback,
            "event" => TokenKind::Event,
            "mut" => TokenKind::Mut,
            "where" => TokenKind::Where,
            _ => TokenKind::Identifier(identifier.to_string()),
        };
        self.add_token(kind);

        Ok(())
    }

    fn number(&mut self) -> Result<(), Diagnostic> {
        self.consume_digits();

        if self.peek() == Some('.') {
            self.advance();

            if !self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                return Err(self.invalid_number());
            }

            self.consume_digits();
        }

        if matches!(self.peek(), Some('e' | 'E')) {
            self.advance();

            if matches!(self.peek(), Some('+' | '-')) {
                self.advance();
            }

            if !self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                return Err(self.invalid_number());
            }

            self.consume_digits();
        }

        let number_end = self.offset;
        let unit = if self.consume_if('m') {
            if self.consume_if('s') {
                Some(DurationUnit::Milliseconds)
            } else {
                return Err(self.invalid_number());
            }
        } else if self.consume_if('s') {
            Some(DurationUnit::Seconds)
        } else {
            None
        };

        if self
            .peek()
            .is_some_and(|ch| Self::is_identifier_continue(ch) || ch == '.')
        {
            return Err(self.invalid_number());
        }

        let value = self.source[self.start..number_end]
            .parse::<f64>()
            .expect("the numeric grammar was checked before parsing");

        let kind = match unit {
            Some(unit) => TokenKind::Duration { value, unit },
            None => TokenKind::Number(value),
        };
        self.add_token(kind);

        Ok(())
    }

    fn consume_digits(&mut self) {
        while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            self.advance();
        }
    }

    fn invalid_number(&mut self) -> Diagnostic {
        while self
            .peek()
            .is_some_and(|ch| Self::is_identifier_continue(ch) || ch == '.')
        {
            self.advance();
        }

        let literal = &self.source[self.start..self.offset];
        self.error(
            ErrorCode::InvalidNumberOrDuration,
            format!("invalid number or duration `{literal}`"),
            "use forms like `42`, `0.5`, `1e3`, `250ms`, or `2s`",
        )
    }

    fn is_identifier_continue(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_'
    }

    fn consume_if(&mut self, expected: char) -> bool {
        if self.peek() != Some(expected) {
            return false;
        }
        self.advance();
        true
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.offset..)?.chars().next()
    }

    fn add_token(&mut self, kind: TokenKind) {
        let span = SourceSpan::new(self.source_id, self.start, self.offset)
            .expect("the lexer should produce valid spans");
        self.tokens.push(Token::new(kind, span));
    }

    fn error(
        &self,
        code: ErrorCode,
        message: impl Into<String>,
        label: impl Into<String>,
    ) -> Diagnostic {
        let span = SourceSpan::new(self.source_id, self.start, self.offset)
            .expect("the lexer should produce valid spans");
        Diagnostic::error(code, message, DiagnosticLabel::new(span, label))
    }
}
