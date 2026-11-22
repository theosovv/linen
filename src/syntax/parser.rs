use crate::{
    Diagnostic, DiagnosticLabel, ErrorCode,
    source::{SourceId, SourceSpan},
    syntax::{
        BinaryOp, DurationLiteral, Expr, ExprKind, FunctionDecl, Identifier, LetDecl, Parameter,
        RenderDecl, TokenKind, TopDecl, TypeAnnotation, TypeName, UnaryOp,
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

pub fn parse(source: &str, source_id: SourceId) -> Result<Program, Diagnostic> {
    parse_with_config(source, source_id, ParserConfig::default())
}

pub fn parse_with_config(
    source: &str,
    source_id: SourceId,
    config: ParserConfig,
) -> Result<Program, Diagnostic> {
    let tokens = lex(source, source_id)?;
    parse_tokens_with_config(&tokens, source_id, config)
}

pub fn parse_tokens(tokens: &[Token], source_id: SourceId) -> Result<Program, Diagnostic> {
    parse_tokens_with_config(tokens, source_id, ParserConfig::default())
}

pub fn parse_tokens_with_config(
    tokens: &[Token],
    source_id: SourceId,
    config: ParserConfig,
) -> Result<Program, Diagnostic> {
    Parser::new(tokens, source_id, config).parse_program()
}

pub struct Parser<'tokens> {
    tokens: &'tokens [Token],
    source_id: SourceId,
    current: usize,
    expression_depth: usize,
    nodes: usize,
    config: ParserConfig,
}

impl<'tokens> Parser<'tokens> {
    pub const fn new(tokens: &'tokens [Token], source_id: SourceId, config: ParserConfig) -> Self {
        Self {
            tokens,
            source_id,
            current: 0,
            expression_depth: 0,
            nodes: 0,
            config,
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, Diagnostic> {
        if let Some(token) = self
            .tokens
            .iter()
            .find(|token| token.span.source() != self.source_id)
        {
            return Err(self.unexpected_token(Some(token), Some("tokens from the expected source")));
        }

        let mut declarations = Vec::new();

        while matches!(self.peek().map(|token| &token.kind), Some(TokenKind::Let)) {
            declarations.push(self.parse_top_decl()?);
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
                    "this render is followed by another declaration",
                ),
            )
            .with_secondary(DiagnosticLabel::new(
                token.span,
                "move this declaration before the render declaration",
            )));
        }

        let mut first_out: Option<SourceSpan> = None;

        for declaration in &declarations {
            let TopDecl::Let(binding) = declaration else {
                continue;
            };
            if binding.name.name != "out" {
                continue;
            }

            if let Some(first_out) = first_out {
                return Err(Diagnostic::error(
                    ErrorCode::MultipleOutputs,
                    "multiple out bindings",
                    DiagnosticLabel::new(binding.name.span, "the second out binding is here"),
                )
                .with_secondary(DiagnosticLabel::new(
                    first_out,
                    "the first out binding is here",
                )));
            }

            first_out = Some(binding.name.span);
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

        let start_span = declarations
            .first()
            .map_or(render_keyword_span, TopDecl::span);

        let span = self.span_between(start_span, render.span)?;

        Ok(Program {
            declarations,
            render,
            span,
        })
    }

    fn peek(&self) -> Option<&'tokens Token> {
        self.tokens.get(self.current)
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

    fn eof_span(&self) -> SourceSpan {
        self.tokens.last().map_or_else(
            || SourceSpan::empty(self.source_id, 0),
            |token| SourceSpan::empty(self.source_id, token.span.end()),
        )
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

    fn span_here(&self) -> SourceSpan {
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
        let span = token.map_or_else(
            || self.eof_span(),
            |token| SourceSpan::empty(self.source_id, token.span.start()),
        );
        Diagnostic::error(
            ErrorCode::MissingSemicolon,
            "missing semicolon",
            DiagnosticLabel::new(span, "expected a semicolon here"),
        )
    }

    fn span_between(&self, start: SourceSpan, end: SourceSpan) -> Result<SourceSpan, Diagnostic> {
        if start.source() != end.source() {
            return Err(Diagnostic::error(
                ErrorCode::UnexpectedToken,
                "cannot combine spans from different sources",
                DiagnosticLabel::new(end, "this token belongs to another source"),
            ));
        }

        SourceSpan::new(start.source(), start.start(), end.end()).map_err(|_| {
            Diagnostic::error(
                ErrorCode::UnexpectedToken,
                "token spans are not ordered",
                DiagnosticLabel::new(end, "this token ends before the expression starts"),
            )
        })
    }

    fn parse_nested_expression(&mut self, opening_span: SourceSpan) -> Result<Expr, Diagnostic> {
        if self.expression_depth >= self.config.max_depth {
            return Err(Diagnostic::error(
                ErrorCode::ExpressionDepthLimit,
                "expression nesting limit exceeded",
                DiagnosticLabel::new(
                    opening_span,
                    format!(
                        "this expression exceeds the configured maximum depth of {}",
                        self.config.max_depth
                    ),
                ),
            ));
        }

        self.expression_depth += 1;
        let result = self.parse_expr();
        self.expression_depth -= 1;

        result
    }

    fn parse_expr(&mut self) -> Result<Expr, Diagnostic> {
        let start_span = self.span_here();
        let expression = self.parse_additive()?;

        if matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::PipeGreater)
        ) {
            return self.parse_pipeline_suffix(expression, start_span);
        }
        Ok(expression)
    }

    fn parse_pipeline_suffix(
        &mut self,
        mut left: Expr,
        start_span: SourceSpan,
    ) -> Result<Expr, Diagnostic> {
        while matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::PipeGreater)
        ) {
            self.expect(TokenKind::PipeGreater)?;

            let callee = self.expect_identifier()?;
            let left_span = left.span;
            let mut arguments = vec![left];
            let end_span = if matches!(
                self.peek().map(|token| &token.kind),
                Some(TokenKind::LeftParen)
            ) {
                let opening = self.expect(TokenKind::LeftParen)?;

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

                self.expect(TokenKind::RightParen)?.span
            } else {
                callee.span
            };

            let span = self.span_between(left_span, end_span)?;

            left = self.make_expr(ExprKind::Call { callee, arguments }, span, start_span)?;

            if matches!(
                self.peek().map(|token| &token.kind),
                Some(TokenKind::LeftParen)
            ) {
                return Err(self.unexpected_token(self.peek(), Some("only one call suffix")));
            }
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, Diagnostic> {
        let start_span = self.span_here();
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
                start_span,
            )?;
        }

        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, Diagnostic> {
        let start_span = self.span_here();
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
                start_span,
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
            minus.span,
        )
    }

    fn parse_unary_operand(&mut self, minus_span: SourceSpan) -> Result<Expr, Diagnostic> {
        if self.expression_depth >= self.config.max_depth {
            return Err(Diagnostic::error(
                ErrorCode::ExpressionDepthLimit,
                "expression nesting limit exceeded",
                DiagnosticLabel::new(
                    minus_span,
                    format!(
                        "this unary expression exceeds the configured maximum depth of {}",
                        self.config.max_depth
                    ),
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

    fn parse_top_decl(&mut self) -> Result<TopDecl, Diagnostic> {
        if self.function_decl_ahead() {
            return self.parse_function_decl().map(TopDecl::Function);
        }

        self.parse_let_decl().map(TopDecl::Let)
    }

    fn function_decl_ahead(&self) -> bool {
        if !matches!(
            self.tokens.get(self.current).map(|token| &token.kind),
            Some(TokenKind::Let)
        ) || !matches!(
            self.tokens.get(self.current + 1).map(|token| &token.kind),
            Some(TokenKind::Identifier(_))
        ) || !matches!(
            self.tokens.get(self.current + 2).map(|token| &token.kind),
            Some(TokenKind::Equal)
        ) || !matches!(
            self.tokens.get(self.current + 3).map(|token| &token.kind),
            Some(TokenKind::LeftParen)
        ) {
            return false;
        }

        let mut cursor = self.current + 4;
        loop {
            match self.tokens.get(cursor).map(|token| &token.kind) {
                Some(TokenKind::RightParen) => break,
                Some(TokenKind::Semicolon | TokenKind::Eof) | None => return false,
                Some(_) => cursor += 1,
            }
        }

        let result_type = self.tokens.get(cursor + 2).map(|token| &token.kind);
        let valid_result_type = matches!(result_type, Some(TokenKind::Float | TokenKind::Sig))
            || matches!(
                result_type,
                Some(TokenKind::Identifier(name)) if name == "Stereo"
            );

        matches!(
            self.tokens.get(cursor + 1).map(|token| &token.kind),
            Some(TokenKind::Colon | TokenKind::Float | TokenKind::Sig | TokenKind::FatArrow)
        ) && valid_result_type
            && matches!(
                self.tokens.get(cursor + 3).map(|token| &token.kind),
                Some(TokenKind::FatArrow)
            )
    }

    fn parse_function_decl(&mut self) -> Result<FunctionDecl, Diagnostic> {
        self.count_declaration()?;

        let token = self.expect(TokenKind::Let)?;
        let name = self.expect_identifier()?;

        self.expect(TokenKind::Equal)?;
        self.expect(TokenKind::LeftParen)?;

        let parameters = self.parse_parameters()?;

        self.expect(TokenKind::RightParen)?;

        let result = self.parse_annotation()?.ok_or_else(|| {
            self.unexpected_token(self.peek(), Some(" return type annotation after `)`"))
        })?;

        self.expect(TokenKind::FatArrow)?;

        let body = self.parse_expr()?;
        let end_function_decl = self.expect(TokenKind::Semicolon)?;
        let span = self.span_between(token.span, end_function_decl.span)?;

        Ok(FunctionDecl {
            name,
            parameters,
            result,
            body,
            span,
        })
    }

    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, Diagnostic> {
        let mut parameters = Vec::new();

        if !matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::Identifier(_))
        ) {
            return Ok(parameters);
        }

        loop {
            let identifier = self.expect_identifier()?;

            let type_annotation = self.parse_annotation()?.ok_or_else(|| {
                self.unexpected_token(
                    self.peek(),
                    Some("`: Float`, `: Sig` or `: Stereo` after the parameter name"),
                )
            })?;
            let span = self.span_between(identifier.span, type_annotation.span)?;

            parameters.push(Parameter {
                name: identifier,
                annotation: type_annotation,
                span,
            });

            if !matches!(self.peek().map(|token| &token.kind), Some(TokenKind::Comma)) {
                break;
            }

            self.advance();

            if !matches!(
                self.peek().map(|token| &token.kind),
                Some(TokenKind::Identifier(_))
            ) {
                return Err(self.unexpected_token(self.peek(), Some("a parameter after `,`")));
            }
        }

        Ok(parameters)
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
            .ok_or_else(|| self.unexpected_token(None, Some("Float, Sig or Stereo")))?;

        let (kind, span) = match &token.kind {
            TokenKind::Float => (TypeName::Float, token.span),
            TokenKind::Sig => (TypeName::Sig, token.span),
            TokenKind::Identifier(name) if name == "Stereo" => (TypeName::Stereo, token.span),
            _ => {
                return Err(self.unexpected_token(Some(token), Some("Float, Sig or Stereo")));
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

        Err(self.missing_semicolon(self.peek()))
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
                self.make_expr(ExprKind::Number(value), span, span)
            }
            TokenKind::Identifier(value) => {
                let identifier = Identifier::new(value.clone(), token.span);
                let span = token.span;

                self.advance();
                self.make_expr(ExprKind::Identifier(identifier), span, span)
            }
            TokenKind::LeftParen => self.parse_group(),
            TokenKind::LeftBrace => self.parse_block_expr(),
            _ => Err(self.unexpected_token(Some(token), Some("an expression"))),
        }
    }

    fn parse_block_expr(&mut self) -> Result<Expr, Diagnostic> {
        let start_block = self.expect(TokenKind::LeftBrace)?;

        if self.expression_depth >= self.config.max_depth {
            return Err(Diagnostic::error(
                ErrorCode::ExpressionDepthLimit,
                "expression nesting limit exceeded",
                DiagnosticLabel::new(
                    start_block.span,
                    format!(
                        "this block expression exceeds the configured maximum depth of {}",
                        self.config.max_depth
                    ),
                ),
            ));
        }

        self.expression_depth += 1;

        let parsed = (|| {
            let mut bindings = Vec::new();

            while matches!(self.peek().map(|token| &token.kind), Some(TokenKind::Let)) {
                bindings.push(self.parse_let_decl()?);
            }

            let result = self.parse_expr()?;
            Ok((bindings, result))
        })();

        self.expression_depth -= 1;

        let (bindings, result) = parsed?;

        let end_block = self.expect(TokenKind::RightBrace)?;
        let span = self.span_between(start_block.span, end_block.span)?;

        let block_expr = self.make_expr(
            ExprKind::Block {
                bindings,
                result: Box::new(result),
            },
            span,
            start_block.span,
        )?;

        Ok(block_expr)
    }

    fn parse_group(&mut self) -> Result<Expr, Diagnostic> {
        let opening = self.expect(TokenKind::LeftParen)?;
        let inner = self.parse_nested_expression(opening.span)?;
        let closing = self.expect(TokenKind::RightParen)?;
        let span = self.span_between(opening.span, closing.span)?;

        self.make_expr(ExprKind::Group(Box::new(inner)), span, opening.span)
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

    fn make_expr(
        &mut self,
        kind: ExprKind,
        span: SourceSpan,
        start_span: SourceSpan,
    ) -> Result<Expr, Diagnostic> {
        if self.nodes >= self.config.max_nodes {
            return Err(Diagnostic::error(
                ErrorCode::AstNodeLimit,
                "too many nodes in the AST",
                DiagnosticLabel::new(
                    start_span,
                    "this expression exceeds the maximum number of nodes",
                ),
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
