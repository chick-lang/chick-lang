use crate::ast::*;
use crate::token::Token;
use chumsky::{
    input::{Stream, ValueInput},
    pratt::*,
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
        let assignments = choice((
            ident
                .then_ignore(just(Token::Equal))
                .then(expr_parser())
                .then_ignore(just(Token::Semicolon))
                .map(|(left, right)| Stmt::Assign {
                    target: left,
                    op: None,
                    value: right,
                }),
            ident
                .then_ignore(just(Token::AmpAmpEqual))
                .then(expr_parser())
                .then_ignore(just(Token::Semicolon))
                .map(|(left, right)| Stmt::Assign {
                    target: left,
                    op: Some(BinaryOpKind::LogAnd),
                    value: right,
                }),
            ident
                .then_ignore(just(Token::VertVertEqual))
                .then(expr_parser())
                .then_ignore(just(Token::Semicolon))
                .map(|(left, right)| Stmt::Assign {
                    target: left,
                    op: Some(BinaryOpKind::LogOr),
                    value: right,
                }),
            ident
                .then_ignore(just(Token::CaretEqual))
                .then(expr_parser())
                .then_ignore(just(Token::Semicolon))
                .map(|(left, right)| Stmt::Assign {
                    target: left,
                    op: Some(BinaryOpKind::Xor),
                    value: right,
                }),
            ident
                .then_ignore(just(Token::PlusEqual))
                .then(expr_parser())
                .then_ignore(just(Token::Semicolon))
                .map(|(left, right)| Stmt::Assign {
                    target: left,
                    op: Some(BinaryOpKind::Plus),
                    value: right,
                }),
            ident
                .then_ignore(just(Token::MinusEqual))
                .then(expr_parser())
                .then_ignore(just(Token::Semicolon))
                .map(|(left, right)| Stmt::Assign {
                    target: left,
                    op: Some(BinaryOpKind::Minus),
                    value: right,
                }),
            ident
                .then_ignore(just(Token::StarEqual))
                .then(expr_parser())
                .then_ignore(just(Token::Semicolon))
                .map(|(left, right)| Stmt::Assign {
                    target: left,
                    op: Some(BinaryOpKind::Mult),
                    value: right,
                }),
            ident
                .then_ignore(just(Token::SlashEqual))
                .then(expr_parser())
                .then_ignore(just(Token::Semicolon))
                .map(|(left, right)| Stmt::Assign {
                    target: left,
                    op: Some(BinaryOpKind::Div),
                    value: right,
                }),
            ident
                .then_ignore(just(Token::StarStarEqual))
                .then(expr_parser())
                .then_ignore(just(Token::Semicolon))
                .map(|(left, right)| Stmt::Assign {
                    target: left,
                    op: Some(BinaryOpKind::Power),
                    value: right,
                }),
        ));
        choice((
            assignments,
            just(Token::Break)
                .then(just(Token::Semicolon))
                .to(Stmt::Break),
            just(Token::Continue)
                .then(just(Token::Semicolon))
                .to(Stmt::Continue),
            expr_parser()
                .then_ignore(just(Token::Semicolon))
                .map(|expr| Stmt::Expr(expr)),
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

fn expr_parser<'a, I>() -> impl Parser<'a, I, Expr, extra::Err<Rich<'a, Token>>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    recursive(|expr| {
        let ident = select! {
            Token::Identifier(name) => name,
        };
        let atom_parser = expr.clone();
        let operators = atom_parser.pratt((
            infix(
                chumsky::pratt::left(1),
                just(Token::VertVert),
                |l, _, r, _| Expr::BinaryOp {
                    left: Box::new(l),
                    op: BinaryOpKind::LogOr,
                    right: Box::new(r),
                },
            ),
            infix(
                chumsky::pratt::left(2),
                just(Token::AmpAmp),
                |l, _, r, _| Expr::BinaryOp {
                    left: Box::new(l),
                    op: BinaryOpKind::LogAnd,
                    right: Box::new(r),
                },
            ),
            prefix(3, just(Token::Not), |_, t, _| Expr::UnaryOp {
                target: Box::new(t),
                op: UnaryOpKind::Not,
            }),
            infix(
                chumsky::pratt::left(4),
                just(Token::EqualEqual),
                |l, _, r, _| Expr::BinaryOp {
                    left: Box::new(l),
                    op: BinaryOpKind::Equal,
                    right: Box::new(r),
                },
            ),
            infix(
                chumsky::pratt::left(4),
                just(Token::NotEqual),
                |l, _, r, _| Expr::BinaryOp {
                    left: Box::new(l),
                    op: BinaryOpKind::NotEqual,
                    right: Box::new(r),
                },
            ),
            infix(chumsky::pratt::left(4), just(Token::Less), |l, _, r, _| {
                Expr::BinaryOp {
                    left: Box::new(l),
                    op: BinaryOpKind::Less,
                    right: Box::new(r),
                }
            }),
            infix(
                chumsky::pratt::left(4),
                just(Token::GreaterEqual),
                |l, _, r, _| Expr::BinaryOp {
                    left: Box::new(l),
                    op: BinaryOpKind::GreaterEqual,
                    right: Box::new(r),
                },
            ),
            infix(
                chumsky::pratt::left(4),
                just(Token::LessEqual),
                |l, _, r, _| Expr::BinaryOp {
                    left: Box::new(l),
                    op: BinaryOpKind::LessEqual,
                    right: Box::new(r),
                },
            ),
            infix(chumsky::pratt::left(5), just(Token::Vert), |l, _, r, _| {
                Expr::BinaryOp {
                    left: Box::new(l),
                    op: BinaryOpKind::BitOr,
                    right: Box::new(r),
                }
            }),
            infix(chumsky::pratt::left(7), just(Token::Amp), |l, _, r, _| {
                Expr::BinaryOp {
                    left: Box::new(l),
                    op: BinaryOpKind::BitAnd,
                    right: Box::new(r),
                }
            }),
            infix(chumsky::pratt::left(8), just(Token::Caret), |l, _, r, _| {
                Expr::BinaryOp {
                    left: Box::new(l),
                    op: BinaryOpKind::Xor,
                    right: Box::new(r),
                }
            }),
            infix(
                chumsky::pratt::left(8),
                just(Token::DotDot),
                |l, _, r, _| Expr::BinaryOp {
                    left: Box::new(l),
                    op: BinaryOpKind::Range,
                    right: Box::new(r),
                },
            ),
            infix(chumsky::pratt::left(9), just(Token::Plus), |l, _, r, _| {
                Expr::BinaryOp {
                    left: Box::new(l),
                    op: BinaryOpKind::Plus,
                    right: Box::new(r),
                }
            }),
            infix(chumsky::pratt::left(9), just(Token::Minus), |l, _, r, _| {
                Expr::BinaryOp {
                    left: Box::new(l),
                    op: BinaryOpKind::Minus,
                    right: Box::new(r),
                }
            }),
            infix(chumsky::pratt::left(10), just(Token::Star), |l, _, r, _| {
                Expr::BinaryOp {
                    left: Box::new(l),
                    op: BinaryOpKind::Mult,
                    right: Box::new(r),
                }
            }),
            infix(
                chumsky::pratt::left(10),
                just(Token::Slash),
                |l, _, r, _| Expr::BinaryOp {
                    left: Box::new(l),
                    op: BinaryOpKind::Div,
                    right: Box::new(r),
                },
            ),
            infix(
                chumsky::pratt::left(10),
                just(Token::Percent),
                |l, _, r, _| Expr::BinaryOp {
                    left: Box::new(l),
                    op: BinaryOpKind::Mod,
                    right: Box::new(r),
                },
            ),
            infix(
                chumsky::pratt::left(11),
                just(Token::StarStar),
                |l, _, r, _| Expr::BinaryOp {
                    left: Box::new(l),
                    op: BinaryOpKind::Power,
                    right: Box::new(r),
                },
            ),
            prefix(12, just(Token::Plus), |_, t, _| Expr::UnaryOp {
                target: Box::new(t),
                op: UnaryOpKind::Plus,
            }),
            prefix(12, just(Token::Minus), |_, t, _| Expr::UnaryOp {
                target: Box::new(t),
                op: UnaryOpKind::Minus,
            }),
            prefix(12, just(Token::PlusPlus), |_, t, _| Expr::UnaryOp {
                target: Box::new(t),
                op: UnaryOpKind::PreIncrement,
            }),
            prefix(12, just(Token::MinusMinus), |_, t, _| Expr::UnaryOp {
                target: Box::new(t),
                op: UnaryOpKind::PreDecrement,
            }),
            postfix(13, just(Token::PlusPlus), |t, _, _| Expr::UnaryOp {
                target: Box::new(t),
                op: UnaryOpKind::AfterDecrement,
            }),
            postfix(13, just(Token::MinusMinus), |t, _, _| Expr::UnaryOp {
                target: Box::new(t),
                op: UnaryOpKind::AfterDecrement,
            }),
        ));
        choice((
            expr.clone()
                .delimited_by(just(Token::StartParenthese), just(Token::FinishParenthese)),
            operators,
            ident
                .then(
                    expr.clone()
                        .separated_by(just(Token::Comma))
                        .allow_trailing()
                        .collect::<Vec<_>>()
                        .delimited_by(just(Token::StartParenthese), just(Token::FinishParenthese)),
                )
                .map(|(calee, args)| Expr::Call { calee, args }),
            select! { Token::Char(c) => Expr::Char(c) },
            just(Token::False).to(Expr::False),
            select! { Token::Float(f) => Expr::Float(f) },
            just(Token::Function)
                .ignore_then(ident.or_not())
                .then(
                    ident
                        .then(just(Token::Colon).ignore_then(ident).or_not())
                        .separated_by(just(Token::Comma))
                        .allow_trailing()
                        .collect::<Vec<_>>()
                        .delimited_by(just(Token::StartParenthese), just(Token::FinishParenthese)),
                )
                .then(parser().delimited_by(just(Token::StartBrace), just(Token::FinishBrace)))
                .map(|((name, args), body)| Expr::Function { name, args, body }),
            select! { Token::Int(i) => Expr::Int(i) },
            select! { Token::String(s) => Expr::String(s) },
            just(Token::True).to(Expr::True),
            ident.map(|name| Expr::Variable(name)),
        ))
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
    fn assignment_stmt_test() {
        let ops = vec![
            (Token::Equal, None),
            (Token::AmpAmpEqual, Some(BinaryOpKind::LogAnd)),
            (Token::VertVertEqual, Some(BinaryOpKind::LogOr)),
            (Token::CaretEqual, Some(BinaryOpKind::Xor)),
            (Token::PlusEqual, Some(BinaryOpKind::Plus)),
            (Token::MinusEqual, Some(BinaryOpKind::Minus)),
            (Token::StarEqual, Some(BinaryOpKind::Mult)),
            (Token::SlashEqual, Some(BinaryOpKind::Div)),
            (Token::PercentEqual, Some(BinaryOpKind::Mod)),
            (Token::StarStarEqual, Some(BinaryOpKind::Power)),
        ];

        for (token, expected_op) in ops {
            let result = parse_stmt_tokens(vec![
                Token::Identifier("x".into()),
                token,
                Token::Int(10.to_string()),
                Token::Semicolon,
            ]);

            assert_eq!(
                result,
                Ok(vec![Stmt::Assign {
                    target: "x".into(),
                    op: expected_op,
                    value: Expr::Int(10.to_string()),
                }])
            );
        }
    }

    #[test]
    fn one_word_stmt_test() {
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
                iter: Expr::BinaryOp {
                    left: Box::new(Expr::Int(1.to_string())),
                    op: BinaryOpKind::Range,
                    right: Box::new(Expr::Int(11.to_string())),
                },
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
                body: Box::new(vec![Stmt::Expr(Expr::Call {
                    calee: "print".into(),
                    args: vec![Expr::Variable("i".into())],
                })]),
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
                variable: ("y".into(), None),
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
    fn binary_range_operators_test() {
        let result = parse_expr_tokens(vec![
            Token::Int(0.to_string()),
            Token::DotDot,
            Token::Int(10.to_string()),
        ]);

        assert_eq!(
            result,
            Ok(Expr::BinaryOp {
                left: Box::new(Expr::Int(0.to_string())),
                op: BinaryOpKind::Range,
                right: Box::new(Expr::Int(10.to_string()))
            })
        );
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

        // +a
        assert_eq!(
            parse_expr_tokens(vec![Token::Plus, Token::Identifier("a".into())]),
            Ok(Expr::UnaryOp {
                op: UnaryOpKind::Plus,
                target: Box::new(Expr::Variable("a".into())),
            })
        );

        // -a
        assert_eq!(
            parse_expr_tokens(vec![Token::Minus, Token::Identifier("a".into())]),
            Ok(Expr::UnaryOp {
                op: UnaryOpKind::Minus,
                target: Box::new(Expr::Variable("a".into())),
            })
        );

        // ++a
        assert_eq!(
            parse_expr_tokens(vec![Token::PlusPlus, Token::Identifier("a".into())]),
            Ok(Expr::UnaryOp {
                op: UnaryOpKind::PreIncrement,
                target: Box::new(Expr::Variable("a".into())),
            })
        );

        // --a
        assert_eq!(
            parse_expr_tokens(vec![Token::MinusMinus, Token::Identifier("a".into())]),
            Ok(Expr::UnaryOp {
                op: UnaryOpKind::PreDecrement,
                target: Box::new(Expr::Variable("a".into())),
            })
        );

        // ++a
        assert_eq!(
            parse_expr_tokens(vec![Token::Identifier("a".into()), Token::PlusPlus]),
            Ok(Expr::UnaryOp {
                op: UnaryOpKind::AfterIncrement,
                target: Box::new(Expr::Variable("a".into())),
            })
        );

        // --a
        assert_eq!(
            parse_expr_tokens(vec![Token::Identifier("a".into()), Token::MinusMinus]),
            Ok(Expr::UnaryOp {
                op: UnaryOpKind::AfterDecrement,
                target: Box::new(Expr::Variable("a".into())),
            })
        );
    }
}
