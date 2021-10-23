use logos::Logos;


fn convert_string(s: &str) -> String {
    s
    
        .replace("\r", "")
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\\"", "\"")
        .replace("\\'", "'")
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    #[regex(r"([0-9]+(\.[0-9]*)?|\.[0-9]*)", |lex| lex.slice().parse::<f64>())]
    Number(f64),

    #[regex(r#""(?:\\.|[^\\"])*"|'(?:\\.|[^\\'])*'"#, 
        |s| convert_string(&s.slice()[1..s.slice().len()-1])
    )]
    StringLiteral(String),

    
    #[regex(r"#[a-zA-Z_][a-zA-Z_0-9]*", |lex| lex.slice()[1..].to_string())]
    TypeName(String),

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
    
    #[token("..")]
    Range,

    #[token(".")]
    Dot,

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
    #[token("as")]
    As,

    #[regex(r"[a-zA-Z_ඞ][a-zA-Z_0-9ඞ]*", |lex| lex.slice().to_string())]
    Identifier(String),

    #[error]
    #[regex(r"[ \t\f]+", logos::skip)]
    Error,

    Eof,
}


