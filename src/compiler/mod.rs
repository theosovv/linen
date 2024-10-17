use std::{borrow::BorrowMut, collections::HashMap};

use parser::{Parser, Precedence};
use scanner::Scanner;
use token::{Token, TokenType};

use crate::vm::chunk::{
    debug::disassemble_chunk,
    object::{Obj, ObjectType, StringObject},
    value::Val,
    Chunk, OpCode,
};

pub mod parser;
pub mod scanner;
pub mod token;
pub mod utils;

#[derive(Debug)]
pub struct ParseRule<'a> {
    pub prefix: Option<fn(&mut Compiler<'a>, Option<bool>)>,
    pub infix: Option<fn(&mut Compiler<'a>, Option<bool>)>,
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

        while !self.match_token(TokenType::EOF) {
            self.declaration();

            if self.parser.panic_mode {
                self.synchronize();
            }
        }

        self.end_compiler();

        *chunk = self.current_chunk.clone().unwrap();

        !self.parser.had_error
    }

    fn declaration(&mut self) {
        if self.match_token(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }
    }

    fn variable(&mut self, can_assign: Option<bool>) {
        self.named_variable(self.parser.previous.clone().unwrap(), can_assign);
    }

    fn named_variable(&mut self, name: Token<'a>, can_assign: Option<bool>) {
        let arg = self.identifier_constant(name);

        if can_assign.unwrap() && self.match_token(TokenType::Equal) {
            self.expression();
            self.emit_bytes(OpCode::OpSetGlobal as u8, arg);
        } else {
            self.emit_bytes(OpCode::OpGetGlobal as u8, arg);
        }
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expected variable name.");

        if self.match_token(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::OpNil as u8);
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );

        self.define_variable(global);
    }

    fn define_variable(&mut self, global: u8) {
        self.emit_bytes(OpCode::OpDefineGlobal as u8, global);
    }

    fn parse_variable(&mut self, error_message: &'a str) -> u8 {
        self.consume(TokenType::Identifier, error_message);

        self.identifier_constant(self.parser.previous.clone().unwrap())
    }

    fn identifier_constant(&mut self, name: Token<'a>) -> u8 {
        self.make_constant(Val::object(Obj::String(StringObject::new(
            &name.start[0..name.length],
        ))))
    }

    fn statement(&mut self) {
        if self.match_token(TokenType::Print) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    fn synchronize(&mut self) {
        self.parser.panic_mode = false;

        while self.parser.current.clone().unwrap().token_type != TokenType::EOF {
            if self.parser.previous.clone().unwrap().token_type == TokenType::Semicolon {
                return;
            }

            match self.parser.current.clone().unwrap().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::OpPop as u8);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(OpCode::OpPrint as u8);
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if !self.check(token_type) {
            return false;
        }
        self.advance();
        true
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        self.parser.current.clone().unwrap().token_type == token_type
    }

    fn literal(&mut self, _: Option<bool>) {
        match self.parser.previous.clone().unwrap().token_type {
            TokenType::False => self.emit_byte(OpCode::OpFalse as u8),
            TokenType::True => self.emit_byte(OpCode::OpTrue as u8),
            TokenType::Nil => self.emit_byte(OpCode::OpNil as u8),
            _ => panic!("Invalid literal"),
        }
    }

    fn string(&mut self, _: Option<bool>) {
        let value = self.parser.previous.clone().unwrap().start
            [0..self.parser.previous.clone().unwrap().length]
            .to_string();

        let value = value.trim_matches('"');

        self.emit_constant(Val::object(Obj::String(StringObject::new(value))));
    }

    fn number(&mut self, _: Option<bool>) {
        let value = self.parser.previous.clone().unwrap().start
            [0..self.parser.previous.clone().unwrap().length]
            .parse::<f64>()
            .unwrap();

        self.emit_constant(Val::number(value));
    }

    fn emit_constant(&mut self, value: Val) {
        let constant = self.make_constant(value);
        self.emit_bytes(OpCode::OpConstant as u8, constant);
        self.parser.previous.clone().unwrap();
    }

    fn make_constant(&mut self, value: Val) -> u8 {
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
        if !self.parser.had_error {
            disassemble_chunk(self.current_chunk.as_ref().unwrap(), "code");
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let rule = self.rules[&self.parser.previous.clone().unwrap().token_type].prefix;
        let can_assign = precedence <= Precedence::Assignment;

        if let Some(rule) = rule {
            rule(self, Some(can_assign));
        }

        while precedence <= self.rules[&self.parser.current.clone().unwrap().token_type].precedence
        {
            self.advance();
            let rule = self.rules[&self.parser.previous.clone().unwrap().token_type].infix;

            if let Some(rule) = rule {
                rule(self, Some(can_assign));
            } else {
                break;
            }
        }

        if can_assign && self.match_token(TokenType::Equal) {
            self.error_at_current("Invalid assignment target.");
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn grouping_expression(&mut self, _: Option<bool>) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary_expression(&mut self, _: Option<bool>) {
        let operator_type = self.parser.previous.clone().unwrap().token_type;

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::OpNegate as u8),
            TokenType::Bang => self.emit_byte(OpCode::OpNot as u8),
            _ => panic!("Invalid unary operator"),
        }
    }

    fn binary_expression(&mut self, _: Option<bool>) {
        let operator_type = self.parser.previous.clone().unwrap().token_type;
        let rule = self.rules.get(&operator_type).unwrap();

        self.parse_precedence(rule.precedence.clone());

        match operator_type {
            TokenType::BangEqual => self.emit_bytes(OpCode::OpEqual as u8, OpCode::OpNot as u8),
            TokenType::EqualEqual => self.emit_byte(OpCode::OpEqual as u8),
            TokenType::Greater => self.emit_byte(OpCode::OpGreater as u8),
            TokenType::GreaterEqual => self.emit_bytes(OpCode::OpLess as u8, OpCode::OpNot as u8),
            TokenType::Less => self.emit_byte(OpCode::OpLess as u8),
            TokenType::LessEqual => self.emit_bytes(OpCode::OpGreater as u8, OpCode::OpNot as u8),
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
