/// Token types produced by the expression lexer.
///
/// Represents all terminal symbols in the GitHub Actions `${{ }}` expression
/// language: literals, identifiers, operators, punctuation, and end-of-file.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// An identifier: `[a-zA-Z_][a-zA-Z0-9_-]*` (excluding keywords).
    Ident(String),
    /// A single-quoted string literal with `''` escape for literal `'`.
    String(String),
    /// A signed 64-bit integer literal.
    Int(i64),
    /// A 64-bit floating-point literal (must contain a decimal point).
    Float(f64),
    /// A boolean literal: `true` or `false`.
    Bool(bool),
    /// The `null` literal.
    Null,
    /// `.` - property access operator.
    Dot,
    /// `[` - index/open bracket.
    LBracket,
    /// `]` - close bracket.
    RBracket,
    /// `(` - open parenthesis.
    LParen,
    /// `)` - close parenthesis.
    RParen,
    /// `!` - logical NOT.
    Not,
    /// `==` - equality comparison.
    Eq,
    /// `!=` - inequality comparison.
    Neq,
    /// `<` - less-than comparison.
    Lt,
    /// `<=` - less-than-or-equal comparison.
    Lte,
    /// `>` - greater-than comparison.
    Gt,
    /// `>=` - greater-than-or-equal comparison.
    Gte,
    /// `&&` - logical AND.
    And,
    /// `||` - logical OR.
    Or,
    /// `*` - array dereference / wildcard.
    Star,
    /// `,` - argument separator.
    Comma,
    /// End of input stream.
    Eof,
}
