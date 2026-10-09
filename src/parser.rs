use crate::ast::*;
use crate::token::Token;
use chumsky::{
    input::{Stream, ValueInput},
    prelude::*,
};

fn parser<'a, I>() -> impl Parser<'a, I, Program, extra::Err<Rich<'a, Token>>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    recursive(|stmt| {
        let ident = select! {
            Token::Identifier(name) => name,
        };
        let elseif_parser = just(Token::Else)
            .then(just(Token::If))
            .ignore_then(expr_parser())
            .then(
                stmt.clone()
                    .delimited_by(just(Token::StartBrace), just(Token::FinishBrace)),
            )
            .repeated()
            .collect::<Vec<_>>();
        let else_parser = just(Token::Else)
            .ignore_then(
                stmt.clone()
                    .delimited_by(just(Token::StartBrace), just(Token::FinishBrace)),
            )
            .or_not();
        choice((
            just(Token::Break)
                .then(just(Token::Semicolon))
                .to(Stmt::Break),
            just(Token::Continue)
                .then(just(Token::Semicolon))
                .to(Stmt::Continue),
            expr_parser()
                .then(just(Token::Semicolon))
                .map(|(expr, _)| Stmt::Expr(expr)),
            just(Token::For)
                .ignore_then(ident)
                .then_ignore(just(Token::In))
                .then(expr_parser())
                .then(
                    stmt.clone()
                        .delimited_by(just(Token::StartBrace), just(Token::FinishBrace)),
                )
                .map(|((var, iter), body)| Stmt::For {
                    var,
                    iter,
                    body: Box::new(body),
                }),
            just(Token::If)
                .ignore_then(expr_parser())
                .then(
                    stmt.clone()
                        .delimited_by(just(Token::StartBrace), just(Token::FinishBrace)),
                )
                .then(elseif_parser)
                .then(else_parser)
                .map(|(((cond, body), elseif_list), else_block)| {
                    let initial_else = else_block.map(Box::new);
                    let else_branch = elseif_list.into_iter().rfold(
                        initial_else,
                        |acc_else, (elem_cond, elem_body)| {
                            Some(Box::new(vec![Stmt::If {
                                cond: elem_cond,
                                body: Box::new(elem_body),
                                else_branch: acc_else,
                            }]))
                        },
                    );
                    Stmt::If {
                        cond,
                        body: Box::new(body),
                        else_branch,
                    }
                }),
            just(Token::Let)
                .ignore_then(ident)
                .then(just(Token::Colon).ignore_then(ident).or_not())
                .then(expr_parser().delimited_by(just(Token::Equal), just(Token::Semicolon)))
                .map(|(variable, initiator)| Stmt::Let {
                    variable,
                    initiator,
                }),
            just(Token::Return)
                .ignore_then(expr_parser())
                .map(|value| Stmt::Return(value)),
            just(Token::While)
                .ignore_then(expr_parser())
                .then(
                    stmt.clone()
                        .delimited_by(just(Token::StartBrace), just(Token::FinishBrace)),
                )
                .map(|(cond, body)| Stmt::While {
                    cond,
                    body: Box::new(body),
                }),
        ))
        .repeated()
        .collect()
    })
}

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
            Token::Int(1.to_string()),
            Token::DotDot,
            Token::Int(11.to_string()),
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
            Token::Int(10.to_string()),
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
            Token::Float(1.414.to_string()),
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
            Token::Int(22.to_string()),
            Token::Slash,
            Token::Int(7.to_string()),
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
            parse_expr_tokens(vec![Token::Int(42.to_string())]),
            Ok(Expr::Int(42.to_string()))
        );
        assert_eq!(
            parse_expr_tokens(vec![Token::Float(3.14.to_string())]),
            Ok(Expr::Float(3.14.to_string()))
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
                Token::Int(1.to_string()),
                token,
                Token::Int(2.to_string()),
            ]);

            assert_eq!(
                result,
                Ok(Expr::BinaryOp {
                    left: Box::new(Expr::Int(1.to_string())),
                    op: expected_op,
                    right: Box::new(Expr::Int(2.to_string())),
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
                Token::Int(10.to_string()),
            ]);

            assert_eq!(
                result,
                Ok(Expr::BinaryOp {
                    left: Box::new(Expr::Variable("x".into())),
                    op: expected_op,
                    right: Box::new(Expr::Int(10.to_string())),
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
            Token::Int(10.to_string()),
            Token::FinishParenthese,
        ]);

        assert_eq!(
            result,
            Ok(Expr::Call {
                calee: "add".into(),
                args: vec![Expr::Variable("a".into()), Expr::Int(10.to_string()),],
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
            Token::Int(0.to_string()),
            Token::DotDot,
            Token::Int(10.to_string()),
        ]);

        assert_eq!(
            result,
            Ok(Expr::Range(
                Box::new(Expr::Int(0.to_string())),
                Box::new(Expr::Int(10.to_string()))
            ))
        );
    }
}
