#[derive(Debug, Clone)]
pub enum AstNode {
    Program(Vec<AstNode>),
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
    Identifier(String),
    List(Vec<AstNode>),
    Symbol(String),
    Function {
        name: String,
        params: Vec<String>,
        body: Box<AstNode>,
    },
    Let {
        bindings: Vec<(String, AstNode)>,
        body: Box<AstNode>,
    },
    If {
        condition: Box<AstNode>,
        then_branch: Box<AstNode>,
        else_branch: Option<Box<AstNode>>,
    },
    Cond(Vec<(AstNode, Vec<AstNode>)>),
    Lambda {
        params: Vec<String>,
        body: Box<AstNode>,
    },
    FunctionCall {
        function: Box<AstNode>,
        arguments: Vec<AstNode>,
    },
    Else,
    Loop {
        bindings: Vec<(String, AstNode)>,
        body: Box<AstNode>,
    },
    Recur(Vec<AstNode>),
}
