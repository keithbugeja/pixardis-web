use crate::common::{
    logger::{
        Logger, 
        LoggerError
    }, 
    status::CompilationResult
};

use super::token::{Token, classify_token};

///
/// Span structure to keep track of the start and end of a token
/// 
#[derive(Debug, PartialEq, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

///
/// Symbols that can be classified by the lexer
/// 
#[derive(Debug, PartialEq)]
pub enum Symbol {
    Alpha(char),
    Digit(char),
    Period,
    Comma,
    Plus,
    Minus,
    Asterisk,
    Ampersand,
    Pipe,
    Slash,
    Bang,
    Equals,
    Colon,
    Semicolon,
    Underscore,
    Pound,
    LAngle,
    RAngle,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Whitespace,
    CR,
    EOL,
    EOF,
    Undefined,
}

///
/// Classify a symbol into a Symbol enum    
/// 
pub fn classify_symbol(symbol: char) -> Symbol {
    match symbol {
        'a'..='z' | 'A'..='Z' => Symbol::Alpha(symbol),
        '0'..='9' => Symbol::Digit(symbol),
        ' ' | '\t' => Symbol::Whitespace,
        '.' => Symbol::Period,
        ',' => Symbol::Comma,
        '+' => Symbol::Plus,
        '-' => Symbol::Minus,
        '*' => Symbol::Asterisk,
        '&' => Symbol::Ampersand,
        '|' => Symbol::Pipe,
        '/' => Symbol::Slash,
        '!' => Symbol::Bang,
        ':' => Symbol::Colon,        
        ';' => Symbol::Semicolon,
        '_' => Symbol::Underscore,
        '<' => Symbol::LAngle,
        '>' => Symbol::RAngle,
        '=' => Symbol::Equals,
        '#' => Symbol::Pound,
        '(' => Symbol::LParen,
        ')' => Symbol::RParen,
        '{' => Symbol::LBrace,
        '}' => Symbol::RBrace,
        '\n' => Symbol::EOL,
        '\r' => Symbol::CR,
        _ => Symbol::Undefined    
    }
}

///
/// Lexer structure to tokenize the input string
/// 
#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    token_index: usize,
    tokens: Vec<Token>,
    newlines: Vec<usize>,
    logger: &'a mut Logger<'a>,
    status: CompilationResult,
}

///
/// Lexer implementation
/// 
impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, logger: &'a mut Logger<'a>) -> Self {
        let mut lexer = Lexer {
            input, 
            position: 0,
            token_index: 0,
            tokens: vec![],
            newlines: vec![],
            logger,
            status: CompilationResult::Pending,
        };

        lexer.enumerate_newlines();

        lexer
    }

    fn status_set(&mut self, status: CompilationResult) {
        self.status = status;
    }

    pub fn status(&self) -> CompilationResult {
        self.status.clone()
    }

    fn enumerate_newlines(&mut self) {
        for (line_end, _) in self.input.match_indices("\n") {
            self.newlines.push(line_end);
        }
    }

    fn token_position_to_line_number(&self, position: usize) -> usize {
        if let Some(line_number) = self.newlines.iter().position(|&p| p >= position) {
            return line_number;
        }

        return 0;
    }

    fn next(&mut self) {
        self.position += 1;
    }

    fn peek(&mut self) -> Symbol {
        if self.position >= self.input.len() {
            return Symbol::EOF;
        }

        let mut sym = Symbol::Undefined;

        if let Some(chr_as_str) = self.input.get(self.position..self.position+1) {
            if let Some(chr) = chr_as_str.chars().next() {
                sym = classify_symbol(chr);
            }
        }

        return sym;
    }

    fn peek_char(&mut self) -> Option<char> {
        if self.position >= self.input.len() {
            return None;
        }

        if let Some(chr_as_str) = self.input.get(self.position..self.position+1) {
            return chr_as_str.chars().next();
        }

        return None;
    }

    fn get_identifier_char(&mut self, symbol: Symbol) -> Option<char> {
        match symbol 
        {
            Symbol::Alpha(value) => Some(value),
            Symbol::Digit(value) => Some(value),
            Symbol::Underscore => Some('_'),
            _ => None
        }
    }

    fn scan_identifier(&mut self) -> String {
        let mut identkey = String::new();
        let mut symbol = self.peek();
        while let Some(ch) = self.get_identifier_char(symbol)  {
            identkey.push(ch);

            self.next();

            symbol = self.peek();
        }

        return identkey;
    }

    fn get_number_char(&mut self, symbol: Symbol) -> Option<char> {
        match symbol
        {
            Symbol::Digit(value) => Some(value),
            Symbol::Period => Some('.'),
            _ => None
        }
    }

    fn scan_number(&mut self) -> String {
        let mut number = String::new();
        let mut symbol = self.peek();
        let mut period = false;
        
        while let Some(ch) = self.get_number_char(symbol) 
        {
            number.push(ch);

            self.next();

            symbol = self.peek();
            
            if symbol == Symbol::Period {
                if period {
                    panic!("Malformed numeric literal!");
                } else {
                    period = true;
                }
            }
        }

        // println!("Number: {}", number);

        return number;
    }

    fn get_colour_char(&mut self, symbol: Symbol) -> Option<char> {
        match symbol
        {
            Symbol::Pound => Some('#'),
            Symbol::Digit(value) => Some(value),
            Symbol::Alpha(value) => {
                match value {
                    'a'..='f' | 'A'..='F' => Some(value),
                    _ => None
                }
            },
            _ => None
        }
    }

    fn scan_colour(&mut self) -> String {
        let mut colour: String = String::new();
        let mut symbol: Symbol = self.peek();
        while let Some(ch) = self.get_colour_char(symbol) {
            colour.push(ch);

            self.next();

            symbol = self.peek();
        }

        return colour;
    }

    fn scan_character(&mut self) -> String {
        let mut result = String::new();
        
        if let Some(chr) = self.peek_char() 
        { 
            result = String::from(chr);
            self.next();
        }

        return result;
    }

    fn scan_operator(&mut self) -> String {
        let mut operator = String::new();        
        let symbol_left = self.peek();
        
        self.next(); 
        
        let symbol_right = self.peek();

        match symbol_left {
            Symbol::Plus => {
                operator = String::from("+");
            }
            Symbol::Minus => {
                match symbol_right {
                    Symbol::RAngle => {
                        self.next();
                        operator = String::from("->");
                    },
                    _ => { operator = String::from("-"); }
                }
            },
            Symbol::Asterisk => {
                operator = String::from("*");
            },
            Symbol::Slash => {
                operator = String::from("/");
            },
            Symbol::LAngle => {
                match symbol_right {
                    Symbol::Equals => {
                        self.next();
                        operator = String::from("<=");
                    },
                    _ => { operator = String::from("<"); }
                }
            },
            Symbol::RAngle => {
                match symbol_right {
                    Symbol::Equals => {
                        self.next();
                        operator = String::from(">=");
                    },
                    _ => { operator = String::from(">"); }
                }
            },
            Symbol::Bang => {
                match symbol_right {
                    Symbol::Equals => {
                        self.next();
                        operator = String::from("!=");
                    },
                    _ => { operator = String::from("!"); }
                }
            },
            Symbol::Equals => {
                match symbol_right {
                    Symbol::Equals => {
                        self.next();
                        operator = String::from("==");
                    },
                    _ => { operator = String::from("="); }
                }
            },
            Symbol::Ampersand => {
                match symbol_right {
                    Symbol::Ampersand => {
                        self.next();
                        operator = String::from("&&");
                    },
                    _ => { operator = String::from("&"); }
                }
            },
            Symbol::Pipe => {
                match symbol_right {
                    Symbol::Pipe => {
                        self.next();
                        operator = String::from("||");
                    },
                    _ => { operator = String::from("|"); }
                }
            }
            _ => { }
        }

        // println!("Operator: {}", operator);

        return operator;
    }

    fn scan_comment(&mut self) {
        let comment_type = self.peek();

        match comment_type {
            // Line comment
            Symbol::Slash => {
                self.next();

                loop 
                {
                    let symbol = self.peek();

                    self.next();

                    if symbol == Symbol::EOL || symbol == Symbol::EOF {
                        break;
                    }
                }
            },

            Symbol::Asterisk => {
                self.next();

                loop 
                {
                    let symbol = self.peek();

                    self.next();

                    if symbol == Symbol::Asterisk
                    {
                        let symbol_right = self.peek();

                        self.next();

                        if symbol_right == Symbol::Slash {
                            break;
                        }
                    }
                }
            },

            _ => { },
        }
    }

    pub fn scan(&mut self) {
        // Set success flag (this will be cleared if any errors are encountered)
        self.status_set(CompilationResult::Success);
        
        let mut symbol = Symbol::Undefined;
        let mut symbol_position ;

        while symbol != Symbol::EOF 
        {
            symbol = self.peek();
            symbol_position = self.position;

            match symbol
            {
                // whitespace is ignored unless within quoted literal
                Symbol::Whitespace => self.next(),
                
                // slash may start a line or block comment
                Symbol::Slash => {
                    self.next();

                    let symbol_right = self.peek();

                    match symbol_right {
                        Symbol::Slash | Symbol::Asterisk => {
                            self.scan_comment();
                        },
                        _ => {
                            let token_input: String = String::from("/");
                            let token = classify_token(&token_input);
                            let line_number = self.token_position_to_line_number(symbol_position); 
                            self.tokens.push(Token { 
                                kind: token,
                                span: Span { 
                                    start: symbol_position, 
                                    end: self.position,
                                },
                                line: line_number,
                            }); 
                        },
                    }
                },

                // identifier
                Symbol::Underscore | Symbol::Alpha(_) => {
                    let token_input = self.scan_identifier();
                    let token = classify_token(&token_input);
                    let line_number = self.token_position_to_line_number(symbol_position); 
                    self.tokens.push(Token { 
                        kind: token,
                        span: Span { 
                            start: symbol_position, 
                            end: self.position,
                        },
                        line: line_number,
                    });
                },

                // integer or float literal
                Symbol::Digit(_) => {
                    let token_input = self.scan_number();
                    let token = classify_token(&token_input);
                    let line_number = self.token_position_to_line_number(symbol_position); 
                    self.tokens.push(Token { 
                        kind: token,
                        span: Span { 
                            start: symbol_position, 
                            end: self.position,
                        },
                        line: line_number,
                    });
                },

                // colour literal
                Symbol::Pound => {
                    let token_input = self.scan_colour();
                    let token = classify_token(&token_input);
                    let line_number = self.token_position_to_line_number(symbol_position); 
                    self.tokens.push(Token { 
                        kind: token,
                        span: Span { 
                            start: symbol_position, 
                            end: self.position,
                        },
                        line: line_number,
                    });
                },

                // delimiters and punctuation
                Symbol::LParen | Symbol::RParen | Symbol::LBrace | Symbol::RBrace | Symbol::Comma | Symbol::Colon | Symbol::Semicolon => {
                    let token_input: String = self.scan_character();
                    let token = classify_token(&token_input);
                    let line_number = self.token_position_to_line_number(symbol_position); 
                    self.tokens.push(Token { 
                        kind: token,
                        span: Span { 
                            start: symbol_position, 
                            end: self.position,
                        },
                        line: line_number,
                    });
                },

                // operators
                Symbol::Equals | Symbol::Bang | Symbol::LAngle | Symbol::RAngle | Symbol::Asterisk | Symbol::Plus | Symbol::Minus | Symbol::Ampersand | Symbol::Pipe => {
                    let token_input: String = self.scan_operator();
                    let token = classify_token(&token_input);
                    let line_number = self.token_position_to_line_number(symbol_position); 
                    self.tokens.push(Token { 
                        kind: token,
                        span: Span { 
                            start: symbol_position, 
                            end: self.position,
                        },
                        line: line_number,
                    });
                },

                // End of line or file
                Symbol::EOL | Symbol::EOF | Symbol::CR => {
                    self.next();
                },

                // unrecognised
                _ => { 
                    self.logger.print_error(
                        LoggerError::Lexical, 
                        format!("Skipping unidentified token {:?}", self.input.chars().nth(symbol_position).unwrap()).as_str(),
                        self.token_position_to_line_number(symbol_position));

                        self.status_set(CompilationResult::Warning);
                    self.next()
                },
            }
        }

        self.token_index = 0;
    }

    pub fn peek_token(&self) -> Option<&Token> {
        self.peek_k_tokens(0)
    }

    pub fn peek_k_tokens(&self, k: usize) -> Option<&Token> {
        if self.token_index + k < self.tokens.len()  {
            return Some(&self.tokens[self.token_index + k])
        }

        return None;
    }

    pub fn next_token(&mut self) -> Option<&Token> {
        let mut token:Option<&Token> = None;

        if self.token_index < self.tokens.len()  {
            
            token = Some(&self.tokens[self.token_index])
        }

        self.token_index += 1;

        return token;        
    }
}