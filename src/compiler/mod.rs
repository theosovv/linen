use std::{borrow::BorrowMut, collections::HashMap};

use parser::{Parser, Precedence};
use scanner::Scanner;
use token::{Token, TokenType};

use crate::vm::chunk::{debug::disassemble_chunk, Chunk, OpCode};

pub mod parser;
pub mod scanner;
pub mod token;
pub mod utils;

#[derive(Debug)]
pub struct ParseRule<'a> {
    pub prefix: Option<fn(&mut Compiler<'a>)>,
    pub infix: Option<fn(&mut Compiler<'a>)>,
    pub precedence: Precedence,
}

#[derive(Debug)]
pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    parser: Parser<'a>,
    current_chunk: Option<Chunk>,
    rules: HashMap<TokenType, ParseRule<'a>>,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str) -> Self {
        let rules = utils::get_rules();

        Compiler {
            scanner: Scanner::new(source),
            parser: Parser::new(None, None),
            current_chunk: None,
            rules,
        }
    }

    pub fn compile(&mut self, chunk: &mut Chunk) -> bool {
        self.current_chunk = Some(chunk.clone());
        self.advance();
        self.expression();

        self.end_compiler();

        *chunk = self.current_chunk.clone().unwrap();

        !self.parser.had_error
    }

    fn number(&mut self) {
        let value = self.parser.previous.clone().unwrap().start
            [0..self.parser.previous.clone().unwrap().length]
            .parse::<f64>()
            .unwrap();
        self.emit_constant(value);
    }

    fn emit_constant(&mut self, value: f64) {
        let constant = self.make_constant(value);
        self.emit_bytes(OpCode::OpConstant as u8, constant);
        self.parser.previous.clone().unwrap();
    }

    fn make_constant(&mut self, value: f64) -> u8 {
        let constant = self.current_chunk.as_mut().unwrap().add_constant(value);

        if constant > 255 {
            self.error_at_current("Too many constants in one chunk.");
            return 0;
        }

        constant as u8
    }

    fn consume(&mut self, token_type: TokenType, message: &'a str) {
        if self.parser.current.clone().unwrap().token_type == token_type {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn emit_byte(&mut self, byte: u8) {
        if let Some(chunk) = self.current_chunk.as_mut() {
            chunk.write(byte, self.parser.previous.clone().unwrap().line);
        }
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn end_compiler(&mut self) {
        self.emit_return();

        if !self.parser.had_error {
            disassemble_chunk(self.current_chunk.as_ref().unwrap(), "code");
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let rule = self.rules[&self.parser.previous.clone().unwrap().token_type].prefix;

        if let Some(rule) = rule {
            rule(self);
        }

        while precedence <= self.rules[&self.parser.current.clone().unwrap().token_type].precedence
        {
            self.advance();
            let rule = self.rules[&self.parser.previous.clone().unwrap().token_type].infix;

            if let Some(rule) = rule {
                rule(self);
            } else {
                break;
            }
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn grouping_expression(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary_expression(&mut self) {
        let operator_type = self.parser.previous.clone().unwrap().token_type;

        self.expression();

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::OpNegate as u8),
            TokenType::Bang => self.emit_byte(OpCode::OpSubtract as u8),
            _ => panic!("Invalid unary operator"),
        }
    }

    fn binary_expression(&mut self) {
        let operator_type = self.parser.previous.clone().unwrap().token_type;
        let rule = self.rules.get(&operator_type).unwrap();

        self.parse_precedence(rule.precedence.clone());

        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::OpSubtract as u8),
            TokenType::Plus => self.emit_byte(OpCode::OpAdd as u8),
            TokenType::Star => self.emit_byte(OpCode::OpMultiply as u8),
            TokenType::Slash => self.emit_byte(OpCode::OpDivide as u8),
            _ => panic!("Invalid binary operator"),
        }
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::OpReturn as u8);
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current.take();

        loop {
            let token = self.scanner.scan_token();
            if token.token_type != TokenType::Error {
                self.parser.current = Some(token);
                break;
            }

            let error_message = token.start;
            self.error_at_current(error_message);
        }
    }

    fn error_at_current(&mut self, message: &'a str) {
        self.error_at(self.parser.current.clone().unwrap(), message);
    }

    fn error_at(&mut self, token: Token<'a>, message: &'a str) {
        self.parser.panic_mode = true;

        print!("[line {}] Error", token.line);

        if token.token_type == TokenType::EOF {
            print!(" at end");
        } else if token.token_type == TokenType::Error {
            // Nothing.
        } else {
            print!(" at '{}'", token.start);
        }

        println!(": {}", message);
        self.parser.had_error = true;
    }
}
