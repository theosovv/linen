use ast_node::AstNode;

use crate::{
    error::LinenError,
    lexer::token::{Token, TokenType},
};
pub mod ast_node;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    statements: Vec<AstNode>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            statements: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<AstNode>, LinenError> {
        while !self.is_at_end() {
            let statement = self.declaration()?;
            self.statements.push(statement);
        }

        Ok(self.statements.clone())
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn declaration(&mut self) -> Result<AstNode, LinenError> {
        if self.match_token(TokenType::LeftParen) {
            return self.parse_parenthesized_expression();
        }

        if self.match_token(TokenType::LeftBrace) {
            return self.parse_list();
        }

        if self.match_token(TokenType::Identifier) {
            return self.parse_identifier();
        }

        if self.match_token(TokenType::String) {
            return self.parse_string();
        }

        if self.match_token(TokenType::Number) {
            return self.parse_number();
        }

        if self.match_token(TokenType::Boolean) {
            return self.parse_boolean();
        }

        if self.match_token(TokenType::Nil) {
            return self.parse_nil();
        }

        if self.match_token(TokenType::Symbol) {
            return self.parse_symbol();
        }

        if self.match_token(TokenType::Eof) {
            self.advance();
            return Ok(AstNode::Nil);
        }

        Err(LinenError::new(
            "Unexpected token".to_string(),
            self.tokens[self.current].line,
            self.tokens[self.current].source.clone(),
        ))
    }

    fn parse_list(&mut self) -> Result<AstNode, LinenError> {
        let mut list = Vec::new();
        self.consume(TokenType::LeftBrace, "Expect '{' after '('.")?;
        while !self.check_token(TokenType::RightBrace) && !self.is_at_end() {
            list.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after expression.")?;
        Ok(AstNode::List(list))
    }

    fn parse_parenthesized_expression(&mut self) -> Result<AstNode, LinenError> {
        let mut expressions = Vec::new();
        self.consume(TokenType::LeftParen, "Expect '(' after '('.")?;
        while !self.check_token(TokenType::RightParen) && !self.is_at_end() {
            expressions.push(self.declaration()?);
        }

        self.consume(TokenType::RightParen, "Expect ')' after expression.")?;

        if expressions.len() == 1 {
            Ok(expressions.pop().unwrap())
        } else {
            Ok(AstNode::Program(expressions))
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, LinenError> {
        if self.check_token(token_type) {
            return Ok(self.advance());
        }
        Err(LinenError::new(
            message.to_string(),
            self.tokens[self.current].line,
            self.tokens[self.current].source.clone(),
        ))
    }

    fn parse_identifier(&mut self) -> Result<AstNode, LinenError> {
        let token = self.advance();

        Ok(AstNode::Identifier(token.lexeme))
    }

    fn parse_string(&mut self) -> Result<AstNode, LinenError> {
        let token = self.advance();

        Ok(AstNode::String(token.lexeme))
    }

    fn parse_number(&mut self) -> Result<AstNode, LinenError> {
        let token = self.advance();

        Ok(AstNode::Number(token.lexeme.parse().unwrap()))
    }

    fn parse_boolean(&mut self) -> Result<AstNode, LinenError> {
        let token = self.advance();

        Ok(AstNode::Boolean(token.lexeme == "true"))
    }

    fn parse_nil(&mut self) -> Result<AstNode, LinenError> {
        let _ = self.advance();

        Ok(AstNode::Nil)
    }

    fn parse_symbol(&mut self) -> Result<AstNode, LinenError> {
        let token = self.advance();

        match token.lexeme.as_str() {
            "let" => self.parse_let_statement(),
            "fn" => self.parse_function_definition(),
            "if" => self.parse_if_expression(),
            "cond" => self.parse_cond_expression(),
            "lambda" => self.parse_lambda_expression(),
            "else" => Ok(AstNode::Else),
            "loop" => self.parse_loop_expression(),
            "recur" => self.parse_recur_expression(),
            _ => Ok(AstNode::Symbol(token.lexeme)),
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn parse_let_statement(&mut self) -> Result<AstNode, LinenError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'let'.")?;

        let mut bindings = Vec::new();

        // Parse each binding
        while !self.check_token(TokenType::RightParen) && !self.is_at_end() {
            self.consume(TokenType::LeftParen, "Expect '(' for binding.")?;

            // Parse the identifier
            let identifier = if self.match_token(TokenType::Identifier) {
                self.advance().lexeme
            } else {
                return Err(LinenError::new(
                    "Expect identifier in binding.".to_string(),
                    self.tokens[self.current].line,
                    self.tokens[self.current].source.clone(),
                ));
            };

            // Parse the expression associated with the identifier
            let expression = self.declaration()?;

            // Consume the closing parenthesis for the binding
            self.consume(
                TokenType::RightParen,
                "Expect ')' after binding expression.",
            )?;

            bindings.push((identifier, expression));
        }

        // Consume the closing parenthesis for the bindings list
        self.consume(TokenType::RightParen, "Expect ')' after bindings.")?;

        // Parse the body of the let expression
        let body = self.declaration()?;

        // Return the let expression as an AST node
        Ok(AstNode::Let {
            bindings,
            body: Box::new(body),
        })
    }

    fn parse_function_definition(&mut self) -> Result<AstNode, LinenError> {
        let name = if self.match_token(TokenType::Identifier) {
            self.advance().lexeme
        } else {
            return Err(LinenError::new(
                "Expect function name.".to_string(),
                self.tokens[self.current].line,
                self.tokens[self.current].source.clone(),
            ));
        };

        let mut params = Vec::new();

        self.consume(TokenType::LeftParen, "Expect '(' after function name.")?;

        // Parse function parameters
        while !self.check_token(TokenType::RightParen) && !self.is_at_end() {
            if self.match_token(TokenType::Identifier) {
                params.push(self.advance().lexeme);
            } else {
                return Err(LinenError::new(
                    "Expect parameter name.".to_string(),
                    self.tokens[self.current].line,
                    self.tokens[self.current].source.clone(),
                ));
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        // Parse function body
        let body = self.declaration()?;

        Ok(AstNode::Function {
            name,
            params,
            body: Box::new(body),
        })
    }

    fn parse_if_expression(&mut self) -> Result<AstNode, LinenError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;

        let condition = self.declaration()?;

        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.declaration()?;

        let else_branch = if self.match_token(TokenType::Symbol) && self.peek().lexeme == "else" {
            self.advance();
            Some(Box::new(self.declaration()?))
        } else {
            None
        };

        Ok(AstNode::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn parse_cond_expression(&mut self) -> Result<AstNode, LinenError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'cond'.")?;

        let mut clauses = Vec::new();

        while !self.check_token(TokenType::RightParen) && !self.is_at_end() {
            self.consume(TokenType::LeftParen, "Expect '(' for cond clause.")?;

            let condition = self.declaration()?;

            let mut expressions = Vec::new();
            while !self.check_token(TokenType::RightParen) && !self.is_at_end() {
                expressions.push(self.declaration()?);
            }

            self.consume(TokenType::RightParen, "Expect ')' after cond clause.")?;

            clauses.push((condition, expressions));
        }

        self.consume(TokenType::RightParen, "Expect ')' after cond clauses.")?;

        Ok(AstNode::Cond(clauses))
    }

    fn parse_lambda_expression(&mut self) -> Result<AstNode, LinenError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'lambda'.")?;

        let mut params = Vec::new();

        while !self.check_token(TokenType::RightParen) && !self.is_at_end() {
            if self.match_token(TokenType::Identifier) {
                params.push(self.advance().lexeme);
            } else {
                return Err(LinenError::new(
                    "Expect parameter name.".to_string(),
                    self.tokens[self.current].line,
                    self.tokens[self.current].source.clone(),
                ));
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after lambda parameters.")?;

        let body = self.declaration()?;

        Ok(AstNode::Lambda {
            params,
            body: Box::new(body),
        })
    }

    fn parse_loop_expression(&mut self) -> Result<AstNode, LinenError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'loop'.")?;

        let mut bindings = Vec::new();

        while !self.check_token(TokenType::RightParen) && !self.is_at_end() {
            self.consume(TokenType::LeftParen, "Expect '(' for loop binding.")?;

            let identifier = if self.match_token(TokenType::Identifier) {
                self.advance().lexeme
            } else {
                return Err(LinenError::new(
                    "Expect identifier in loop binding.".to_string(),
                    self.tokens[self.current].line,
                    self.tokens[self.current].source.clone(),
                ));
            };

            let expression = self.declaration()?;

            self.consume(TokenType::RightParen, "Expect ')' after loop binding.")?;

            bindings.push((identifier, expression));
        }

        self.consume(TokenType::RightParen, "Expect ')' after loop bindings.")?;

        let body = self.declaration()?;

        Ok(AstNode::Loop {
            bindings,
            body: Box::new(body),
        })
    }

    fn parse_recur_expression(&mut self) -> Result<AstNode, LinenError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'recur'.")?;

        let mut expressions = Vec::new();

        while !self.check_token(TokenType::RightParen) && !self.is_at_end() {
            expressions.push(self.declaration()?);
        }

        self.consume(TokenType::RightParen, "Expect ')' after recur expressions.")?;

        Ok(AstNode::Recur(expressions))
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        self.check_token(token_type)
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens[self.current - 1].clone()
    }

    fn check_token(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.tokens[self.current].token_type == token_type
    }
}
