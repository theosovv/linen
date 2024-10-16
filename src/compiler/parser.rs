use std::collections::HashMap;

use super::token::{Token, TokenType};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

#[derive(Clone, Debug)]
pub struct Parser<'a> {
    pub current: Option<Token<'a>>,
    pub previous: Option<Token<'a>>,
    pub had_error: bool,
    pub panic_mode: bool,
}

impl<'a> Parser<'a> {
    pub fn new(current: Option<Token<'a>>, previous: Option<Token<'a>>) -> Self {
        Parser {
            current,
            previous,
            had_error: false,
            panic_mode: false,
        }
    }
}
