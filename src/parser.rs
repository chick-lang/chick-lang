use crate::ast::*;
use crate::token::Token;
use chumsky::{
    input::{Stream, ValueInput},
    prelude::*,
};

#[cfg(test)]
mod tests {
    use super::*;

    pub fn parse_stmt_tokens(tokens: Vec<Token>) -> Result<Program, Vec<Rich<'static, Token>>> {
        let dummy_span = SimpleSpan::from(0..0);

        let token_iter = tokens.into_iter().map(move |tok| (tok, dummy_span));

        let stream = Stream::from_iter(token_iter).map(dummy_span, |(tok, span)| (tok, span));

        parser().parse(stream).into_result()
    }

    #[test]
    fn one_word_test() {
        let result = parse_stmt_tokens(vec![
            Token::Break,
            Token::Semicolon,
            Token::Continue,
            Token::Semicolon,
        ]);
        assert_eq!(result, Ok(vec![Stmt::Break, Stmt::Continue]));
    }

    #[test]
    fn for_stmt_test() {
        let result = parse_stmt_tokens(vec![
            Token::For,
            Token::Identifier("i".into()),
            Token::In,
            Token::Literal(1.to_string()),
            Token::DotDot,
            Token::Literal(11.to_string()),
            Token::StartBrace,
            Token::Identifier("print".into()),
            Token::StartParenthese,
            Token::Identifier("i".into()),
            Token::FinishParenthese,
            Token::Semicolon,
            Token::FinishBrace,
        ]);
        assert_eq!(
            result,
            Ok(vec![Stmt::For {
                var: "i".into(),
                iter: Expr::Range(
                    Box::new(Expr::Int(1.to_string())),
                    Box::new(Expr::Int(11.to_string()))
                ),
                body: Box::new(vec![Stmt::Expr(Expr::Call {
                    calee: "print".into(),
                    args: vec![Expr::Variable("i".into())],
                })]),
            }])
        );
    }

    #[test]
    fn if_stmt_test() {
        let result = parse_stmt_tokens(vec![
            Token::If,
            Token::Identifier("n".into()),
            Token::Less,
            Token::Literal(10.to_string()),
            Token::StartBrace,
            Token::Identifier("print".into()),
            Token::StartParenthese,
            Token::Identifier("i".into()),
            Token::FinishParenthese,
            Token::Semicolon,
            Token::FinishBrace,
        ]);
        assert_eq!(
            result,
            Ok(vec![Stmt::If {
                cond: Expr::BinaryOp {
                    left: Box::new(Expr::Variable("n".into())),
                    op: BinaryOpKind::Less,
                    right: Box::new(Expr::Int(10.to_string())),
                },
                body: Box::new(vec![]),
                else_branch: None,
            }])
        );
    }

    #[test]
    fn if_else_test() {
        let result = parse_stmt_tokens(vec![
            Token::If,
            Token::False,
            Token::StartBrace,
            Token::FinishBrace,
            Token::Else,
            Token::If,
            Token::True,
            Token::StartBrace,
            Token::FinishBrace,
            Token::Else,
            Token::If,
            Token::False,
            Token::StartBrace,
            Token::FinishBrace,
            Token::Else,
            Token::StartBrace,
            Token::FinishBrace,
        ]);
        assert_eq!(
            result,
            Ok(vec![Stmt::If {
                cond: Expr::False,
                body: Box::new(vec![]),
                else_branch: Some(Box::new(vec![Stmt::If {
                    cond: Expr::True,
                    body: Box::new(vec![]),
                    else_branch: Some(Box::new(vec![Stmt::If {
                        cond: Expr::False,
                        body: Box::new(vec![]),
                        else_branch: Some(Box::new(vec![])),
                    }])),
                }])),
            }])
        );
    }

    #[test]
    fn let_stmt_test() {
        let result = parse_stmt_tokens(vec![
            Token::Let,
            Token::Identifier("x".into()),
            Token::Colon,
            Token::Identifier("f64".into()),
            Token::Equal,
            Token::Literal(1.414.to_string()),
            Token::Semicolon,
        ]);
        assert_eq!(
            result,
            Ok(vec![Stmt::Let {
                variable: ("x".into(), Some("f64".into())),
                initiator: Expr::Float(1.414.to_string()),
            }])
        );
    }

    #[test]
    fn let_calc_test() {
        let result = parse_stmt_tokens(vec![
            Token::Let,
            Token::Identifier("y".into()),
            Token::Equal,
            Token::Literal(22.to_string()),
            Token::Slash,
            Token::Literal(7.to_string()),
            Token::Semicolon,
        ]);
        assert_eq!(
            result,
            Ok(vec![Stmt::Let {
                variable: ("x".into(), None),
                initiator: Expr::BinaryOp {
                    left: Box::new(Expr::Int(22.to_string())),
                    op: BinaryOpKind::Div,
                    right: Box::new(Expr::Int(7.to_string())),
                },
            }])
        );
    }

    #[test]
    fn while_stmt_test() {
        let result = parse_stmt_tokens(vec![
            Token::While,
            Token::True,
            Token::StartBrace,
            Token::Identifier("print".into()),
            Token::StartParenthese,
            Token::String("\"Hello, World!\"".into()),
            Token::FinishParenthese,
            Token::Semicolon,
            Token::FinishBrace,
        ]);
        assert_eq!(
            result,
            Ok(vec![Stmt::While {
                cond: Expr::True,
                body: Box::new(vec![Stmt::Expr(Expr::Call {
                    calee: "print".into(),
                    args: vec![Expr::String("\"Hello, World!\"".into())],
                })]),
            }])
        );
    }

    pub fn parse_expr_tokens(tokens: Vec<Token>) -> Result<Expr, Vec<Rich<'static, Token>>> {
        let dummy_span = SimpleSpan::from(0..0);
        let token_iter = tokens.into_iter().map(move |tok| (tok, dummy_span));
        let stream = Stream::from_iter(token_iter).map(dummy_span, |(tok, span)| (tok, span));

        expr_parser().parse(stream).into_result()
    }

    #[test]
    fn literal_expr_test() {
        assert_eq!(
            parse_expr_tokens(vec![Token::Literal("42".into())]),
            Ok(Expr::Int("42".into()))
        );
        assert_eq!(
            parse_expr_tokens(vec![Token::Literal("3.14".into())]),
            Ok(Expr::Float("3.14".into()))
        );
        assert_eq!(
            parse_expr_tokens(vec![Token::String("\"hello\"".into())]),
            Ok(Expr::String("\"hello\"".into()))
        );
        assert_eq!(
            parse_expr_tokens(vec![Token::Char('あ')]),
            Ok(Expr::Char('あ'))
        );
        assert_eq!(parse_expr_tokens(vec![Token::True]), Ok(Expr::True));
        assert_eq!(parse_expr_tokens(vec![Token::False]), Ok(Expr::False));
    }

    #[test]
    fn binary_arithmetic_operators_test() {
        let ops = vec![
            (Token::Plus, BinaryOpKind::Plus),
            (Token::Minus, BinaryOpKind::Minus),
            (Token::Star, BinaryOpKind::Mult),
            (Token::Slash, BinaryOpKind::Div),
            (Token::Percent, BinaryOpKind::Mod),
            (Token::StarStar, BinaryOpKind::Power),
            (Token::Amp, BinaryOpKind::BitAnd),
            (Token::Vert, BinaryOpKind::BitOr),
            (Token::Caret, BinaryOpKind::Xor),
            (Token::AmpAmp, BinaryOpKind::LogAnd),
            (Token::VertVert, BinaryOpKind::LogOr),
        ];

        for (token, expected_op) in ops {
            let result = parse_expr_tokens(vec![
                Token::Literal("1".into()),
                token,
                Token::Literal("2".into()),
            ]);

            assert_eq!(
                result,
                Ok(Expr::BinaryOp {
                    left: Box::new(Expr::Int("1".into())),
                    op: expected_op,
                    right: Box::new(Expr::Int("2".into())),
                })
            );
        }
    }

    #[test]
    fn binary_comparison_operators_test() {
        let ops = vec![
            (Token::EqualEqual, BinaryOpKind::Equal),
            (Token::NotEqual, BinaryOpKind::NotEqual),
            (Token::Greater, BinaryOpKind::Greater),
            (Token::Less, BinaryOpKind::Less),
            (Token::GreaterEqual, BinaryOpKind::GreaterEqual),
            (Token::LessEqual, BinaryOpKind::LessEqual),
        ];

        for (token, expected_op) in ops {
            let result = parse_expr_tokens(vec![
                Token::Identifier("a".into()),
                token,
                Token::Identifier("b".into()),
            ]);

            assert_eq!(
                result,
                Ok(Expr::BinaryOp {
                    left: Box::new(Expr::Variable("a".into())),
                    op: expected_op,
                    right: Box::new(Expr::Variable("b".into())),
                })
            );
        }
    }

    #[test]
    fn binary_assignment_operators_test() {
        let ops = vec![
            (Token::Equal, BinaryOpKind::Assign),
            (Token::AmpAmpEqual, BinaryOpKind::AndAssign),
            (Token::VertVertEqual, BinaryOpKind::OrAssign),
            (Token::CaretEqual, BinaryOpKind::XorAssign),
            (Token::PlusEqual, BinaryOpKind::PlusAssign),
            (Token::MinusEqual, BinaryOpKind::MinusAssign),
            (Token::StarEqual, BinaryOpKind::MultAssign),
            (Token::SlashEqual, BinaryOpKind::DivAssign),
            (Token::PercentEqual, BinaryOpKind::ModAssign),
            (Token::StarStarEqual, BinaryOpKind::PowerAssign),
        ];

        for (token, expected_op) in ops {
            let result = parse_expr_tokens(vec![
                Token::Identifier("x".into()),
                token,
                Token::Literal("10".into()),
            ]);

            assert_eq!(
                result,
                Ok(Expr::BinaryOp {
                    left: Box::new(Expr::Variable("x".into())),
                    op: expected_op,
                    right: Box::new(Expr::Int("10".into())),
                })
            );
        }
    }

    #[test]
    fn call_expr_test() {
        let result = parse_expr_tokens(vec![
            Token::Identifier("add".into()),
            Token::StartParenthese,
            Token::Identifier("a".into()),
            Token::Comma,
            Token::Literal("10".into()),
            Token::FinishParenthese,
        ]);

        assert_eq!(
            result,
            Ok(Expr::Call {
                calee: "add".into(),
                args: vec![Expr::Variable("a".into()), Expr::Int("10".into()),],
            })
        );
    }

    #[test]
    fn class_expr_test() {
        let result = parse_expr_tokens(vec![
            Token::Class,
            Token::Identifier("MyClass".into()),
            Token::StartBrace,
            Token::Function,
            Token::Identifier("method".into()),
            Token::StartParenthese,
            Token::FinishParenthese,
            Token::StartBrace,
            Token::FinishBrace,
            Token::FinishBrace,
        ]);

        assert_eq!(
            result,
            Ok(Expr::Class {
                name: Some("MyClass".into()),
                body: vec![Stmt::Expr(Expr::Function {
                    name: Some("method".into()),
                    args: vec![],
                    body: vec![],
                })],
            })
        );
    }

    #[test]
    fn function_expr_test() {
        let result = parse_expr_tokens(vec![
            Token::Function,
            Token::Identifier("add".into()),
            Token::StartParenthese,
            Token::Identifier("a".into()),
            Token::Colon,
            Token::Identifier("i32".into()),
            Token::Comma,
            Token::Identifier("b".into()),
            Token::FinishParenthese,
            Token::StartBrace,
            Token::Return,
            Token::Identifier("a".into()),
            Token::Plus,
            Token::Identifier("b".into()),
            Token::Semicolon,
            Token::FinishBrace,
        ]);

        assert_eq!(
            result,
            Ok(Expr::Function {
                name: Some("add".into()),
                args: vec![("a".into(), Some("i32".into())), ("b".into(), None)],
                body: vec![Stmt::Return(Expr::BinaryOp {
                    left: Box::new(Expr::Variable("a".into())),
                    op: BinaryOpKind::Plus,
                    right: Box::new(Expr::Variable("b".into())),
                })],
            })
        );
    }

    #[test]
    fn anonymous_function_expr_test() {
        let result = parse_expr_tokens(vec![
            Token::Function,
            Token::StartParenthese,
            Token::FinishParenthese,
            Token::StartBrace,
            Token::FinishBrace,
        ]);

        assert_eq!(
            result,
            Ok(Expr::Function {
                name: None,
                args: vec![],
                body: vec![],
            })
        );
    }

    #[test]
    fn unary_operators_test() {
        // !a
        assert_eq!(
            parse_expr_tokens(vec![Token::Not, Token::Identifier("a".into())]),
            Ok(Expr::UnaryOp {
                op: UnaryOpKind::Not,
                target: Box::new(Expr::Variable("a".into())),
            })
        );

        // ++a
        assert_eq!(
            parse_expr_tokens(vec![Token::PlusPlus, Token::Identifier("a".into())]),
            Ok(Expr::UnaryOp {
                op: UnaryOpKind::Increment,
                target: Box::new(Expr::Variable("a".into())),
            })
        );

        // --a
        assert_eq!(
            parse_expr_tokens(vec![Token::MinusMinus, Token::Identifier("a".into())]),
            Ok(Expr::UnaryOp {
                op: UnaryOpKind::Decrement,
                target: Box::new(Expr::Variable("a".into())),
            })
        );
    }

    #[test]
    fn range_expr_test() {
        // Test: 0..10
        let result = parse_expr_tokens(vec![
            Token::Literal("0".into()),
            Token::DotDot,
            Token::Literal("10".into()),
        ]);

        assert_eq!(
            result,
            Ok(Expr::Range(
                Box::new(Expr::Int("0".into())),
                Box::new(Expr::Int("10".into()))
            ))
        );
    }
}
