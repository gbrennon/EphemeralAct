use super::{CompareOp, Expr, Lexer, Literal, LogicalOp, Token, parse_error::ParseError};

/// Recursive-descent parser for expression tokens.
pub struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    /// Creates a new parser from a token slice.
    #[must_use]
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parses the full token stream into an [`Expr`] AST.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] if the token stream is malformed.
    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_expr()?;
        if self.pos < self.tokens.len() {
            return Err(self.error("unexpected tokens after expression"));
        }
        Ok(expr)
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_logical()
    }

    fn parse_logical(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_compare()?;
        loop {
            let op = match self.peek() {
                Some(Token::And) => LogicalOp::And,
                Some(Token::Or) => LogicalOp::Or,
                _ => break,
            };
            self.advance();
            let right = self.parse_compare()?;
            left = Expr::Logical(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_compare(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_unary()?;
        let op = match self.peek() {
            Some(Token::Eq) => CompareOp::Eq,
            Some(Token::Neq) => CompareOp::Neq,
            Some(Token::Lt) => CompareOp::Lt,
            Some(Token::Lte) => CompareOp::Lte,
            Some(Token::Gt) => CompareOp::Gt,
            Some(Token::Gte) => CompareOp::Gte,
            _ => return Ok(left),
        };
        self.advance();
        let right = self.parse_unary()?;
        Ok(Expr::Compare(op, Box::new(left), Box::new(right)))
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.peek() == Some(&Token::Not) {
            self.advance();
            let inner = self.parse_unary()?;
            return Ok(Expr::Not(Box::new(inner)));
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                Some(Token::Dot) => {
                    self.advance();
                    if self.peek() == Some(&Token::Star) {
                        self.advance();
                        expr = Expr::ArrayDeref(Box::new(expr));
                    } else {
                        let ident = self.expect_ident("property name after '.'")?;
                        expr = Expr::PropertyAccess(Box::new(expr), ident);
                    }
                }
                Some(Token::LBracket) => {
                    self.advance();
                    let idx = self.parse_expr()?;
                    self.expect(Token::RBracket, "expected ']'")?;
                    expr = Expr::IndexAccess(Box::new(expr), Box::new(idx));
                }
                Some(Token::LParen) => {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(Token::RParen, "expected ')'")?;
                    let name = match &expr {
                        Expr::Variable(n) => n.clone(),
                        _ => {
                            return Err(
                                self.error("function call requires a function name before '('")
                            );
                        }
                    };
                    expr = Expr::FuncCall(name, args);
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek().cloned() {
            Some(Token::Ident(name)) => {
                self.advance();
                match name.as_str() {
                    "true" => return Ok(Expr::Literal(Literal::Bool(true))),
                    "false" => return Ok(Expr::Literal(Literal::Bool(false))),
                    "null" => return Ok(Expr::Literal(Literal::Null)),
                    _ => {}
                }
                Ok(Expr::Variable(name))
            }
            Some(Token::String(s)) => {
                self.advance();
                Ok(Expr::Literal(Literal::String(s)))
            }
            Some(Token::Int(n)) => {
                self.advance();
                Ok(Expr::Literal(Literal::Int(n)))
            }
            Some(Token::Float(f)) => {
                self.advance();
                Ok(Expr::Literal(Literal::Float(f)))
            }
            Some(Token::Bool(b)) => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(b)))
            }
            Some(Token::Null) => {
                self.advance();
                Ok(Expr::Literal(Literal::Null))
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RParen, "expected ')'")?;
                Ok(expr)
            }
            Some(other) => Err(self.error(&format!("unexpected token: {other:?}"))),
            None => Err(self.error("unexpected end of expression")),
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        if self.peek() == Some(&Token::RParen) {
            return Ok(args);
        }
        loop {
            args.push(self.parse_expr()?);
            if self.peek() == Some(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(args)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, expected: Token, msg: &str) -> Result<(), ParseError> {
        match self.peek() {
            Some(t) if *t == expected => {
                self.advance();
                Ok(())
            }
            Some(t) => Err(self.error(&format!("{msg}, found {t:?}"))),
            None => Err(self.error(&format!("{msg}, found end of input"))),
        }
    }

    fn expect_ident(&mut self, msg: &str) -> Result<String, ParseError> {
        match self.peek() {
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            Some(t) => Err(self.error(&format!("{msg}, found {t:?}"))),
            None => Err(self.error(&format!("{msg}, found end of input"))),
        }
    }

    fn error(&self, msg: &str) -> ParseError {
        ParseError {
            message: msg.to_string(),
            position: self.pos,
        }
    }
}

/// Convenience function: lexes and parses an expression string.
///
/// # Errors
///
/// Returns [`ParseError`] if lexing or parsing fails.
pub fn parse_expr(input: &str) -> Result<Expr, ParseError> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    loop {
        match lexer.next_token() {
            Ok(Token::Eof) => break,
            Ok(tok) => tokens.push(tok),
            Err(e) => {
                return Err(ParseError {
                    message: format!("lexer error: {e}"),
                    position: 0,
                });
            }
        }
    }
    let mut parser = Parser::new(&tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<Expr, ParseError> {
        parse_expr(input)
    }

    #[test]
    fn parse_bool_true() {
        let expr = parse("true").unwrap();
        assert_eq!(expr, Expr::Literal(Literal::Bool(true)));
    }

    #[test]
    fn parse_bool_false() {
        let expr = parse("false").unwrap();
        assert_eq!(expr, Expr::Literal(Literal::Bool(false)));
    }

    #[test]
    fn parse_null() {
        let expr = parse("null").unwrap();
        assert_eq!(expr, Expr::Literal(Literal::Null));
    }

    #[test]
    fn parse_int() {
        let expr = parse("42").unwrap();
        assert_eq!(expr, Expr::Literal(Literal::Int(42)));
    }

    #[test]
    fn parse_negative_int() {
        let expr = parse("7").unwrap();
        assert_eq!(expr, Expr::Literal(Literal::Int(7)));
    }

    #[test]
    fn parse_float() {
        let expr = parse("2.71").unwrap();
        assert_eq!(expr, Expr::Literal(Literal::Float(2.71)));
    }

    #[test]
    fn parse_string() {
        let expr = parse("'hello'").unwrap();
        assert_eq!(expr, Expr::Literal(Literal::String("hello".into())));
    }

    #[test]
    fn parse_variable() {
        let expr = parse("github").unwrap();
        assert_eq!(expr, Expr::Variable("github".into()));
    }

    #[test]
    fn parse_variable_env() {
        let expr = parse("env").unwrap();
        assert_eq!(expr, Expr::Variable("env".into()));
    }

    #[test]
    fn parse_property_access() {
        let expr = parse("github.ref").unwrap();
        assert_eq!(
            expr,
            Expr::PropertyAccess(Box::new(Expr::Variable("github".into())), "ref".into())
        );
    }

    #[test]
    fn parse_nested_property_access() {
        let expr = parse("github.event_name").unwrap();
        assert_eq!(
            expr,
            Expr::PropertyAccess(
                Box::new(Expr::Variable("github".into())),
                "event_name".into()
            )
        );
    }

    #[test]
    fn parse_deep_property_access() {
        let expr = parse("a.b.c").unwrap();
        assert_eq!(
            expr,
            Expr::PropertyAccess(
                Box::new(Expr::PropertyAccess(
                    Box::new(Expr::Variable("a".into())),
                    "b".into()
                )),
                "c".into()
            )
        );
    }

    #[test]
    fn parse_index_access() {
        let expr = parse("arr[0]").unwrap();
        assert_eq!(
            expr,
            Expr::IndexAccess(
                Box::new(Expr::Variable("arr".into())),
                Box::new(Expr::Literal(Literal::Int(0)))
            )
        );
    }

    #[test]
    fn parse_index_access_string_key() {
        let expr = parse("obj['key']").unwrap();
        assert_eq!(
            expr,
            Expr::IndexAccess(
                Box::new(Expr::Variable("obj".into())),
                Box::new(Expr::Literal(Literal::String("key".into())))
            )
        );
    }

    #[test]
    fn parse_array_deref() {
        let expr = parse("foo.*").unwrap();
        assert_eq!(
            expr,
            Expr::ArrayDeref(Box::new(Expr::Variable("foo".into())))
        );
    }

    #[test]
    fn parse_func_call_no_args() {
        let expr = parse("success()").unwrap();
        assert_eq!(expr, Expr::FuncCall("success".into(), vec![]));
    }

    #[test]
    fn parse_func_call_one_arg() {
        let expr = parse("always()").unwrap();
        assert_eq!(expr, Expr::FuncCall("always".into(), vec![]));
    }

    #[test]
    fn parse_func_call_two_args() {
        let expr = parse("contains('hello', 'll')").unwrap();
        assert_eq!(
            expr,
            Expr::FuncCall(
                "contains".into(),
                vec![
                    Expr::Literal(Literal::String("hello".into())),
                    Expr::Literal(Literal::String("ll".into()))
                ]
            )
        );
    }

    #[test]
    fn parse_eq() {
        let expr = parse("a == b").unwrap();
        assert_eq!(
            expr,
            Expr::Compare(
                CompareOp::Eq,
                Box::new(Expr::Variable("a".into())),
                Box::new(Expr::Variable("b".into()))
            )
        );
    }

    #[test]
    fn parse_neq() {
        let expr = parse("a != b").unwrap();
        assert_eq!(
            expr,
            Expr::Compare(
                CompareOp::Neq,
                Box::new(Expr::Variable("a".into())),
                Box::new(Expr::Variable("b".into()))
            )
        );
    }

    #[test]
    fn parse_lt() {
        let expr = parse("a < b").unwrap();
        assert_eq!(
            expr,
            Expr::Compare(
                CompareOp::Lt,
                Box::new(Expr::Variable("a".into())),
                Box::new(Expr::Variable("b".into()))
            )
        );
    }

    #[test]
    fn parse_gt() {
        let expr = parse("a > b").unwrap();
        assert_eq!(
            expr,
            Expr::Compare(
                CompareOp::Gt,
                Box::new(Expr::Variable("a".into())),
                Box::new(Expr::Variable("b".into()))
            )
        );
    }

    #[test]
    fn parse_lte() {
        let expr = parse("a <= b").unwrap();
        assert_eq!(
            expr,
            Expr::Compare(
                CompareOp::Lte,
                Box::new(Expr::Variable("a".into())),
                Box::new(Expr::Variable("b".into()))
            )
        );
    }

    #[test]
    fn parse_gte() {
        let expr = parse("a >= b").unwrap();
        assert_eq!(
            expr,
            Expr::Compare(
                CompareOp::Gte,
                Box::new(Expr::Variable("a".into())),
                Box::new(Expr::Variable("b".into()))
            )
        );
    }

    #[test]
    fn parse_and() {
        let expr = parse("a && b").unwrap();
        assert_eq!(
            expr,
            Expr::Logical(
                LogicalOp::And,
                Box::new(Expr::Variable("a".into())),
                Box::new(Expr::Variable("b".into()))
            )
        );
    }

    #[test]
    fn parse_or() {
        let expr = parse("a || b").unwrap();
        assert_eq!(
            expr,
            Expr::Logical(
                LogicalOp::Or,
                Box::new(Expr::Variable("a".into())),
                Box::new(Expr::Variable("b".into()))
            )
        );
    }

    #[test]
    fn parse_and_or_left_assoc() {
        let expr = parse("a && b || c").unwrap();
        assert_eq!(
            expr,
            Expr::Logical(
                LogicalOp::Or,
                Box::new(Expr::Logical(
                    LogicalOp::And,
                    Box::new(Expr::Variable("a".into())),
                    Box::new(Expr::Variable("b".into()))
                )),
                Box::new(Expr::Variable("c".into()))
            )
        );
    }

    #[test]
    fn parse_or_and_left_assoc() {
        let expr = parse("a || b && c").unwrap();
        assert_eq!(
            expr,
            Expr::Logical(
                LogicalOp::And,
                Box::new(Expr::Logical(
                    LogicalOp::Or,
                    Box::new(Expr::Variable("a".into())),
                    Box::new(Expr::Variable("b".into()))
                )),
                Box::new(Expr::Variable("c".into()))
            )
        );
    }

    #[test]
    fn parse_not() {
        let expr = parse("!a").unwrap();
        assert_eq!(expr, Expr::Not(Box::new(Expr::Variable("a".into()))));
    }

    #[test]
    fn parse_double_not() {
        let expr = parse("!!a").unwrap();
        assert_eq!(
            expr,
            Expr::Not(Box::new(Expr::Not(Box::new(Expr::Variable("a".into())))))
        );
    }

    #[test]
    fn parse_not_compare() {
        let expr = parse("!a == b").unwrap();
        assert_eq!(
            expr,
            Expr::Compare(
                CompareOp::Eq,
                Box::new(Expr::Not(Box::new(Expr::Variable("a".into())))),
                Box::new(Expr::Variable("b".into()))
            )
        );
    }

    #[test]
    fn parse_parens() {
        let expr = parse("(a)").unwrap();
        assert_eq!(expr, Expr::Variable("a".into()));
    }

    #[test]
    fn parse_parens_override_precedence() {
        let expr = parse("(a || b) && c").unwrap();
        assert_eq!(
            expr,
            Expr::Logical(
                LogicalOp::And,
                Box::new(Expr::Logical(
                    LogicalOp::Or,
                    Box::new(Expr::Variable("a".into())),
                    Box::new(Expr::Variable("b".into()))
                )),
                Box::new(Expr::Variable("c".into()))
            )
        );
    }

    #[test]
    fn parse_complex_expression() {
        let expr = parse("github.ref == 'refs/heads/main' && success()").unwrap();
        assert_eq!(
            expr,
            Expr::Logical(
                LogicalOp::And,
                Box::new(Expr::Compare(
                    CompareOp::Eq,
                    Box::new(Expr::PropertyAccess(
                        Box::new(Expr::Variable("github".into())),
                        "ref".into()
                    )),
                    Box::new(Expr::Literal(Literal::String("refs/heads/main".into())))
                )),
                Box::new(Expr::FuncCall("success".into(), vec![]))
            )
        );
    }

    #[test]
    fn parse_chained_postfix() {
        let expr = parse("foo.bar[0].baz").unwrap();
        assert_eq!(
            expr,
            Expr::PropertyAccess(
                Box::new(Expr::IndexAccess(
                    Box::new(Expr::PropertyAccess(
                        Box::new(Expr::Variable("foo".into())),
                        "bar".into()
                    )),
                    Box::new(Expr::Literal(Literal::Int(0)))
                )),
                "baz".into()
            )
        );
    }

    #[test]
    fn parse_error_empty() {
        let result = parse("");
        assert!(result.is_err());
    }

    #[test]
    fn parse_error_trailing_tokens() {
        let result = parse("a b");
        assert!(result.is_err());
    }

    #[test]
    fn parse_error_unclosed_paren() {
        let result = parse("(a");
        assert!(result.is_err());
    }

    #[test]
    fn parse_error_unclosed_bracket() {
        let result = parse("a[0");
        assert!(result.is_err());
    }
}
