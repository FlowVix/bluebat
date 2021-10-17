use logos::Logos;



#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    #[regex(r"([0-9]+(\.[0-9]*)?|\.[0-9]*)", |lex| lex.slice().parse::<f64>())]
    Number(f64),

    #[token("+=")]
    PlusEq,

    #[token("-=")]
    MinusEq,

    #[token("*=")]
    MultEq,

    #[token("/=")]
    DivEq,

    #[token("%=")]
    ModEq,

    #[token("^=")]
    PowEq,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Mult,

    #[token("/")]
    Div,

    #[token("%")]
    Mod,

    #[token("^")]
    Pow,


    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("<=")]
    LesserEq,

    #[token(">=")]
    GreaterEq,

    #[token("<")]
    Lesser,

    #[token(">")]
    Greater,
    
    #[token("!=")]
    NotEq,

    #[token("!")]
    Not,

    #[token("{")]
    LBracket,

    #[token("}")]
    RBracket,

    #[token("[")]
    LSqBracket,

    #[token("]")]
    RSqBracket,

    #[regex(r";|\n|\r")]
    Eol,

    #[regex(",")]
    Comma,

    #[token(":=")]
    LocalAssign,

    #[token(":")]
    Colon,

    #[token("==")]
    Eq,

    #[token("=")]
    Assign,

    #[token("||")]
    Or,

    #[token("|")]
    Pipe,

    #[token("&&")]
    And,

    #[token("True")]
    True,
    #[token("False")]
    False,
    #[token("Null")]
    Null,
    #[token("if")]
    If,
    #[token("elif")]
    Elif,
    #[token("else")]
    Else,
    #[token("while")]
    While,

    #[regex(r"[a-zA-Z_][a-zA-Z_0-9]*", |lex| lex.slice().to_string())]
    Identifier(String),

    #[error]
    #[regex(r"[ \t\f]+", logos::skip)]
    Error,

    Eof,
}


