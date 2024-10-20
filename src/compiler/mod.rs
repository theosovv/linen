use std::collections::HashMap;

use parser::{Parser, Precedence};
use scanner::Scanner;
use token::{Token, TokenType};

use crate::vm::chunk::{
    debug::disassemble_chunk,
    object::{FunctionObject, Obj, StringObject},
    value::Val,
    Chunk, OpCode,
};

pub mod parser;
pub mod scanner;
pub mod token;
pub mod utils;

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionType {
    Function,
    Script,
}

#[derive(Debug, Clone)]
pub struct ParseRule<'a> {
    pub prefix: Option<fn(&mut Compiler<'a>, Option<bool>)>,
    pub infix: Option<fn(&mut Compiler<'a>, Option<bool>)>,
    pub precedence: Precedence,
}

#[derive(Debug, Clone)]
pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    parser: Parser<'a>,
    current_chunk: Option<Chunk>,
    rules: HashMap<TokenType, ParseRule<'a>>,
    locals: Vec<Local<'a>>,
    scope_depth: usize,
    function: FunctionObject,
    function_type: FunctionType,
}
#[derive(Debug, Clone)]
pub struct Local<'a> {
    pub name: Token<'a>,
    pub depth: usize,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str, function_type: FunctionType) -> Self {
        let rules = utils::get_rules();

        let compiler = Compiler {
            scanner: Scanner::new(source),
            parser: Parser::new(None, None),
            current_chunk: None,
            rules,
            locals: Vec::new(),
            scope_depth: 0,
            function: FunctionObject::new(),
            function_type: function_type.clone(),
        };

        compiler
    }

    pub fn compile(&mut self, chunk: Chunk) -> Result<FunctionObject, String> {
        self.current_chunk = Some(chunk);
        self.advance();

        while !self.match_token(TokenType::EOF) {
            self.declaration();

            if self.parser.panic_mode {
                self.synchronize();
            }
        }

        let function = self.end_compiler();

        if self.parser.had_error {
            Err("Compilation failed".to_string())
        } else {
            Ok(function)
        }
    }

    fn declaration(&mut self) {
        if self.match_token(TokenType::Fun) {
            self.fun_declaration();
        } else if self.match_token(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }
    }

    fn fun_declaration(&mut self) {
        let global = self.parse_variable("Expect function name");
        self.mark_initialized();

        self.function(FunctionType::Function);

        self.define_variable(global);
    }

    fn call(&mut self, _: Option<bool>) {
        let arg_count = self.argument_list();
        self.emit_bytes(OpCode::Call as u8, arg_count);
    }

    fn argument_list(&mut self) -> u8 {
        let mut arg_count = 0;
        if !self.check(TokenType::RightParen) {
            loop {
                self.expression();
                if arg_count == 255 {
                    self.error_at_current("Can't have more than 255 arguments.");
                    return 0;
                }
                arg_count += 1;

                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after arguments.");
        arg_count
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.function.chunk
    }

    fn function(&mut self, function_type: FunctionType) {
        let mut compiler = Compiler::new(self.scanner.source, function_type);

        // self.enclosing = Some(Box::new(enclosing.clone()));

        compiler.parser = self.parser.clone();
        compiler.scanner = self.scanner.clone();
        compiler.current_chunk = Some(Chunk::new());

        let length = compiler.parser.previous.as_ref().unwrap().length;
        let name = compiler.parser.previous.as_ref().unwrap().start[0..length].to_string();
        compiler.function.name = StringObject::new(&name);

        compiler.begin_scope();

        compiler.consume(TokenType::LeftParen, "Expect '(' after function name.");
        if !compiler.check(TokenType::RightParen) {
            loop {
                compiler.function.arity += 1;

                if compiler.function.arity > 255 {
                    compiler.error_at_current("Can't have more than 255 parameters.");
                }

                let constant = compiler.parse_variable("Expect parameter name");
                compiler.define_variable(constant);

                if !compiler.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        compiler.consume(TokenType::RightParen, "Expect ')' after parameters.");
        compiler.consume(TokenType::LeftBrace, "Expect '{' before function body.");

        compiler.block();

        self.scanner = compiler.scanner.clone();
        self.parser = compiler.parser.clone();

        let function = compiler.end_compiler();
        // *self = enclosing;
        let constant = self.make_constant(Val::object(Obj::Function(function)));

        self.emit_bytes(OpCode::OpConstant as u8, constant);
    }
    fn variable(&mut self, can_assign: Option<bool>) {
        self.named_variable(self.parser.previous.clone().unwrap(), can_assign.unwrap());
    }

    fn named_variable(&mut self, name: Token<'a>, can_assign: bool) {
        if let Some(local_index) = self.resolve_local(&name) {
            // Variable is local
            if can_assign && self.match_token(TokenType::Equal) {
                self.expression();
                self.emit_bytes(OpCode::OpSetLocal as u8, local_index as u8);
            } else {
                self.emit_bytes(OpCode::OpGetLocal as u8, local_index as u8);
            }
        } else {
            // Variable is global
            let arg = self.identifier_constant(name);
            if can_assign && self.match_token(TokenType::Equal) {
                self.expression();
                self.emit_bytes(OpCode::OpSetGlobal as u8, arg);
            } else {
                self.emit_bytes(OpCode::OpGetGlobal as u8, arg);
            }
        }
    }
    fn resolve_local(&mut self, name: &Token<'a>) -> Option<usize> {
        println!("Resolving variable '{}'", &name.start[0..name.length]);
        for (i, local) in self.locals.iter().enumerate().rev() {
            if self.identifiers_equal(name, &local.name) {
                if local.depth == usize::MAX {
                    self.error_at_current("Can't read local variable in its own initializer.");
                }
                return Some(i);
            }
        }
        None
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
        if self.scope_depth > 0 {
            self.mark_initialized();
            return;
        }
        self.emit_bytes(OpCode::OpDefineGlobal as u8, global);
    }

    fn mark_initialized(&mut self) {
        if self.scope_depth == 0 {
            return;
        }

        if let Some(local) = self.locals.last_mut() {
            local.depth = self.scope_depth;
        }
    }

    fn parse_variable(&mut self, error_message: &'a str) -> u8 {
        self.consume(TokenType::Identifier, error_message);

        self.declare_variable();

        if self.scope_depth > 0 {
            return 0;
        }

        self.identifier_constant(self.parser.previous.clone().unwrap())
    }

    fn declare_variable(&mut self) {
        if self.scope_depth == 0 {
            return;
        }

        let name = self.parser.previous.clone().unwrap();

        for local in self.locals.iter().rev() {
            if local.depth < self.scope_depth {
                break;
            }

            if self.identifiers_equal(&local.name, &name) {
                self.error_at_current("Already a variable with this name in this scope.");
                return;
            }
        }

        self.add_local(name);
    }
    fn identifiers_equal(&self, a: &Token<'a>, b: &Token<'a>) -> bool {
        if a.length != b.length {
            return false;
        }

        let a_string = &a.start[0..a.length];
        let b_string = &b.start[0..a.length];

        if a_string != b_string {
            return false;
        }

        true
    }

    fn add_local(&mut self, name: Token<'a>) {
        if self.locals.len() >= u8::MAX as usize {
            self.error_at_current("Too many local variables in function.");
            return;
        }

        println!(
            "Added local variable '{}' at depth {}",
            &name.start[0..name.length],
            self.scope_depth
        );

        self.locals.push(Local {
            name,
            depth: usize::MAX,
        });
    }

    fn identifier_constant(&mut self, name: Token<'a>) -> u8 {
        self.make_constant(Val::object(Obj::String(StringObject::new(
            &name.start[0..name.length],
        ))))
    }

    fn statement(&mut self) {
        if self.match_token(TokenType::Print) {
            self.print_statement();
        } else if self.match_token(TokenType::For) {
            self.for_statement();
        } else if self.match_token(TokenType::If) {
            self.if_statement();
        } else if self.match_token(TokenType::Return) {
            self.return_statement();
        } else if self.match_token(TokenType::While) {
            self.while_statement();
        } else if self.match_token(TokenType::LeftBrace) {
            self.block();
        } else {
            self.expression_statement();
        }
    }

    fn return_statement(&mut self) {
        if self.function_type == FunctionType::Script {
            self.error_at_current("Can't return from top-level code.");
        }

        if self.match_token(TokenType::Semicolon) {
            self.emit_return();
        } else {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after return value.");
            self.emit_byte(OpCode::OpReturn as u8);
        }
    }

    fn for_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");

        if self.match_token(TokenType::Semicolon) {
            // No initializer
        } else if self.match_token(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        self.begin_scope();

        let mut loop_start = self.current_chunk().code.len();

        let mut exit_jump = None;
        if !self.match_token(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");

            exit_jump = Some(self.emit_jump(OpCode::OpJumpFalse as u8));
            self.emit_byte(OpCode::OpPop as u8);
        }

        if !self.match_token(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::OpJump as u8);
            let increment_start = self.current_chunk().code.len();
            self.expression();
            self.emit_byte(OpCode::OpPop as u8);

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.consume(TokenType::RightParen, "Expect ')' after for clauses.");

        self.statement();

        self.emit_loop(loop_start);

        if let Some(exit_jump) = exit_jump {
            self.patch_jump(exit_jump);
            self.emit_byte(OpCode::OpPop as u8);
        }

        self.end_scope();
    }

    fn while_statement(&mut self) {
        let loop_start = self.current_chunk().code.len();
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let exit_jump = self.emit_jump(OpCode::OpJumpFalse as u8);
        self.emit_byte(OpCode::OpPop as u8);
        self.statement();

        self.emit_loop(loop_start);
        self.patch_jump(exit_jump);
        self.emit_byte(OpCode::OpPop as u8);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_byte(OpCode::OpLoop as u8);

        let offset = self.current_chunk().code.len() - loop_start + 2;
        if offset > u16::MAX as usize {
            self.error_at_current("Loop body too large.");
        }

        self.emit_byte(((offset >> 8) & 0xff) as u8);
        self.emit_byte((offset & 0xff) as u8);
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let then_jump = self.emit_jump(OpCode::OpJumpFalse as u8);
        self.emit_byte(OpCode::OpPop as u8);
        self.statement();

        let else_jump = self.emit_jump(OpCode::OpJump as u8);

        self.patch_jump(then_jump);

        self.emit_byte(OpCode::OpPop as u8);

        if self.match_token(TokenType::Else) {
            self.statement();
        }

        self.patch_jump(else_jump);
    }

    fn emit_jump(&mut self, instruction: u8) -> u16 {
        self.emit_byte(instruction);
        self.emit_byte(0xFF);
        self.emit_byte(0xFF);

        self.current_chunk().code.len() as u16 - 2
    }

    fn patch_jump(&mut self, offset: u16) {
        let jump = self.current_chunk().code.len() as u16 - offset - 2;

        if jump == u16::MAX {
            self.error_at_current("Too much code to jump over.");
        }

        self.current_chunk().code[offset as usize] = ((jump >> 8) & 0xFF) as u8;
        self.current_chunk().code[offset as usize + 1] = (jump & 0xFF) as u8;
    }

    fn begin_scope(&mut self) {
        println!("Begin scope, depth {}", self.scope_depth);
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        println!("End scope, depth {}", self.scope_depth);
        self.scope_depth -= 1;

        while let Some(local) = self.locals.last() {
            if local.depth > self.scope_depth {
                self.emit_byte(OpCode::OpPop as u8);
                self.locals.pop();
            } else {
                break;
            }
        }
    }

    fn block(&mut self) {
        self.begin_scope();
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::EOF) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
        self.end_scope();
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

    fn and(&mut self, _: Option<bool>) {
        let jump_offset = self.emit_jump(OpCode::OpJumpFalse as u8);
        self.emit_byte(OpCode::OpPop as u8);

        self.parse_precedence(Precedence::And);

        self.patch_jump(jump_offset);
    }

    fn or(&mut self, _: Option<bool>) {
        let else_jump = self.emit_jump(OpCode::OpJumpFalse as u8);
        let end_jump = self.emit_jump(OpCode::OpJump as u8);

        self.patch_jump(else_jump);
        self.emit_byte(OpCode::OpPop as u8);

        self.parse_precedence(Precedence::Or);

        self.patch_jump(end_jump);
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
    }

    fn make_constant(&mut self, value: Val) -> u8 {
        let constant = self.current_chunk().add_constant(value);

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
        let line = self.parser.previous.clone().unwrap().line;
        self.current_chunk().write(byte, line);
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn end_compiler(&mut self) -> FunctionObject {
        self.emit_return();
        let mut function = self.function.clone();
        function.chunk = self.current_chunk().clone();
        if !self.parser.had_error {
            disassemble_chunk(&function.chunk, &function.name.as_str());
        }

        function
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        println!(
            "Parsing token: {:?}, precedence: {:?}",
            self.parser.previous.as_ref().unwrap(),
            precedence
        );
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
            TokenType::GreaterEqual => self.emit_byte(OpCode::OpGreaterEqual as u8),
            TokenType::Less => self.emit_byte(OpCode::OpLess as u8),
            TokenType::LessEqual => self.emit_byte(OpCode::OpLessEqual as u8),
            TokenType::Minus => self.emit_byte(OpCode::OpSubtract as u8),
            TokenType::Plus => self.emit_byte(OpCode::OpAdd as u8),
            TokenType::Star => self.emit_byte(OpCode::OpMultiply as u8),
            TokenType::Slash => self.emit_byte(OpCode::OpDivide as u8),
            _ => panic!("Invalid binary operator"),
        }
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::OpNil as u8);
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
