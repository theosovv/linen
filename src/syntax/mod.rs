mod ast;
mod lexer;
mod parser;
mod token;

use crate::Diagnostic;

pub use ast::{
    BinaryOp, DurationLiteral, Expr, ExprKind, Identifier, LetDecl, Program, RenderDecl,
    TypeAnnotation, TypeName, UnaryOp,
};
pub use lexer::{Lexer, lex};
pub use parser::{
    Parser, ParserConfig, parse, parse_tokens, parse_tokens_with_config, parse_with_config,
};
pub use token::{DurationUnit, Token, TokenKind};

pub fn check(source: &str) -> Result<Program, Diagnostic> {
    parse(source)
}
