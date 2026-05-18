//! Recursive descent parser for the Linen programming language.
//!
//! Converts a stream of tokens into an AST. Supports error recovery
//! by skipping to the next statement boundary.
//!
//! Grammar:
//!   module     ::= import* decl*
//!   import     ::= "import" string "as" ident
//!   decl       ::= "let" ident [":" type] "=" expr
//!                | "fn" ident param+ [":" type] "->" expr
//!                | "type" ident [type-param*] "=" variant ("|" variant)*
//!                | "export" primary string expr
//!   expr       ::= or_expr
//!   or_expr    ::= and_expr ("||" and_expr)*
//!   and_expr   ::= eq_expr ("&&" eq_expr)*
//!   eq_expr    ::= cmp_expr (("==" | "!=") cmp_expr)*
//!   cmp_expr   ::= add_expr (("<" | ">" | "<=" | ">=") add_expr)*
//!   add_expr   ::= mul_expr (("+" | "-") mul_expr)*
//!   mul_expr   ::= pipeline (("*" | "/") pipeline)*
//!   pipeline   ::= apply ("|>" apply)*
//!   apply      ::= primary+
//!   primary    ::= number | string | bool | ident | "(" expr ")"
//!                | record | list | lambda | match | if
//!   lambda     ::= "\" param+ "->" expr
//!   match      ::= "match" expr "with" ("|" pattern "->" expr)+
//!   if         ::= "if" expr "then" expr "else" expr
//!   record     ::= "{" [field ("," field)*] "}"
//!   list       ::= "[" [expr ("," expr)*] "]"
//!   pattern    ::= ident | "_" | number | string | record_pat | variant_pat
//!   record_pat ::= "{" [ident ("," ident)*] "}"
//!   variant_pat ::= ident ["(" pattern* ")"]

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

    // ------------------------------------------------------------------
    // Entry points
    // ------------------------------------------------------------------

    /// Parse a complete module.
    pub fn parse_module(&mut self) -> Result<Module, ParseError> {
        let mut imports = Vec::new();
        let mut declarations = Vec::new();

        while !self.is_at_end() {
            if self.check(&Token::Import) {
                match self.parse_import() {
                    Ok(i) => imports.push(i),
                    Err(e) => {
                        self.report(e);
                        self.synchronize();
                    }
                }
            } else {
                match self.parse_declaration() {
                    Ok(d) => declarations.push(d),
                    Err(e) => {
                        self.report(e);
                        self.synchronize();
                    }
                }
            }
        }

        Ok(Module {
            imports,
            declarations,
        })
    }

    // ------------------------------------------------------------------
    // Declarations
    // ------------------------------------------------------------------

    fn parse_import(&mut self) -> Result<Import, ParseError> {
        let start = self.consume(&Token::Import, "import")?.span;
        let path = self.expect_string("import path")?;
        self.consume(&Token::As, "as")?;
        let alias = self.expect_ident("import alias")?;
        let end = self.prev_span();
        Ok(Import {
            path,
            alias,
            span: start.merge(end),
        })
    }

    fn parse_declaration(&mut self) -> Result<Declaration, ParseError> {
        if self.check(&Token::Let) {
            Ok(Declaration::Let(self.parse_let_decl()?))
        } else if self.check(&Token::Fn) {
            Ok(Declaration::Fn(self.parse_fn_decl()?))
        } else if self.check(&Token::Type) {
            Ok(Declaration::Type(self.parse_type_decl()?))
        } else if self.check(&Token::Export) {
            Ok(Declaration::Export(self.parse_export_decl()?))
        } else {
            Err(self.error("Expected declaration (let, fn, type, export)"))
        }
    }

    fn parse_let_decl(&mut self) -> Result<LetDecl, ParseError> {
        let start = self.consume(&Token::Let, "let")?.span;
        let name = self.expect_ident("variable name")?;
        let ty = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.consume(&Token::Equals, "=")?;
        let value = self.parse_expr()?;
        let end = self.prev_span();
        Ok(LetDecl {
            name,
            ty,
            value,
            span: start.merge(end),
        })
    }

    fn parse_fn_decl(&mut self) -> Result<FnDecl, ParseError> {
        let start = self.consume(&Token::Fn, "fn")?.span;
        let name = self.expect_ident("function name")?;
        let mut params = Vec::new();
        while self.check_ident() {
            params.push(Param {
                name: self.expect_ident("parameter name")?,
                ty: None,
            });
        }
        let ret_ty = if self.match_token(&Token::Colon) {
            // MVP: ret_ty is a single primary type (no arrow types here)
            Some(self.parse_type_primary()?)
        } else {
            None
        };
        self.consume(&Token::Arrow, "->")?;
        let body = self.parse_expr()?;
        let end = self.prev_span();
        Ok(FnDecl {
            name,
            params,
            ret_ty,
            body,
            span: start.merge(end),
        })
    }

    fn parse_type_decl(&mut self) -> Result<TypeDecl, ParseError> {
        let start = self.consume(&Token::Type, "type")?.span;
        let name = self.expect_ident("type name")?;
        let mut params = Vec::new();
        while self.check_ident() {
            params.push(self.expect_ident("type parameter")?);
        }
        self.consume(&Token::Equals, "=")?;
        let mut variants = vec![self.parse_variant()?];
        while self.match_token(&Token::Pipe) {
            variants.push(self.parse_variant()?);
        }
        let end = self.prev_span();
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
            if !self.check(&Token::RParen) {
                loop {
                    fields.push(self.parse_type()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            self.consume(&Token::RParen, ")")?;
        } else if self.match_token(&Token::LBrace) {
            // Record variant: Name { field: Type, ... }
            if !self.check(&Token::RBrace) {
                loop {
                    let field_name = self.expect_ident("field name")?;
                    self.consume(&Token::Colon, ":")?;
                    let field_ty = self.parse_type()?;
                    fields.push(Type::Named {
                        name: field_name,
                        args: vec![field_ty],
                    });
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            self.consume(&Token::RBrace, "}")?;
        }
        Ok(Variant { name, fields })
    }

    fn parse_export_decl(&mut self) -> Result<ExportDecl, ParseError> {
        let start = self.consume(&Token::Export, "export")?.span;
        let expr = self.parse_primary()?;
        let filename = self.expect_string("filename")?;
        let duration = self.parse_expr()?;
        let end = self.prev_span();
        Ok(ExportDecl {
            expr,
            filename,
            duration,
            span: start.merge(end),
        })
    }

    // ------------------------------------------------------------------
    // Types
    // ------------------------------------------------------------------

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        self.parse_type_arrow()
    }

    fn parse_type_arrow(&mut self) -> Result<Type, ParseError> {
        let mut left = self.parse_type_primary()?;
        while self.match_token(&Token::Arrow) {
            let right = self.parse_type_primary()?;
            left = Type::Arrow(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_type_primary(&mut self) -> Result<Type, ParseError> {
        if self.match_token(&Token::LParen) {
            let mut elems = Vec::new();
            if !self.check(&Token::RParen) {
                loop {
                    elems.push(self.parse_type()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            self.consume(&Token::RParen, ")")?;
            if elems.len() == 1 {
                Ok(elems.into_iter().next().unwrap())
            } else {
                Ok(Type::Tuple(elems))
            }
        } else if self.match_token(&Token::LBrace) {
            let mut fields = Vec::new();
            if !self.check(&Token::RBrace) {
                loop {
                    let name = self.expect_ident("field name")?;
                    self.consume(&Token::Colon, ":")?;
                    let ty = self.parse_type()?;
                    fields.push((name, ty));
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            self.consume(&Token::RBrace, "}")?;
            Ok(Type::Record(fields))
        } else {
            let name = self.expect_ident("type name")?;
            let mut args = Vec::new();
            // NOTE: generic syntax `<A, B>` is not tokenised yet; skip for MVP
            Ok(Type::Named { name, args })
        }
    }

    // ------------------------------------------------------------------
    // Expressions — precedence climbing
    // ------------------------------------------------------------------

    pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_pipeline()
    }

    fn parse_pipeline(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_or_expr()?;
        while self.match_token(&Token::Pipeline) {
            let right = self.parse_or_expr()?;
            let span = left.get_span().merge(right.get_span());
            left = Expr::Pipeline(Box::new(left), Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_or_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and_expr()?;
        while self.match_token(&Token::Or) {
            let right = self.parse_and_expr()?;
            let span = left.get_span().merge(right.get_span());
            left = Expr::Binary(BinOp::Or, Box::new(left), Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_eq_expr()?;
        while self.match_token(&Token::And) {
            let right = self.parse_eq_expr()?;
            let span = left.get_span().merge(right.get_span());
            left = Expr::Binary(BinOp::And, Box::new(left), Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_eq_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_cmp_expr()?;
        loop {
            if self.match_token(&Token::Eq) {
                let right = self.parse_cmp_expr()?;
                let span = left.get_span().merge(right.get_span());
                left = Expr::Binary(BinOp::Eq, Box::new(left), Box::new(right), span);
            } else if self.match_token(&Token::Neq) {
                let right = self.parse_cmp_expr()?;
                let span = left.get_span().merge(right.get_span());
                left = Expr::Binary(BinOp::Neq, Box::new(left), Box::new(right), span);
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_cmp_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_add_expr()?;
        loop {
            if self.match_token(&Token::Lt) {
                let right = self.parse_add_expr()?;
                let span = left.get_span().merge(right.get_span());
                left = Expr::Binary(BinOp::Lt, Box::new(left), Box::new(right), span);
            } else if self.match_token(&Token::Gt) {
                let right = self.parse_add_expr()?;
                let span = left.get_span().merge(right.get_span());
                left = Expr::Binary(BinOp::Gt, Box::new(left), Box::new(right), span);
            } else if self.match_token(&Token::Lte) {
                let right = self.parse_add_expr()?;
                let span = left.get_span().merge(right.get_span());
                left = Expr::Binary(BinOp::Lte, Box::new(left), Box::new(right), span);
            } else if self.match_token(&Token::Gte) {
                let right = self.parse_add_expr()?;
                let span = left.get_span().merge(right.get_span());
                left = Expr::Binary(BinOp::Gte, Box::new(left), Box::new(right), span);
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_add_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_mul_expr()?;
        loop {
            if self.match_token(&Token::Plus) {
                let right = self.parse_mul_expr()?;
                let span = left.get_span().merge(right.get_span());
                left = Expr::Binary(BinOp::Add, Box::new(left), Box::new(right), span);
            } else if self.match_token(&Token::Minus) {
                let right = self.parse_mul_expr()?;
                let span = left.get_span().merge(right.get_span());
                left = Expr::Binary(BinOp::Sub, Box::new(left), Box::new(right), span);
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_mul_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_apply()?;
        loop {
            if self.match_token(&Token::Star) {
                let right = self.parse_apply()?;
                let span = left.get_span().merge(right.get_span());
                left = Expr::Binary(BinOp::Mul, Box::new(left), Box::new(right), span);
            } else if self.match_token(&Token::Slash) {
                let right = self.parse_apply()?;
                let span = left.get_span().merge(right.get_span());
                left = Expr::Binary(BinOp::Div, Box::new(left), Box::new(right), span);
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_apply(&mut self) -> Result<Expr, ParseError> {
        let func = self.parse_primary()?;
        let mut args = Vec::new();
        while self.is_primary_start() {
            args.push(self.parse_primary()?);
        }
        if args.is_empty() {
            Ok(func)
        } else {
            let span = func.get_span().merge(args.last().unwrap().get_span());
            Ok(Expr::Apply(Box::new(func), args, span))
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        if let Some(lit) = self.match_literal() {
            let span = self.prev_span();
            Ok(Expr::Literal(lit, span))
        } else if self.check(&Token::Backslash) {
            self.parse_lambda()
        } else if self.check(&Token::Match) {
            self.parse_match_expr()
        } else if self.check(&Token::If) {
            self.parse_if_expr()
        } else if self.check(&Token::LParen) {
            self.parse_paren_or_tuple()
        } else if self.check(&Token::LBrace) {
            self.parse_record_expr()
        } else if self.check(&Token::LBracket) {
            self.parse_list_expr()
        } else if let Some(name) = self.match_ident() {
            let span = self.prev_span();
            Ok(Expr::Ident(name, span))
        } else {
            Err(self.error("Expected expression"))
        }
    }

    fn parse_lambda(&mut self) -> Result<Expr, ParseError> {
        let start = self.consume(&Token::Backslash, "\\")?.span;
        let mut params = Vec::new();
        while self.check_ident() {
            params.push(Param {
                name: self.expect_ident("parameter name")?,
                ty: None,
            });
        }
        self.consume(&Token::Arrow, "->")?;
        let body = self.parse_expr()?;
        let end = self.prev_span();
        Ok(Expr::Lambda(params, Box::new(body), start.merge(end)))
    }

    fn parse_match_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.consume(&Token::Match, "match")?.span;
        let scrutinee = self.parse_expr()?;
        self.consume(&Token::With, "with")?;
        let mut arms = Vec::new();
        loop {
            self.consume(&Token::Pipe, "|")?;
            let pattern = self.parse_pattern()?;
            self.consume(&Token::Arrow, "->")?;
            let expr = self.parse_expr()?;
            arms.push(Arm { pattern, expr });
            if !self.check(&Token::Pipe) {
                break;
            }
        }
        let end = self.prev_span();
        Ok(Expr::Match(Box::new(scrutinee), arms, start.merge(end)))
    }

    fn parse_if_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.consume(&Token::If, "if")?.span;
        let cond = self.parse_expr()?;
        self.consume(&Token::Then, "then")?;
        let then_branch = self.parse_expr()?;
        self.consume(&Token::Else, "else")?;
        let else_branch = self.parse_expr()?;
        let end = self.prev_span();
        Ok(Expr::If(
            Box::new(cond),
            Box::new(then_branch),
            Box::new(else_branch),
            start.merge(end),
        ))
    }

    fn parse_paren_or_tuple(&mut self) -> Result<Expr, ParseError> {
        let start = self.consume(&Token::LParen, "(")?.span;
        if self.match_token(&Token::RParen) {
            // Unit literal
            return Ok(Expr::Literal(Literal::Unit, start.merge(self.prev_span())));
        }
        let first = self.parse_expr()?;
        if self.match_token(&Token::RParen) {
            return Ok(first);
        }
        let mut elems = vec![first];
        while self.match_token(&Token::Comma) {
            elems.push(self.parse_expr()?);
        }
        let end = self.consume(&Token::RParen, ")")?.span;
        Ok(Expr::Tuple(elems, start.merge(end)))
    }

    fn parse_record_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.consume(&Token::LBrace, "{")?.span;
        let mut fields = Vec::new();
        if !self.check(&Token::RBrace) {
            loop {
                let name = self.expect_ident("field name")?;
                self.consume(&Token::Colon, ":")?;
                let value = self.parse_expr()?;
                fields.push((name, value));
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        let end = self.consume(&Token::RBrace, "}")?.span;
        Ok(Expr::Record(fields, start.merge(end)))
    }

    fn parse_list_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.consume(&Token::LBracket, "[")?.span;
        let mut elements = Vec::new();
        if !self.check(&Token::RBracket) {
            loop {
                elements.push(self.parse_expr()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        let end = self.consume(&Token::RBracket, "]")?.span;
        Ok(Expr::List(elements, start.merge(end)))
    }

    // ------------------------------------------------------------------
    // Patterns
    // ------------------------------------------------------------------

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        if self.match_ident_by_name("_") {
            Ok(Pattern::Wildcard(self.prev_span()))
        } else if let Some(lit) = self.match_literal() {
            Ok(Pattern::Literal(lit, self.prev_span()))
        } else if self.check(&Token::LBrace) {
            self.parse_record_pattern()
        } else {
            let name = self.expect_ident("pattern name")?;
            let start = self.prev_span();
            if self.match_token(&Token::LParen) {
                // Variant pattern: Name(pat, pat)
                let mut args = Vec::new();
                if !self.check(&Token::RParen) {
                    loop {
                        args.push(self.parse_pattern()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                let end = self.consume(&Token::RParen, ")")?.span;
                Ok(Pattern::Variant(name, args, start.merge(end)))
            } else {
                // Simple identifier pattern
                Ok(Pattern::Ident(name, start))
            }
        }
    }

    fn parse_record_pattern(&mut self) -> Result<Pattern, ParseError> {
        let start = self.consume(&Token::LBrace, "{")?.span;
        let mut fields = Vec::new();
        if !self.check(&Token::RBrace) {
            loop {
                let name = self.expect_ident("field name")?;
                // Allow both `{x}` (bind x) and `{x: pat}` (match pattern)
                let pat = if self.match_token(&Token::Colon) {
                    self.parse_pattern()?
                } else {
                    Pattern::Ident(name.clone(), self.prev_span())
                };
                fields.push((name, pat));
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        let end = self.consume(&Token::RBrace, "}")?.span;
        Ok(Pattern::Record(fields, start.merge(end)))
    }

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

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
                Token::Backslash
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

    fn consume(&mut self, expected: &Token, name: &str) -> Result<&SpannedToken, ParseError> {
        if self.match_token(expected) {
            Ok(&self.tokens[self.pos - 1])
        } else {
            let found = self
                .peek()
                .map(|t| format!("{:?}", t))
                .unwrap_or_else(|| "EOF".to_string());
            Err(ParseError::UnexpectedToken {
                expected: name.to_string(),
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

    fn report(&self, err: ParseError) {
        eprintln!("Parse error: {:?}", err);
    }

    fn synchronize(&mut self) {
        while !self.is_at_end() {
            if matches!(
                self.peek(),
                Some(Token::Let | Token::Fn | Token::Type | Token::Import | Token::Export)
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
