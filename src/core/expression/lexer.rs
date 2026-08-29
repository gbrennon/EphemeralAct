use super::{LexerError, token::Token};

/// A hand-written lexer for GitHub Actions `${{ }}` expression syntax.
///
/// Tokenizes the input stream one token at a time. Supports single-character
/// lookahead via [`peek_token`](Lexer::peek_token).
pub struct Lexer<'a> {
    /// Remaining characters to tokenize.
    chars: &'a str,
    /// Current byte position in the original input.
    pos: usize,
    /// Buffered token from a previous peek, if any.
    peeked: Option<Token>,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given input string.
    ///
    /// The lexer borrows the input; no copying is performed.
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input,
            pos: 0,
            peeked: None,
        }
    }

    /// Returns the next token without consuming it.
    ///
    /// Subsequent calls to `peek_token` return the same token. The token is
    /// only consumed when [`next_token`](Lexer::next_token) is called.
    ///
    /// # Errors
    ///
    /// Returns [`LexerError`] if the next characters form an invalid token.
    pub fn peek_token(&mut self) -> Result<Token, LexerError> {
        if let Some(ref token) = self.peeked {
            return Ok(token.clone());
        }
        let token = self.advance()?;
        self.peeked = Some(token.clone());
        Ok(token)
    }

    /// Consumes and returns the next token from the input stream.
    ///
    /// Returns [`Token::Eof`] once the entire input has been consumed.
    ///
    /// # Errors
    ///
    /// Returns [`LexerError`] if the next characters form an invalid token.
    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        if let Some(token) = self.peeked.take() {
            return Ok(token);
        }
        self.advance()
    }

    /// Core tokenization logic. Skips whitespace, then dispatches on the
    /// current character to produce the next token.
    fn advance(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace();

        let Some(ch) = self.current_char() else {
            return Ok(Token::Eof);
        };

        match ch {
            '\'' => self.lex_string(),
            '.' => {
                self.bump();
                Ok(Token::Dot)
            }
            '[' => {
                self.bump();
                Ok(Token::LBracket)
            }
            ']' => {
                self.bump();
                Ok(Token::RBracket)
            }
            '(' => {
                self.bump();
                Ok(Token::LParen)
            }
            ')' => {
                self.bump();
                Ok(Token::RParen)
            }
            '*' => {
                self.bump();
                Ok(Token::Star)
            }
            ',' => {
                self.bump();
                Ok(Token::Comma)
            }
            '!' => {
                self.bump();
                if self.current_char() == Some('=') {
                    self.bump();
                    Ok(Token::Neq)
                } else {
                    Ok(Token::Not)
                }
            }
            '=' => {
                self.bump();
                if self.current_char() == Some('=') {
                    self.bump();
                    Ok(Token::Eq)
                } else {
                    Err(LexerError::UnexpectedChar('=', self.pos - 1))
                }
            }
            '<' => {
                self.bump();
                if self.current_char() == Some('=') {
                    self.bump();
                    Ok(Token::Lte)
                } else {
                    Ok(Token::Lt)
                }
            }
            '>' => {
                self.bump();
                if self.current_char() == Some('=') {
                    self.bump();
                    Ok(Token::Gte)
                } else {
                    Ok(Token::Gt)
                }
            }
            '&' => {
                self.bump();
                if self.current_char() == Some('&') {
                    self.bump();
                    Ok(Token::And)
                } else {
                    Err(LexerError::UnexpectedChar('&', self.pos - 1))
                }
            }
            '|' => {
                self.bump();
                if self.current_char() == Some('|') {
                    self.bump();
                    Ok(Token::Or)
                } else {
                    Err(LexerError::UnexpectedChar('|', self.pos - 1))
                }
            }
            c if c.is_ascii_digit()
                || (c == '-' && self.peek_next_char().is_some_and(|n| n.is_ascii_digit())) =>
            {
                self.lex_number()
            }
            c if c.is_ascii_alphabetic() || c == '_' => self.lex_ident_or_keyword(),
            other => Err(LexerError::UnexpectedChar(other, self.pos)),
        }
    }

    /// Lexes a single-quoted string literal.
    ///
    /// Supports `''` as an escape sequence for a literal single quote within
    /// the string.
    fn lex_string(&mut self) -> Result<Token, LexerError> {
        let start = self.pos;
        self.bump();

        let mut value = String::new();

        loop {
            match self.current_char() {
                None => return Err(LexerError::UnterminatedString(start)),
                Some('\'') => {
                    self.bump();
                    if self.current_char() == Some('\'') {
                        self.bump();
                        value.push('\'');
                    } else {
                        return Ok(Token::String(value));
                    }
                }
                Some(ch) => {
                    value.push(ch);
                    self.bump();
                }
            }
        }
    }

    /// Lexes a numeric literal: integer or float.
    ///
    /// Handles an optional leading `-` for negative numbers. A decimal point
    /// followed by at least one digit produces a [`Token::Float`]; otherwise
    /// the result is a [`Token::Int`].
    fn lex_number(&mut self) -> Result<Token, LexerError> {
        let mut num_str = String::new();
        let mut is_float = false;

        if self.current_char() == Some('-') {
            num_str.push('-');
            self.bump();
        }

        while self.current_char().is_some_and(|c| c.is_ascii_digit()) {
            num_str.push(self.current_char().unwrap());
            self.bump();
        }

        if self.current_char() == Some('.')
            && self.peek_next_char().is_some_and(|c| c.is_ascii_digit())
        {
            is_float = true;
            num_str.push('.');
            self.bump();
            while self.current_char().is_some_and(|c| c.is_ascii_digit()) {
                num_str.push(self.current_char().unwrap());
                self.bump();
            }
        }

        if is_float {
            let value: f64 = num_str
                .parse()
                .expect("lex_number produced an unparseable float");
            Ok(Token::Float(value))
        } else {
            let value: i64 = num_str
                .parse()
                .expect("lex_number produced an unparseable int");
            Ok(Token::Int(value))
        }
    }

    /// Lexes an identifier or keyword (`true`, `false`, `null`).
    ///
    /// Identifiers match `[a-zA-Z_][a-zA-Z0-9_-]*`. If the lexeme matches a
    /// reserved keyword, the corresponding keyword token is returned.
    fn lex_ident_or_keyword(&mut self) -> Result<Token, LexerError> {
        let mut ident = String::new();

        ident.push(self.current_char().unwrap());
        self.bump();

        while self
            .current_char()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            ident.push(self.current_char().unwrap());
            self.bump();
        }

        match ident.as_str() {
            "true" => Ok(Token::Bool(true)),
            "false" => Ok(Token::Bool(false)),
            "null" => Ok(Token::Null),
            _ => Ok(Token::Ident(ident)),
        }
    }

    /// Advances past any whitespace characters.
    fn skip_whitespace(&mut self) {
        while self.current_char().is_some_and(|c| c.is_ascii_whitespace()) {
            self.bump();
        }
    }

    /// Returns the current character without consuming it.
    fn current_char(&self) -> Option<char> {
        self.chars.chars().next()
    }

    /// Returns the character after the current one without consuming anything.
    fn peek_next_char(&self) -> Option<char> {
        let mut iter = self.chars.chars();
        iter.next();
        iter.next()
    }

    /// Consumes the current character and advances the position.
    fn bump(&mut self) {
        if let Some(ch) = self.chars.chars().next() {
            self.chars = &self.chars[ch.len_utf8()..];
            self.pos += ch.len_utf8();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: collect all tokens from input into a Vec.
    fn lex_all(input: &str) -> Result<Vec<Token>, LexerError> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_token()?;
            let done = matches!(token, Token::Eof);
            tokens.push(token);
            if done {
                break;
            }
        }
        Ok(tokens)
    }

    #[test]
    fn lex_dot() {
        let tokens = lex_all(".").unwrap();
        assert_eq!(tokens, vec![Token::Dot, Token::Eof]);
    }

    #[test]
    fn lex_brackets() {
        let tokens = lex_all("[]").unwrap();
        assert_eq!(tokens, vec![Token::LBracket, Token::RBracket, Token::Eof]);
    }

    #[test]
    fn lex_parens() {
        let tokens = lex_all("()").unwrap();
        assert_eq!(tokens, vec![Token::LParen, Token::RParen, Token::Eof]);
    }

    #[test]
    fn lex_star() {
        let tokens = lex_all("*").unwrap();
        assert_eq!(tokens, vec![Token::Star, Token::Eof]);
    }

    #[test]
    fn lex_comma() {
        let tokens = lex_all(",").unwrap();
        assert_eq!(tokens, vec![Token::Comma, Token::Eof]);
    }

    #[test]
    fn lex_not() {
        let tokens = lex_all("!").unwrap();
        assert_eq!(tokens, vec![Token::Not, Token::Eof]);
    }

    #[test]
    fn lex_eq() {
        let tokens = lex_all("==").unwrap();
        assert_eq!(tokens, vec![Token::Eq, Token::Eof]);
    }

    #[test]
    fn lex_neq() {
        let tokens = lex_all("!=").unwrap();
        assert_eq!(tokens, vec![Token::Neq, Token::Eof]);
    }

    #[test]
    fn lex_lt() {
        let tokens = lex_all("<").unwrap();
        assert_eq!(tokens, vec![Token::Lt, Token::Eof]);
    }

    #[test]
    fn lex_lte() {
        let tokens = lex_all("<=").unwrap();
        assert_eq!(tokens, vec![Token::Lte, Token::Eof]);
    }

    #[test]
    fn lex_gt() {
        let tokens = lex_all(">").unwrap();
        assert_eq!(tokens, vec![Token::Gt, Token::Eof]);
    }

    #[test]
    fn lex_gte() {
        let tokens = lex_all(">=").unwrap();
        assert_eq!(tokens, vec![Token::Gte, Token::Eof]);
    }

    #[test]
    fn lex_and() {
        let tokens = lex_all("&&").unwrap();
        assert_eq!(tokens, vec![Token::And, Token::Eof]);
    }

    #[test]
    fn lex_or() {
        let tokens = lex_all("||").unwrap();
        assert_eq!(tokens, vec![Token::Or, Token::Eof]);
    }

    #[test]
    fn lex_simple_string() {
        let tokens = lex_all("'hello'").unwrap();
        assert_eq!(tokens, vec![Token::String("hello".into()), Token::Eof]);
    }

    #[test]
    fn lex_empty_string() {
        let tokens = lex_all("''").unwrap();
        assert_eq!(tokens, vec![Token::String(String::new()), Token::Eof]);
    }

    #[test]
    fn lex_string_with_escaped_quote() {
        let tokens = lex_all("'it''s'").unwrap();
        assert_eq!(tokens, vec![Token::String("it's".into()), Token::Eof]);
    }

    #[test]
    fn lex_string_with_multiple_escapes() {
        let tokens = lex_all("'a''b''c'").unwrap();
        assert_eq!(tokens, vec![Token::String("a'b'c".into()), Token::Eof]);
    }

    #[test]
    fn lex_unterminated_string_error() {
        let err = lex_all("'no end").unwrap_err();
        assert_eq!(err, LexerError::UnterminatedString(0));
    }

    #[test]
    fn lex_positive_int() {
        let tokens = lex_all("42").unwrap();
        assert_eq!(tokens, vec![Token::Int(42), Token::Eof]);
    }

    #[test]
    fn lex_negative_int() {
        let tokens = lex_all("-7").unwrap();
        assert_eq!(tokens, vec![Token::Int(-7), Token::Eof]);
    }

    #[test]
    fn lex_zero() {
        let tokens = lex_all("0").unwrap();
        assert_eq!(tokens, vec![Token::Int(0), Token::Eof]);
    }

    #[test]
    fn lex_float() {
        let tokens = lex_all("2.71").unwrap();
        assert_eq!(tokens, vec![Token::Float(2.71), Token::Eof]);
    }

    #[test]
    fn lex_negative_float() {
        let tokens = lex_all("-0.5").unwrap();
        assert_eq!(tokens, vec![Token::Float(-0.5), Token::Eof]);
    }

    #[test]
    fn lex_float_with_trailing_dot_is_int() {
        let tokens = lex_all("42.").unwrap();
        assert_eq!(tokens, vec![Token::Int(42), Token::Dot, Token::Eof]);
    }

    #[test]
    fn lex_true() {
        let tokens = lex_all("true").unwrap();
        assert_eq!(tokens, vec![Token::Bool(true), Token::Eof]);
    }

    #[test]
    fn lex_false() {
        let tokens = lex_all("false").unwrap();
        assert_eq!(tokens, vec![Token::Bool(false), Token::Eof]);
    }

    #[test]
    fn lex_null() {
        let tokens = lex_all("null").unwrap();
        assert_eq!(tokens, vec![Token::Null, Token::Eof]);
    }

    #[test]
    fn lex_simple_ident() {
        let tokens = lex_all("github").unwrap();
        assert_eq!(tokens, vec![Token::Ident("github".into()), Token::Eof]);
    }

    #[test]
    fn lex_ident_with_underscore() {
        let tokens = lex_all("event_name").unwrap();
        assert_eq!(tokens, vec![Token::Ident("event_name".into()), Token::Eof]);
    }

    #[test]
    fn lex_ident_with_hyphen() {
        let tokens = lex_all("my-job").unwrap();
        assert_eq!(tokens, vec![Token::Ident("my-job".into()), Token::Eof]);
    }

    #[test]
    fn lex_ident_with_digits() {
        let tokens = lex_all("step1").unwrap();
        assert_eq!(tokens, vec![Token::Ident("step1".into()), Token::Eof]);
    }

    #[test]
    fn lex_ident_starting_with_underscore() {
        let tokens = lex_all("_private").unwrap();
        assert_eq!(tokens, vec![Token::Ident("_private".into()), Token::Eof]);
    }

    #[test]
    fn peek_does_not_consume() {
        let mut lexer = Lexer::new("foo");
        let first = lexer.peek_token().unwrap();
        let second = lexer.peek_token().unwrap();
        assert_eq!(first, Token::Ident("foo".into()));
        assert_eq!(second, Token::Ident("foo".into()));
    }

    #[test]
    fn peek_then_next_consumes() {
        let mut lexer = Lexer::new("foo bar");
        assert_eq!(lexer.peek_token().unwrap(), Token::Ident("foo".into()));
        assert_eq!(lexer.next_token().unwrap(), Token::Ident("foo".into()));
        assert_eq!(lexer.next_token().unwrap(), Token::Ident("bar".into()));
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn peek_after_next_returns_next_token() {
        let mut lexer = Lexer::new("a b");
        assert_eq!(lexer.next_token().unwrap(), Token::Ident("a".into()));
        assert_eq!(lexer.peek_token().unwrap(), Token::Ident("b".into()));
        assert_eq!(lexer.next_token().unwrap(), Token::Ident("b".into()));
    }

    #[test]
    fn lex_skips_whitespace() {
        let tokens = lex_all("  \t\n\r  foo  ").unwrap();
        assert_eq!(tokens, vec![Token::Ident("foo".into()), Token::Eof]);
    }

    #[test]
    fn lex_full_expression() {
        let tokens = lex_all("github.event_name == 'push' && !cancelled()").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("github".into()),
                Token::Dot,
                Token::Ident("event_name".into()),
                Token::Eq,
                Token::String("push".into()),
                Token::And,
                Token::Not,
                Token::Ident("cancelled".into()),
                Token::LParen,
                Token::RParen,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn lex_function_call_with_args() {
        let tokens = lex_all("contains('hello', 'll')").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("contains".into()),
                Token::LParen,
                Token::String("hello".into()),
                Token::Comma,
                Token::String("ll".into()),
                Token::RParen,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn lex_unexpected_char() {
        let err = lex_all("@").unwrap_err();
        assert_eq!(err, LexerError::UnexpectedChar('@', 0));
    }

    #[test]
    fn lex_lone_ampersand_error() {
        let err = lex_all("&").unwrap_err();
        assert_eq!(err, LexerError::UnexpectedChar('&', 0));
    }

    #[test]
    fn lex_lone_pipe_error() {
        let err = lex_all("|").unwrap_err();
        assert_eq!(err, LexerError::UnexpectedChar('|', 0));
    }

    #[test]
    fn lex_lone_equals_error() {
        let err = lex_all("=").unwrap_err();
        assert_eq!(err, LexerError::UnexpectedChar('=', 0));
    }
}
