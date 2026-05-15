//! Recursive descent parser for the Linen programming language.
//!
//! Converts a stream of tokens into an AST. Supports error recovery
//! by skipping to the next statement boundary.

use linen_core::ast::*;
use linen_core::error::ParseError;
use linen_core::span::Span;
use linen_lexer::{SpannedToken, Token};

/// The parser state.
pub struct Parser<'a> {
    tokens: &'a [SpannedToken],
    pos: usize,
    source: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [SpannedToken], source: &'a str) -> Self {
        Self {
            tokens,
            pos: 0,
            source,
        }
    }

    /// Parse a complete module.
    pub fn parse_module(&mut self) -> Result<Module, ParseError> {
        let mut imports = Vec::new();
        let mut declarations = Vec::new();

        while !self.is_at_end() {
            if self.match_token(&Token::Import) {
                imports.push(self.parse_import()?);
            } else {
                match self.parse_declaration() {
                    Ok(decl) => declarations.push(decl),
                    Err(e) => {
                        self.synchronize();
                        return Err(e);
                    }
                }
            }
        }

        Ok(Module {
            imports,
            declarations,
        })
    }

    fn parse_import(&mut self) -> Result<Import, ParseError> {
        let start = self.prev_span();
        let path = self.expect_string("import path")?;
        self.expect_token(&Token::As)?;
        let alias = self.expect_ident("import alias")?;
        let end = self.prev_span();
        Ok(Import {
            path,
            alias,
            span: start.merge(end),
        })
    }

    fn parse_declaration(&mut self) -> Result<Declaration, ParseError> {
        if self.match_token(&Token::Let) {
            Ok(Declaration::Let(self.parse_let_decl()?))
        } else if self.match_token(&Token::Fn) {
            Ok(Declaration::Fn(self.parse_fn_decl()?))
        } else if self.match_token(&Token::Type) {
            Ok(Declaration::Type(self.parse_type_decl()?))
        } else if self.match_token(&Token::Export) {
            Ok(Declaration::Export(self.parse_export_decl()?))
        } else {
            Err(self.error("Expected declaration (let, fn, type, or export)"))
        }
    }

    fn parse_export_decl(&mut self) -> Result<ExportDecl, ParseError> {
        let start = self.prev_span();
        let expr = self.parse_primary()?;
        let filename = self.expect_string("filename")?;
        let duration = self.parse_expr()?;
        let end = self.current_span();
        Ok(ExportDecl {
            expr,
            filename,
            duration,
            span: start.merge(end),
        })
    }

    fn parse_let_decl(&mut self) -> Result<LetDecl, ParseError> {
        let start = self.prev_span();
        let name = self.expect_ident("variable name")?;
        let ty = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect_token(&Token::Equals)?;
        let value = self.parse_expr()?;
        let end = self.current_span();
        Ok(LetDecl {
            name,
            ty,
            value,
            span: start.merge(end),
        })
    }

    fn parse_fn_decl(&mut self) -> Result<FnDecl, ParseError> {
        let start = self.prev_span();
        let name = self.expect_ident("function name")?;
        let mut params = Vec::new();
        while self.check_ident() {
            let param_name = self.expect_ident("parameter name")?;
            let ty = if self.match_token(&Token::Colon) {
                Some(self.parse_type()?)
            } else {
                None
            };
            params.push(Param {
                name: param_name,
                ty,
            });
        }
        let ret_ty = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect_token(&Token::Arrow)?;
        let body = self.parse_expr()?;
        let end = self.current_span();
        Ok(FnDecl {
            name,
            params,
            ret_ty,
            body,
            span: start.merge(end),
        })
    }

    fn parse_type_decl(&mut self) -> Result<TypeDecl, ParseError> {
        let start = self.prev_span();
        let name = self.expect_ident("type name")?;
        let mut params = Vec::new();
        while self.check_ident() {
            params.push(self.expect_ident("type parameter")?);
        }
        self.expect_token(&Token::Equals)?;
        let mut variants = Vec::new();
        loop {
            variants.push(self.parse_variant()?);
            if !self.match_token(&Token::Pipe) {
                break;
            }
        }
        let end = self.current_span();
        Ok(TypeDecl {
            name,
            params,
            variants,
            span: start.merge(end),
        })
    }

    fn parse_variant(&mut self) -> Result<Variant, ParseError> {
        let name = self.expect_ident("variant name")?;
        let mut fields = Vec::new();
        if self.match_token(&Token::LParen) {
            loop {
                fields.push(self.parse_type()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            self.expect_token(&Token::RParen)?;
        }
        Ok(Variant { name, fields })
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        self.parse_arrow_type()
    }

    fn parse_arrow_type(&mut self) -> Result<Type, ParseError> {
        let mut left = self.parse_primary_type()?;
        while self.match_token(&Token::Arrow) {
            let right = self.parse_primary_type()?;
            left = Type::Arrow(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_primary_type(&mut self) -> Result<Type, ParseError> {
        if self.match_token(&Token::LParen) {
            let mut elems = Vec::new();
            loop {
                elems.push(self.parse_type()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            self.expect_token(&Token::RParen)?;
            if elems.len() == 1 {
                Ok(elems.into_iter().next().unwrap())
            } else {
                Ok(Type::Tuple(elems))
            }
        } else if self.match_token(&Token::LBrace) {
            let mut fields = Vec::new();
            loop {
                let name = self.expect_ident("field name")?;
                self.expect_token(&Token::Colon)?;
                let ty = self.parse_type()?;
                fields.push((name, ty));
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            self.expect_token(&Token::RBrace)?;
            Ok(Type::Record(fields))
        } else {
            let name = self.expect_ident("type name")?;
            let mut args = Vec::new();
            if self.match_token(&Token::Lt) {
                loop {
                    args.push(self.parse_type()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                // Expect '>' but our lexer doesn't have a separate '>' token
                // For now we skip this detail
            }
            Ok(Type::Named { name, args })
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_pipeline()
    }

    fn parse_pipeline(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_application()?;
        while self.match_token(&Token::Pipeline) {
            let right = self.parse_application()?;
            let span = left.get_span().merge(right.get_span());
            left = Expr::Pipeline(Box::new(left), Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_application(&mut self) -> Result<Expr, ParseError> {
        let mut func = self.parse_primary()?;
        while self.is_primary_start() {
            let arg = self.parse_primary()?;
            let span = func.get_span().merge(arg.get_span());
            func = Expr::Apply(Box::new(func), vec![arg], span);
        }
        Ok(func)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&Token::Let) {
            self.parse_let_expr()
        } else if self.match_token(&Token::Match) {
            self.parse_match_expr()
        } else if self.match_token(&Token::If) {
            self.parse_if_expr()
        } else if self.match_token(&Token::LParen) {
            let expr = self.parse_expr()?;
            self.expect_token(&Token::RParen)?;
            Ok(expr)
        } else if self.match_token(&Token::LBracket) {
            self.parse_list()
        } else if self.match_token(&Token::LBrace) {
            self.parse_record()
        } else if let Some(lit) = self.match_literal() {
            let span = self.prev_span();
            Ok(Expr::Literal(lit, span))
        } else if let Some(name) = self.match_ident() {
            let span = self.prev_span();
            Ok(Expr::Ident(name, span))
        } else {
            Err(self.error("Expected expression"))
        }
    }

    fn parse_let_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.prev_span();
        let name = self.expect_ident("variable name")?;
        let ty = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect_token(&Token::Equals)?;
        let value = self.parse_expr()?;
        // In expression position, `let` requires `in` or is part of block syntax
        // For simplicity, we assume let-in or treat as declaration
        let body = Expr::Literal(Literal::Unit, start); // placeholder
        let end = self.current_span();
        Ok(Expr::Let(
            name,
            ty,
            Box::new(value),
            Box::new(body),
            start.merge(end),
        ))
    }

    fn parse_match_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.prev_span();
        let scrutinee = self.parse_expr()?;
        self.expect_token(&Token::With)?;
        let mut arms = Vec::new();
        while self.match_token(&Token::Pipe) {
            let pattern = self.parse_pattern()?;
            self.expect_token(&Token::Arrow)?;
            let expr = self.parse_expr()?;
            arms.push(Arm { pattern, expr });
        }
        let end = self.current_span();
        Ok(Expr::Match(Box::new(scrutinee), arms, start.merge(end)))
    }

    fn parse_if_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.prev_span();
        let cond = self.parse_expr()?;
        self.expect_token(&Token::Then)?;
        let then_branch = self.parse_expr()?;
        self.expect_token(&Token::Else)?;
        let else_branch = self.parse_expr()?;
        let end = self.current_span();
        Ok(Expr::If(
            Box::new(cond),
            Box::new(then_branch),
            Box::new(else_branch),
            start.merge(end),
        ))
    }

    fn parse_list(&mut self) -> Result<Expr, ParseError> {
        let start = self.prev_span();
        let mut elements = Vec::new();
        while !self.check(&Token::RBracket) && !self.is_at_end() {
            elements.push(self.parse_expr()?);
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        self.expect_token(&Token::RBracket)?;
        let end = self.prev_span();
        Ok(Expr::List(elements, start.merge(end)))
    }

    fn parse_record(&mut self) -> Result<Expr, ParseError> {
        let start = self.prev_span();
        let mut fields = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let name = self.expect_ident("field name")?;
            self.expect_token(&Token::Colon)?;
            let value = self.parse_expr()?;
            fields.push((name, value));
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        self.expect_token(&Token::RBrace)?;
        let end = self.prev_span();
        Ok(Expr::Record(fields, start.merge(end)))
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        if self.match_ident_by_name("_") {
            let span = self.prev_span();
            Ok(Pattern::Wildcard(span))
        } else if let Some(lit) = self.match_literal() {
            let span = self.prev_span();
            Ok(Pattern::Literal(lit, span))
        } else {
            let name = self.expect_ident("pattern name")?;
            let span = self.prev_span();
            Ok(Pattern::Ident(name, span))
        }
    }

    // Helper methods

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|t| &t.token)
    }

    fn advance(&mut self) -> &SpannedToken {
        if !self.is_at_end() {
            self.pos += 1;
        }
        &self.tokens[self.pos - 1]
    }

    fn match_token(&mut self, expected: &Token) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, expected: &Token) -> bool {
        self.peek().map(|t| t == expected).unwrap_or(false)
    }

    fn check_ident(&self) -> bool {
        matches!(self.peek(), Some(Token::Ident(_)))
    }

    fn is_primary_start(&self) -> bool {
        matches!(
            self.peek(),
            Some(
                Token::Let
                    | Token::Match
                    | Token::If
                    | Token::LParen
                    | Token::LBracket
                    | Token::LBrace
                    | Token::Ident(_)
                    | Token::Int(_)
                    | Token::Float(_)
                    | Token::String(_)
                    | Token::True
                    | Token::False
            )
        )
    }

    fn match_ident(&mut self) -> Option<String> {
        if let Some(Token::Ident(name)) = self.peek().cloned() {
            self.advance();
            Some(name)
        } else {
            None
        }
    }

    fn match_ident_by_name(&mut self, expected: &str) -> bool {
        if let Some(Token::Ident(name)) = self.peek() {
            if name == expected {
                self.advance();
                return true;
            }
        }
        false
    }

    fn match_literal(&mut self) -> Option<Literal> {
        match self.peek().cloned() {
            Some(Token::Int(n)) => {
                self.advance();
                Some(Literal::Int(n))
            }
            Some(Token::Float(x)) => {
                self.advance();
                Some(Literal::Float(x))
            }
            Some(Token::String(s)) => {
                self.advance();
                Some(Literal::String(s))
            }
            Some(Token::True) => {
                self.advance();
                Some(Literal::Bool(true))
            }
            Some(Token::False) => {
                self.advance();
                Some(Literal::Bool(false))
            }
            _ => None,
        }
    }

    fn expect_ident(&mut self, what: &str) -> Result<String, ParseError> {
        self.match_ident()
            .ok_or_else(|| self.error(&format!("Expected {}", what)))
    }

    fn expect_string(&mut self, what: &str) -> Result<String, ParseError> {
        match self.peek().cloned() {
            Some(Token::String(s)) => {
                self.advance();
                Ok(s)
            }
            _ => Err(self.error(&format!("Expected string {}", what))),
        }
    }

    fn expect_token(&mut self, expected: &Token) -> Result<(), ParseError> {
        if self.match_token(expected) {
            Ok(())
        } else {
            let found = self
                .peek()
                .map(|t| format!("{:?}", t))
                .unwrap_or_else(|| "EOF".to_string());
            Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", expected),
                found,
                span: self.current_span(),
            })
        }
    }

    fn prev_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span::default()
        }
    }

    fn current_span(&self) -> Span {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].span
        } else if !self.tokens.is_empty() {
            self.tokens[self.tokens.len() - 1].span
        } else {
            Span::default()
        }
    }

    fn error(&self, msg: &str) -> ParseError {
        ParseError::UnexpectedToken {
            expected: msg.to_string(),
            found: self
                .peek()
                .map(|t| format!("{:?}", t))
                .unwrap_or_else(|| "EOF".to_string()),
            span: self.current_span(),
        }
    }

    fn synchronize(&mut self) {
        while !self.is_at_end() {
            if matches!(
                self.peek(),
                Some(Token::Let | Token::Fn | Token::Type | Token::Import)
            ) {
                return;
            }
            self.advance();
        }
    }
}

/// Convenience: parse source string into AST.
pub fn parse(input: &str) -> Result<Module, ParseError> {
    let tokens = linen_lexer::lex(input).map_err(|e| ParseError::UnexpectedToken {
        expected: "valid token".to_string(),
        found: format!("{:?}", e),
        span: Span::default(),
    })?;
    let mut parser = Parser::new(&tokens, input);
    parser.parse_module()
}
