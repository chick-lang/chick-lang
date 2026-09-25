#[derive(Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Literal(String),
    String(String),

    DotDot,

    Break,
    Continue,
    Else,
    For,
    Function,
    If,
    In,
    Let,
    Return,
    While,

    StartBrace,
    FinishBrace,
    StartBracket,
    FinishBracket,
    Colon,
    Comma,
    Dot,
    StartParenthese,
    FinishParenthese,
    Semicolon,

    Not,
    AmpAmp,
    VertVert,
    Caret,

    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    StarStar,
    Amp,
    Vert,

    Equal,
    AmpAmpEqual,
    VertVertEqual,
    CaretEqual,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    PercentEqual,
    StarStarEqual,

    PlusPlus,
    MinusMinus,

    EqualEqual,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifiers_and_literals() {
        let mut lex = Token::lexer("x _var123 123 45.67 \"hello world\" \"\"");

        assert_eq!(lex.next(), Some(Ok(Token::Identifier("x".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("_var123".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("123".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("45.67".into()))));
        assert_eq!(
            lex.next(),
            Some(Ok(Token::String("\"hello world\"".into())))
        );
        assert_eq!(lex.next(), Some(Ok(Token::String("\"\"".into()))));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_keywords() {
        let code = "break continue else for function if in let return while";
        let mut lex = Token::lexer(code);

        assert_eq!(lex.next(), Some(Ok(Token::Break)));
        assert_eq!(lex.next(), Some(Ok(Token::Continue)));
        assert_eq!(lex.next(), Some(Ok(Token::Else)));
        assert_eq!(lex.next(), Some(Ok(Token::For)));
        assert_eq!(lex.next(), Some(Ok(Token::Function)));
        assert_eq!(lex.next(), Some(Ok(Token::If)));
        assert_eq!(lex.next(), Some(Ok(Token::In)));
        assert_eq!(lex.next(), Some(Ok(Token::Let)));
        assert_eq!(lex.next(), Some(Ok(Token::Return)));
        assert_eq!(lex.next(), Some(Ok(Token::While)));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_delimiters_and_punctuation() {
        let mut lex = Token::lexer("{ } [ ] : , . ( ) ; ..");

        assert_eq!(lex.next(), Some(Ok(Token::StartBrace)));
        assert_eq!(lex.next(), Some(Ok(Token::FinishBrace)));
        assert_eq!(lex.next(), Some(Ok(Token::StartBracket)));
        assert_eq!(lex.next(), Some(Ok(Token::FinishBracket)));
        assert_eq!(lex.next(), Some(Ok(Token::Colon)));
        assert_eq!(lex.next(), Some(Ok(Token::Comma)));
        assert_eq!(lex.next(), Some(Ok(Token::Dot)));
        assert_eq!(lex.next(), Some(Ok(Token::StartParenthese)));
        assert_eq!(lex.next(), Some(Ok(Token::FinishParenthese)));
        assert_eq!(lex.next(), Some(Ok(Token::Semicolon)));
        assert_eq!(lex.next(), Some(Ok(Token::DotDot)));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_operators() {
        // Including check maximal cransh
        let code = "+ - * / % ** & | ^ ! && || ++ --";
        let mut lex = Token::lexer(code);

        assert_eq!(lex.next(), Some(Ok(Token::Plus)));
        assert_eq!(lex.next(), Some(Ok(Token::Minus)));
        assert_eq!(lex.next(), Some(Ok(Token::Star)));
        assert_eq!(lex.next(), Some(Ok(Token::Slash)));
        assert_eq!(lex.next(), Some(Ok(Token::Percent)));
        assert_eq!(lex.next(), Some(Ok(Token::StarStar)));
        assert_eq!(lex.next(), Some(Ok(Token::Amp)));
        assert_eq!(lex.next(), Some(Ok(Token::Vert)));
        assert_eq!(lex.next(), Some(Ok(Token::Caret)));
        assert_eq!(lex.next(), Some(Ok(Token::Not)));
        assert_eq!(lex.next(), Some(Ok(Token::AmpAmp)));
        assert_eq!(lex.next(), Some(Ok(Token::VertVert)));
        assert_eq!(lex.next(), Some(Ok(Token::PlusPlus)));
        assert_eq!(lex.next(), Some(Ok(Token::MinusMinus)));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_assignments_and_comparisons() {
        let code = "= &&= ||= ^= += -= *= /= %= **= == != > < >= <=";
        let mut lex = Token::lexer(code);

        assert_eq!(lex.next(), Some(Ok(Token::Equal)));
        assert_eq!(lex.next(), Some(Ok(Token::AmpAmpEqual)));
        assert_eq!(lex.next(), Some(Ok(Token::VertVertEqual)));
        assert_eq!(lex.next(), Some(Ok(Token::CaretEqual)));
        assert_eq!(lex.next(), Some(Ok(Token::PlusEqual)));
        assert_eq!(lex.next(), Some(Ok(Token::MinusEqual)));
        assert_eq!(lex.next(), Some(Ok(Token::StarEqual)));
        assert_eq!(lex.next(), Some(Ok(Token::SlashEqual)));
        assert_eq!(lex.next(), Some(Ok(Token::PercentEqual)));
        assert_eq!(lex.next(), Some(Ok(Token::StarStarEqual)));
        assert_eq!(lex.next(), Some(Ok(Token::EqualEqual)));
        assert_eq!(lex.next(), Some(Ok(Token::NotEqual)));
        assert_eq!(lex.next(), Some(Ok(Token::Greater)));
        assert_eq!(lex.next(), Some(Ok(Token::Less)));
        assert_eq!(lex.next(), Some(Ok(Token::GreaterEqual)));
        assert_eq!(lex.next(), Some(Ok(Token::LessEqual)));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_complex_statement() {
        let code = "let x = 10; if (x >= 5) { return x + 1; }";
        let mut lex = Token::lexer(code);

        assert_eq!(lex.next(), Some(Ok(Token::Let)));
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("x".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Equal)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("10".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Semicolon)));

        assert_eq!(lex.next(), Some(Ok(Token::If)));
        assert_eq!(lex.next(), Some(Ok(Token::StartParenthese)));
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("x".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::GreaterEqual)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("5".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::FinishParenthese)));

        assert_eq!(lex.next(), Some(Ok(Token::StartBrace)));
        assert_eq!(lex.next(), Some(Ok(Token::Return)));
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("x".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Plus)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("1".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Semicolon)));
        assert_eq!(lex.next(), Some(Ok(Token::FinishBrace)));

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_unknown_token() {
        let mut lex = Token::lexer("a @ b");

        assert_eq!(lex.next(), Some(Ok(Token::Identifier("a".into()))));
        assert_eq!(lex.next(), Some(Err(())));
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("b".into()))));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_whitespace_handling() {
        let code = "  \n\tlet /**/ /** @\n@ **/ x  // @";
        let mut lex = Token::lexer(code);

        assert_eq!(lex.next(), Some(Ok(Token::Let)));
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("x".into()))));
        assert_eq!(lex.next(), None);
    }
}
