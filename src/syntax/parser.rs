use crate::{
    Diagnostic, DiagnosticLabel, ErrorCode, Span,
    syntax::{
        BinaryOp, DurationLiteral, Expr, ExprKind, Identifier, LetDecl, RenderDecl, TokenKind,
        TypeAnnotation, TypeName, UnaryOp,
    },
};

use super::{Program, Token, lex};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParserConfig {
    pub max_depth: usize,
    pub max_nodes: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_depth: 128,
            max_nodes: 10_000,
        }
    }
}

pub fn parse(source: &str) -> Result<Program, Diagnostic> {
    parse_with_config(source, ParserConfig::default())
}

pub fn parse_with_config(source: &str, config: ParserConfig) -> Result<Program, Diagnostic> {
    let tokens = lex(source)?;
    parse_tokens_with_config(&tokens, config)
}

pub fn parse_tokens(tokens: &[Token]) -> Result<Program, Diagnostic> {
    parse_tokens_with_config(tokens, ParserConfig::default())
}

pub fn parse_tokens_with_config(
    tokens: &[Token],
    config: ParserConfig,
) -> Result<Program, Diagnostic> {
    Parser::new(tokens, config).parse_program()
}

pub struct Parser<'tokens> {
    tokens: &'tokens [Token],
    current: usize,
    expression_depth: usize,
    nodes: usize,
    config: ParserConfig,
}

impl<'tokens> Parser<'tokens> {
    pub const fn new(tokens: &'tokens [Token], config: ParserConfig) -> Self {
        Self {
            tokens,
            current: 0,
            expression_depth: 0,
            nodes: 0,
            config,
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, Diagnostic> {
        let mut bindings = Vec::new();

        while matches!(self.peek().map(|token| &token.kind), Some(TokenKind::Let)) {
            bindings.push(self.parse_let_decl()?);
        }

        if self.is_at_end() {
            return Err(Diagnostic::error(
                ErrorCode::MissingOrMisplacedRender,
                "missing render declaration",
                DiagnosticLabel::new(
                    self.eof_span(),
                    "expected a render declaration at the end of the program",
                ),
            ));
        }

        if !matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::Render)
        ) {
            return Err(self.unexpected_token(self.peek(), Some("a let or render declaration")));
        }

        let render_keyword_span = self.span_here();
        let render = self.parse_render_decl()?;

        // Render должен быть последней declaration.
        if let Some(token) = self.peek()
            && token.kind != TokenKind::Eof
        {
            return Err(Diagnostic::error(
                ErrorCode::MissingOrMisplacedRender,
                "render declaration must be the last declaration",
                DiagnosticLabel::new(
                    render_keyword_span,
                    format!("render must appear after this declaration: {}", token.kind),
                ),
            )
            .with_secondary(DiagnosticLabel::new(
                token.span,
                "this render declaration is followed by another declaration",
            )));
        }

        let mut first_out: Option<Span> = None;

        for binding in &bindings {
            if binding.name.name != "out" {
                continue;
            }

            if first_out.is_some()  {
                return Err(Diagnostic::error(
                    ErrorCode::MultipleOutputs,
                    "multiple out bindings",
                    DiagnosticLabel::new(binding.span, &binding.name.name),
                )
                .with_secondary(DiagnosticLabel::new(
                    binding.span,
                    "the second out binding is here",
                )));
            }

            first_out = Some(binding.span);
        }

        if first_out.is_none() {
            return Err(Diagnostic::error(
                ErrorCode::MissingOutput,
                "missing out binding",
                DiagnosticLabel::new(
                    render_keyword_span,
                    "the program must contain exactly one out binding",
                ),
            ));
        }

        let start_span = bindings
            .first()
            .map_or(render_keyword_span, |binding| binding.span);

        let span = self.span_between(start_span, render.span)?;

        Ok(Program {
            bindings,
            render,
            span,
        })
    }

    fn peek(&self) -> Option<&'tokens Token> {
        self.tokens.get(self.current)
    }

    fn previous(&self) -> Option<&'tokens Token> {
        self.tokens.get(self.current - 1)
    }

    fn advance(&mut self) -> Option<&'tokens Token> {
        let token = self.tokens.get(self.current)?;
        self.current += 1;
        Some(token)
    }

    fn is_at_end(&self) -> bool {
        matches!(
            self.peek().map(|token| &token.kind),
            None | Some(TokenKind::Eof)
        )
    }

    fn eof_span(&self) -> Span {
        self.tokens
            .last()
            .map_or(Span::empty(0), |token| Span::empty(token.span.end()))
    }

    fn expect(&mut self, kind: TokenKind) -> Result<&'tokens Token, Diagnostic> {
        let token = self
            .peek()
            .ok_or_else(|| self.unexpected_token(None, Some("unexpected token")))?;

        if token.kind == kind {
            self.advance();
            return Ok(token);
        }

        let expected = format!("{kind:?}");
        Err(self.unexpected_token(self.peek(), Some(&expected)))
    }

    fn span_here(&self) -> Span {
        self.peek()
            .map_or_else(|| self.eof_span(), |token| token.span)
    }

    fn unexpected_token(
        &self,
        token: Option<&'tokens Token>,
        expected: Option<&str>,
    ) -> Diagnostic {
        let (span, found) = match token {
            Some(token) => (token.span, format!("{:?}", token.kind)),
            None => (self.eof_span(), "end of input".to_string()),
        };

        let message = match expected {
            Some(expected) => format!("unexpected token {found}, expected {expected}"),
            None => format!("unexpected token {found}"),
        };

        Diagnostic::error(
            ErrorCode::UnexpectedToken,
            message,
            DiagnosticLabel::new(span, "unexpected token here"),
        )
    }

    fn missing_semicolon(&self, token: Option<&'tokens Token>) -> Diagnostic {
        let span = token.map_or_else(|| self.eof_span(), |token| Span::empty(token.span.start()));
        let token_kind = &token.expect("expected valid token").kind;
        Diagnostic::error(
            ErrorCode::MissingSemicolon,
            "missing semicolon",
            DiagnosticLabel::new(span, token_kind.to_string()),
        )
        .with_secondary(DiagnosticLabel::new(span, "expected a semicolon here"))
    }

    fn span_between(&self, start: Span, end: Span) -> Result<Span, Diagnostic> {
        Span::new(start.start(), end.end()).map_err(|_| {
            Diagnostic::error(
                ErrorCode::UnexpectedToken,
                "token spans are not ordered",
                DiagnosticLabel::new(end, "this token ends before the expression starts"),
            )
        })
    }

    fn parse_nested_expression(&mut self, opening_span: Span) -> Result<Expr, Diagnostic> {
        if self.expression_depth >= self.config.max_depth {
            return Err(Diagnostic::error(
                ErrorCode::ExpressionDepthLimit,
                "expression nested limit exceeded",
                DiagnosticLabel::new(
                    opening_span,
                    "this expression exceeds the configured nesting limit",
                ),
            ));
        }

        self.expression_depth += 1;
        let result = self.parse_expr();
        self.expression_depth -= 1;

        result
    }

    fn parse_expr(&mut self) -> Result<Expr, Diagnostic> {
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_multiplicative()?;

        loop {
            let operator = match self.peek().map(|token| &token.kind) {
                Some(TokenKind::Plus) => BinaryOp::Add,
                Some(TokenKind::Minus) => BinaryOp::Subtract,
                _ => break,
            };

            self.advance();

            let right = self.parse_multiplicative()?;
            let span = self.span_between(expr.span, right.span)?;

            expr = self.make_expr(
                ExprKind::Binary {
                    operator,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            )?;
        }

        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_unary()?;

        loop {
            let operator = match self.peek().map(|token| &token.kind) {
                Some(TokenKind::Star) => BinaryOp::Multiply,
                Some(TokenKind::Slash) => BinaryOp::Divide,
                _ => break,
            };

            self.advance();

            let right = self.parse_unary()?;
            let span = self.span_between(expr.span, right.span)?;

            expr = self.make_expr(
                ExprKind::Binary {
                    operator,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            )?;
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, Diagnostic> {
        if !matches!(self.peek().map(|token| &token.kind), Some(TokenKind::Minus)) {
            return self.parse_postfix();
        }

        let minus = self.advance().expect("minus was checked");
        let operand = self.parse_unary_operand(minus.span)?;
        let span = self.span_between(minus.span, operand.span)?;

        self.make_expr(
            ExprKind::Unary {
                operator: UnaryOp::Negate,
                operand: Box::new(operand),
            },
            span,
        )
    }

    fn parse_unary_operand(&mut self, minus_span: Span) -> Result<Expr, Diagnostic> {
        if self.expression_depth >= self.config.max_depth {
            return Err(Diagnostic::error(
                ErrorCode::ExpressionDepthLimit,
                "expression nesting limit exceeded",
                DiagnosticLabel::new(
                    minus_span,
                    "this unary expression exceeds the configured nesting limit",
                ),
            ));
        }

        self.expression_depth += 1;
        let result = self.parse_postfix();
        self.expression_depth -= 1;

        result
    }

    fn parse_render_decl(&mut self) -> Result<RenderDecl, Diagnostic> {
        self.count_declaration()?;

        let render_token = self.expect(TokenKind::Render)?;
        let mut negative = false;

        if matches!(self.peek().map(|token| &token.kind), Some(TokenKind::Minus)) {
            self.advance();

            negative = true;
        }

        let duration = self.expect_duration()?;
        let semicolon = self.expect_semicolon()?;
        let span = self.span_between(render_token.span, semicolon.span)?;

        Ok(RenderDecl {
            negative,
            duration,
            span,
        })
    }

    fn expect_duration(&mut self) -> Result<DurationLiteral, Diagnostic> {
        let token = self
            .peek()
            .ok_or_else(|| self.unexpected_token(None, Some("a duration")))?;

        let duration = match &token.kind {
            TokenKind::Duration { value, unit } => DurationLiteral {
                value: *value,
                unit: *unit,
                span: token.span,
            },
            _ => {
                return Err(self.unexpected_token(Some(token), Some("a duration")));
            }
        };

        self.advance();
        Ok(duration)
    }

    fn parse_let_decl(&mut self) -> Result<LetDecl, Diagnostic> {
        self.count_declaration()?;

        let token = self.expect(TokenKind::Let)?;
        let name = self.expect_identifier()?;
        let annotation = self.parse_annotation()?;

        self.expect(TokenKind::Equal)?;

        let value = self.parse_expr()?;
        let semicolon = self.expect_semicolon()?;
        let span = self.span_between(token.span, semicolon.span)?;

        Ok(LetDecl {
            name,
            annotation,
            value,
            span,
        })
    }

    fn parse_annotation(&mut self) -> Result<Option<TypeAnnotation>, Diagnostic> {
        if !matches!(self.peek().map(|token| &token.kind), Some(TokenKind::Colon)) {
            return Ok(None);
        }

        self.advance();

        let token = self
            .peek()
            .ok_or_else(|| self.unexpected_token(None, Some("Float or Sig")))?;

        let (kind, span) = match &token.kind {
            TokenKind::Float => (TypeName::Float, token.span),
            TokenKind::Sig => (TypeName::Sig, token.span),
            _ => {
                return Err(self.unexpected_token(Some(token), Some("Float or Sig")));
            }
        };

        self.advance();

        Ok(Some(TypeAnnotation { kind, span }))
    }

    fn expect_semicolon(&mut self) -> Result<&'tokens Token, Diagnostic> {
        if matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::Semicolon)
        ) {
            return Ok(self.advance().expect("semicolon was checked"));
        }

        Err(self.missing_semicolon(self.previous()))
    }

    fn expect_identifier(&mut self) -> Result<Identifier, Diagnostic> {
        if let Some(token) = self.peek() {
            match &token.kind {
                TokenKind::Identifier(identifier) => {
                    let name = identifier.clone();
                    let span = token.span;
                    let identifier = Identifier::new(name, span);

                    self.advance();

                    return Ok(identifier);
                }
                _ => {
                    return Err(self.unexpected_token(Some(token), Some("an identifier")));
                }
            }
        }

        Err(self.unexpected_token(self.peek(), Some("Identifier")))
    }

    fn parse_primary(&mut self) -> Result<Expr, Diagnostic> {
        let token = self
            .peek()
            .ok_or_else(|| self.unexpected_token(None, Some("an expression")))?;

        match &token.kind {
            TokenKind::Number(value) => {
                let value = *value;
                let span = token.span;

                self.advance();
                self.make_expr(ExprKind::Number(value), span)
            }
            TokenKind::Identifier(value) => {
                let identifier = Identifier::new(value.clone(), token.span);
                let span = token.span;

                self.advance();
                self.make_expr(ExprKind::Identifier(identifier), span)
            }
            TokenKind::LeftParen => self.parse_group(),
            _ => Err(self.unexpected_token(Some(token), Some("an expression"))),
        }
    }

    fn parse_group(&mut self) -> Result<Expr, Diagnostic> {
        let opening = self.expect(TokenKind::LeftParen)?;
        let inner = self.parse_nested_expression(opening.span)?;
        let closing = self.expect(TokenKind::RightParen)?;
        let span = self.span_between(opening.span, closing.span)?;

        self.make_expr(ExprKind::Group(Box::new(inner)), span)
    }

    fn parse_postfix(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_primary()?;

        if !matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::LeftParen)
        ) {
            return Ok(expr);
        }

        let opening = self
            .advance()
            .expect("LeftParen was checked before advancing");
        let callee = match &expr.kind {
            ExprKind::Identifier(identifier) => identifier.clone(),
            _ => {
                return Err(
                    self.unexpected_token(Some(opening), Some("a function name before '('"))
                );
            }
        };
        let mut arguments = Vec::new();

        if !matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::RightParen)
        ) {
            loop {
                arguments.push(self.parse_nested_expression(opening.span)?);

                if matches!(self.peek().map(|token| &token.kind), Some(TokenKind::Comma)) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        let closing = self.expect(TokenKind::RightParen)?;
        let span = self.span_between(callee.span, closing.span)?;

        expr.kind = ExprKind::Call { callee, arguments };
        expr.span = span;

        if matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::LeftParen)
        ) {
            return Err(self.unexpected_token(self.peek(), Some("only one call suffix")));
        }

        Ok(expr)
    }

    fn make_expr(&mut self, kind: ExprKind, span: Span) -> Result<Expr, Diagnostic> {
        if self.nodes >= self.config.max_nodes {
            return Err(Diagnostic::error(
                ErrorCode::AstNodeLimit,
                "too many nodes in the AST",
                DiagnosticLabel::new(span, "this expression exceeds the maximum number of nodes"),
            ));
        }

        self.nodes += 1;
        Ok(Expr::new(kind, span))
    }

    fn count_declaration(&mut self) -> Result<(), Diagnostic> {
        if self.nodes >= self.config.max_nodes {
            return Err(Diagnostic::error(
                ErrorCode::AstNodeLimit,
                "too many nodes in the AST",
                DiagnosticLabel::new(
                    self.span_here(),
                    "this declaration exceeds the maximum number of nodes",
                ),
            ));
        }

        self.nodes += 1;
        Ok(())
    }
}
