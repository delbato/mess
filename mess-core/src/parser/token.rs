use logos::Logos;

/// Token enum
#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    /// Fn decl token
    #[token("fun")]
    Fun,

    #[token("mod")]
    Mod,

    /// var decl token
    #[token("var")]
    Var,

    #[token("static")]
    Static,

    #[token("cont")]
    Cont,

    #[token("ext")]
    Ext,

    #[token("pub")]
    Pub,

    #[token("intf")]
    Intf,

    #[token("enum")]
    Enum,

    #[token("as")]
    As,

    #[token("for")]
    For,

    #[token("in")]
    In,

    #[token("on")]
    On,

    #[token("yield")]
    Yield,

    #[token("import")]
    Import,

    #[token("this")]
    This,

    #[token("&this")]
    ThisRef,

    #[token("else")]
    Else,

    #[token("while")]
    While,

    #[token("break")]
    Break,

    #[token("continue")]
    Continue,

    #[token("return")]
    Return,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Times,

    #[token("/")]
    Divide,

    #[token("=")]
    Assign,

    #[regex("int|bool|float|string")]
    PrimitiveType,

    #[token("<")]
    LessThan,

    #[token(">")]
    GreaterThan,

    #[token("<=")]
    LessThanEquals,

    #[token(">=")]
    GreaterThanEquals,

    #[token("==")]
    Equals,

    #[token("!=")]
    NotEquals,

    #[token("+=")]
    AddAssign,

    #[token("-=")]
    SubAssign,

    #[token("*=")]
    MulAssign,

    #[token("/=")]
    DivAssign,

    #[token(";")]
    Semicolon,

    #[token(":")]
    Colon,

    #[token("::")]
    DoubleColon,

    #[token(".")]
    Dot,

    #[token("..")]
    DoubleDot,

    #[token(",")]
    Comma,

    #[token("&")]
    Ref,

    #[token("~")]
    Tilde,

    #[token("(")]
    OpenParan,

    #[token(")")]
    CloseParan,

    #[token("{")]
    OpenBlock,

    #[token("}")]
    CloseBlock,

    #[regex(r#""([^"\\]|\\[\s\S])*""#)]
    StringLiteral,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex(r"[0-9]+")]
    IntLiteral,

    #[regex(r"[0-9]+\.[0-9]+")]
    FloatLiteral,

    #[regex("true|false")]
    BoolLiteral,

    #[regex("#.*\n", logos::skip)]
    HashLineComment,

    #[regex(r"\s+", logos::skip)]
    Whitespace,

    /// Error token, produced on a lexer error
    #[error]
    Error,
}
