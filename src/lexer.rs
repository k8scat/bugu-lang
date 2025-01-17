use std::collections::HashMap;
// use std::fs::File;
// use std::io;
// use std::io::prelude::*;
use std::rc::Rc;

// Token type
#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Num(i32),      // Number literal
    Ident(String), // Identifier
    Int,           // "int"
    Semicolon,     // ;
    LeftParen,     // (
    RightParen,    // )
    LeftBrace,     // {
    RightBrace,    // }
    Equal,         // =
    Return,        // "return"
    Neg,           // -
    Add,           // +
    Mul,           // *
    Div,            // /
    Mod,            // %
    Lt,             // <
    Gt,             // >
    // Eq,             // =
    Not,            // !
    And,            // &
    Or,             // |
    If,             // if
    Else,             // else
    Colon,          // :
    Ques,           // ?
    For,
    While,
    Continue,
    Break,
    Eof,
    Comma,          // ,
    LeftBrack,      // [
    RightBrack,     // ]
}

// Character Kind
#[derive(Debug, PartialEq)]
pub enum CharacterType {
    Whitespace, // ' '
    NewLine,    // ' \n'
    Alphabetic,
    Digit,
    NonAlphabetic(char),
    Unknown(char),
}

impl TokenType {
    fn new_single_letter(c: char) -> Option<Self> {
        use self::TokenType::*;
        match c {
            ';' => Some(Semicolon),
            '=' => Some(Equal),
            '(' => Some(LeftParen),
            ')' => Some(RightParen),
            '{' => Some(LeftBrace),
            '}' => Some(RightBrace),
            '-' => Some(Neg),
            '+' => Some(Add),
            '*' => Some(Mul),
            '/' => Some(Div),
            '%' => Some(Mod),
            '<' => Some(Lt),
            '>' => Some(Gt),
            '!' => Some(Not),
            '&' => Some(And),
            '|' => Some(Or),
            ':' => Some(Colon),
            '?' => Some(Ques),
            ',' => Some(Comma),
            '[' => Some(LeftBrack),
            ']' => Some(RightBrack),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Ctype {
    Int,
}

impl Default for Ctype {
    fn default() -> Ctype {
        Ctype::Int
    }
}

#[derive(Debug, Clone)]
pub struct Type {
    pub ty: Ctype,
    pub size: usize,  // sizeof
    pub align: usize, // alignof
}

impl Default for Type {
    fn default() -> Type {
        Type {
            ty: Ctype::default(),
            size: 4,// 只支持到 32 位，把它改成 64？
            align: 4,
        }
    }
}

pub fn tokenize(path: String) -> Vec<Token> {
    let mut tokenizer = Tokenizer::new(Rc::new(path));
    tokenizer.scan(&keyword_map());
    tokenizer.tokens
}

fn keyword_map() -> HashMap<String, TokenType> {
    let mut map = HashMap::new();
    map.insert("int".into(), TokenType::Int);
    map.insert("return".into(), TokenType::Return);
    map.insert("if".into(), TokenType::If);
    map.insert("else".into(), TokenType::Else);
    map.insert("for".into(), TokenType::For);
    map.insert("while".into(), TokenType::While);
    map.insert("continue".into(), TokenType::Continue);
    map.insert("break".into(), TokenType::Break);
    map
}

#[derive(Debug, Clone)]
pub struct Token {
    pub ty: TokenType, // Token type
}

impl Default for Token {
    fn default() -> Token {
        Token { ty: TokenType::Int }
    }
}

impl Token {
    pub fn new(ty: TokenType) -> Self {
        Token { ty }
    }

    pub fn is_ident(&self, s: &str) -> bool {
        match self.ty {
            TokenType::Ident(ref name) => name == s,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
struct Tokenizer {
    p: Rc<Vec<char>>, //input
    pos: usize,
    tokens: Vec<Token>,
}

impl Tokenizer {
    fn new(context: Rc<String>) -> Self {
        Tokenizer {
            p: Rc::new(context.chars().collect()),
            pos: 0,
            tokens: vec![],
        }
    }

    fn new_token(&self, ty: TokenType) -> Token {
        Token::new(ty)
    }

    // This does not support non-ASCII characters.
    fn get_character(&self, advance_from_pos: usize) -> Option<CharacterType> {
        self.p.get(self.pos + advance_from_pos).map(|ch| {
            if ch == &'\n' {
                CharacterType::NewLine
            } else if ch == &' ' || ch == &'\t' {
                CharacterType::Whitespace
            } else if ch.is_alphabetic() || ch == &'_' {
                CharacterType::Alphabetic
            } else if ch.is_digit(10) {
                CharacterType::Digit
            } else {
                CharacterType::NonAlphabetic(*ch)
            }
        })
    }

    fn scan(&mut self, keywords: &HashMap<String, TokenType>) -> Vec<Token> {
        while let Some(head_char) = self.get_character(0) {
            match head_char {
                CharacterType::NewLine | CharacterType::Whitespace => self.pos += 1,
                CharacterType::Alphabetic => self.ident(&keywords),
                CharacterType::Digit => self.number(),
                CharacterType::NonAlphabetic(c) => {
                    // Single-letter symbol
                    if let Some(ty) = TokenType::new_single_letter(c) {
                        let t = self.new_token(ty);
                        self.pos += 1;
                        self.tokens.push(t);
                        continue;
                    }
                    panic!("Unknwon character type. {}", c)
                }
                CharacterType::Unknown(_) => self.bad_position("Unknwon character type."),
            }
        }
        self.tokens.push(self.new_token(TokenType::Eof));
        self.tokens.clone()
    }

    fn ident(&mut self, keywords: &HashMap<String, TokenType>) {
        let mut len = 1;
        while let Some(c2) = self.p.get(self.pos + len) {
            if c2.is_alphabetic() || c2.is_ascii_digit() || c2 == &'_' {
                len += 1;
                continue;
            }
            break;
        }

        let name: String = self.p[self.pos..self.pos + len].iter().collect();
        let t;
        if let Some(keyword) = keywords.get(&name) {
            t = self.new_token(keyword.clone());
        } else {
            t = self.new_token(TokenType::Ident(name.clone()));
        }
        self.pos += len;
        self.tokens.push(t);
    }

    fn number(&mut self) {
        let base = 10;
        let mut sum: i32 = 0;
        let mut len = 0;
        for c in self.p[self.pos..].iter() {
            if let Some(val) = c.to_digit(base) {
                sum = sum * base as i32 + val as i32;
                len += 1;
            } else {
                break;
            }
        }
        let t = self.new_token(TokenType::Num(sum as i32));
        self.pos += len;
        self.tokens.push(t);
    }

    fn bad_position(&self, msg: &'static str) {
        panic!("{}", msg);
    }
}
