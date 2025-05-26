use super::lexer::Span;

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub line: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Identifier(String),
    Type(String),
    ArrayType(String, isize),
    BooleanLiteral(bool),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    ColourLiteral(String),
    RandomInt,
    Width,
    Height,
    Read,
    UnaryOp,
    MultiplicativeOp(String),
    AdditiveOp(String),
    RelationalOp(String),
    Equals,
    Let,
    Print,
    Clear,
    Delay,
    WriteLine,
    WriteBox,
    Write,
    Return,
    As,
    If,
    Else,
    For,
    While,
    Fun,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    SemiColon,
    OpenBracket,
    CloseBracket,
    Colon,
    Comma,
    Arrow,
}

pub fn classify_token(s: &str) -> TokenKind {
    match s {
        "float" | "int" | "bool" | "colour" => TokenKind::Type(s.to_string()),
        "true" => TokenKind::BooleanLiteral(true),
        "false" => TokenKind::BooleanLiteral(false),
        "__width" => TokenKind::Width,
        "__height" => TokenKind::Height,
        "__read" => TokenKind::Read,
        "__random_int" | "__randi" => TokenKind::RandomInt,
        "__print" => TokenKind::Print,
        "__clear" => TokenKind::Clear,
        "__delay" => TokenKind::Delay,
        "__write_box" | "__pixelr" => TokenKind::WriteBox,
        "__write_line" | "__pixell" => TokenKind::WriteLine,
        "__write" | "__pixel" => TokenKind::Write,
        "return" => TokenKind::Return,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "for" => TokenKind::For,
        "while" => TokenKind::While,
        "fun" => TokenKind::Fun,
        "let" => TokenKind::Let,
        "as" => TokenKind::As,
        "->" => TokenKind::Arrow,
        "," => TokenKind::Comma,
        ":" => TokenKind::Colon,
        ";" => TokenKind::SemiColon,
        "{" => TokenKind::OpenBrace,
        "}" => TokenKind::CloseBrace,
        "(" => TokenKind::OpenParen,
        ")" => TokenKind::CloseParen,
        "[" => TokenKind::OpenBracket,
        "]" => TokenKind::CloseBracket,
        "=" => TokenKind::Equals,
        "not" => TokenKind::UnaryOp,
        "+" | "-" | "|" => TokenKind::AdditiveOp(s.to_string()),
        "*" | "/" | "&" | "%" => TokenKind::MultiplicativeOp(s.to_string()),
        "==" | "<" | ">" | ">=" | "<=" | "!=" | "&&" | "and" | "||" | "or" => TokenKind::RelationalOp(s.to_string()),
        _ => { // identifier or literal
            // literal type (int, float, colour)
            match s.chars().next().unwrap() {
                '0'..='9' => { 
                    match s.parse::<i64>() {
                        Ok(i) => TokenKind::IntegerLiteral(i),
                        Err(_) => {
                            match s.parse::<f64>() {
                                Ok(f) => TokenKind::FloatLiteral(f),
                                Err(_) => TokenKind::Identifier(s.to_string()),
                            }
                        }
                    }
                },
                '#' => TokenKind::ColourLiteral(s.to_string()),
                _ => TokenKind::Identifier(s.to_string()),
            }
        }
    }
}