mod lexer;

#[cfg(test)]
mod tests {
    use logos::Logos;

    use crate::lexer::Token;

    #[test]
    fn lexing_test() {
        let source = r#"
            fn main() {
                let tempo = signal 120;
                let note = "C4";
                emit tempo, 130;
            }
        "#;

        let mut lexer = Token::lexer(source);
        let tokens: Vec<Token> = lexer.collect::<Result<_, _>>().unwrap();
        for token in tokens.iter() {
            println!("{:?}", token);
        }
    }
}
