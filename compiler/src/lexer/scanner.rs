use crate::error::LinenError;

use super::token::{Token, TokenType};

pub struct Scanner {
    pub source: String,
    pub tokens: Vec<Token>,
    pub start: usize,
    pub current: usize,
    pub line: usize,
    place: String,
}

impl Scanner {
    pub fn new(source: String, place: String) -> Self {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            place,
        }
    }
    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, LinenError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        self.tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: String::from(""),
            line: self.line,
            literal: None,
            source: self.place.clone(),
        });

        Ok(self.tokens.clone())
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<(), LinenError> {
        let c = self.advance();
        match c {
            '-' => self.add_token(TokenType::Symbol, Some(String::from("-"))),
            '+' => self.add_token(TokenType::Symbol, Some(String::from("+"))),
            '*' => self.add_token(TokenType::Symbol, Some(String::from("*"))),
            '/' => self.add_token(TokenType::Symbol, Some(String::from("/"))),
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            '!' => {
                if self.match_next('=') {
                    self.add_token(TokenType::Symbol, Some(String::from("!=")));
                } else {
                    self.add_token(TokenType::Symbol, Some(String::from("!")));
                }
            }
            '=' => {
                if self.match_next('=') {
                    self.add_token(TokenType::Symbol, Some(String::from("==")));
                } else {
                    self.add_token(TokenType::Symbol, Some(String::from("=")));
                }
            }
            '<' => {
                if self.match_next('=') {
                    self.add_token(TokenType::Symbol, Some(String::from("<=")));
                } else {
                    self.add_token(TokenType::Symbol, Some(String::from("<")));
                }
            }
            '>' => {
                if self.match_next('=') {
                    self.add_token(TokenType::Symbol, Some(String::from(">=")));
                } else {
                    self.add_token(TokenType::Symbol, Some(String::from(">")));
                }
            }
            ';' => {
                while self.peek() != '\n' && !self.is_at_end() {
                    self.advance();
                }
            }
            '"' => self.string()?,
            _ => {
                if c.is_ascii_digit() {
                    self.number()?;
                } else if c.is_alphabetic() {
                    self.identifier();
                } else {
                    return Err(LinenError::new(
                        self.place.clone(),
                        self.line,
                        "Unexpected character.".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() {
            self.advance();
        }
        let text = self.source[self.start..self.current].to_string();
        let token_type = match text.as_str() {
            "and" => TokenType::Symbol,
            "false" => TokenType::Boolean,
            "cond" => TokenType::Symbol,
            "nil" => TokenType::Nil,
            "or" => TokenType::Symbol,
            "print" => TokenType::Symbol,
            "true" => TokenType::Boolean,
            "let" => TokenType::Symbol,
            "fn" => TokenType::Symbol,
            "else" => TokenType::Symbol,
            "if" => TokenType::Symbol,
            _ => TokenType::Identifier,
        };

        self.add_token(token_type, Some(text));
    }

    fn number(&mut self) -> Result<(), LinenError> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        let value = self.source[self.start..self.current]
            .parse::<f64>()
            .unwrap();
        self.add_token(TokenType::Number, Some(value.to_string()));
        Ok(())
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap()
        }
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        c
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<String>) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens.push(Token {
            token_type,
            lexeme: text,
            line: self.line,
            literal,
            source: self.place.clone(),
        });
    }

    fn string(&mut self) -> Result<(), LinenError> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            return Err(LinenError::new(
                self.place.clone(),
                self.line,
                "Unterminated string.".to_string(),
            ));
        }
        self.advance();
        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token(TokenType::String, Some(value));
        Ok(())
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap()
        }
    }
}
