use std::collections::HashMap;

use super::{parser::Precedence, token::TokenType, Compiler, ParseRule};

pub fn get_rules<'a>() -> HashMap<TokenType, ParseRule<'a>> {
    let mut rules = HashMap::new();

    rules.insert(
        TokenType::LeftParen,
        ParseRule {
            prefix: Some(Compiler::grouping_expression),
            infix: Some(Compiler::call),
            precedence: Precedence::Call,
        },
    );

    rules.insert(
        TokenType::RightParen,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::LeftBrace,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::RightBrace,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::Comma,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::Dot,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::Plus,
        ParseRule {
            prefix: None,
            infix: Some(Compiler::binary_expression),
            precedence: Precedence::Term,
        },
    );

    rules.insert(
        TokenType::Minus,
        ParseRule {
            prefix: Some(Compiler::unary_expression),
            infix: Some(Compiler::binary_expression),
            precedence: Precedence::Term,
        },
    );

    rules.insert(
        TokenType::Semicolon,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::Slash,
        ParseRule {
            prefix: None,
            infix: Some(Compiler::binary_expression),
            precedence: Precedence::Factor,
        },
    );

    rules.insert(
        TokenType::Star,
        ParseRule {
            prefix: None,
            infix: Some(Compiler::binary_expression),
            precedence: Precedence::Factor,
        },
    );

    rules.insert(
        TokenType::Bang,
        ParseRule {
            prefix: Some(Compiler::unary_expression),
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::BangEqual,
        ParseRule {
            prefix: None,
            infix: Some(Compiler::binary_expression),
            precedence: Precedence::Equality,
        },
    );

    rules.insert(
        TokenType::Equal,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::EqualEqual,
        ParseRule {
            prefix: None,
            infix: Some(Compiler::binary_expression),
            precedence: Precedence::Equality,
        },
    );

    rules.insert(
        TokenType::Greater,
        ParseRule {
            prefix: None,
            infix: Some(Compiler::binary_expression),
            precedence: Precedence::Comparison,
        },
    );

    rules.insert(
        TokenType::GreaterEqual,
        ParseRule {
            prefix: None,
            infix: Some(Compiler::binary_expression),
            precedence: Precedence::Comparison,
        },
    );

    rules.insert(
        TokenType::Less,
        ParseRule {
            prefix: None,
            infix: Some(Compiler::binary_expression),
            precedence: Precedence::Comparison,
        },
    );

    rules.insert(
        TokenType::LessEqual,
        ParseRule {
            prefix: None,
            infix: Some(Compiler::binary_expression),
            precedence: Precedence::Comparison,
        },
    );

    rules.insert(
        TokenType::Identifier,
        ParseRule {
            prefix: Some(Compiler::variable),
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::String,
        ParseRule {
            prefix: Some(Compiler::string),
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::Number,
        ParseRule {
            prefix: Some(Compiler::number),
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::And,
        ParseRule {
            prefix: None,
            infix: Some(Compiler::and),
            precedence: Precedence::And,
        },
    );

    rules.insert(
        TokenType::Class,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::Else,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::False,
        ParseRule {
            prefix: Some(Compiler::literal),
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::Fun,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::For,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::If,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::Nil,
        ParseRule {
            prefix: Some(Compiler::literal),
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::Or,
        ParseRule {
            prefix: None,
            infix: Some(Compiler::or),
            precedence: Precedence::Or,
        },
    );

    rules.insert(
        TokenType::Print,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::Return,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::Super,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::This,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::True,
        ParseRule {
            prefix: Some(Compiler::literal),
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::Var,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::While,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::Error,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules.insert(
        TokenType::EOF,
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    );

    rules
}
