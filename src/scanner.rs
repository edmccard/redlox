use anyhow::bail;
use std::{fmt, str};

use crate::Result;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier,
    String,
    Number,
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    #[default]
    Eof,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{self:?}").to_ascii_uppercase())
    }
}

#[derive(Copy, Clone)]
pub struct Token {
    ty: TokenType,
    start: usize,
    end: usize,
    line: u32,
}

impl Token {
    #[inline]
    pub fn ty(&self) -> TokenType {
        self.ty
    }

    #[inline]
    pub fn start(&self) -> usize {
        self.start
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.end
    }

    #[inline]
    pub fn line(&self) -> u32 {
        self.line
    }
}

impl Default for Token {
    fn default() -> Self {
        Token {
            ty: TokenType::default(),
            start: 0,
            end: 0,
            line: 1,
        }
    }
}

struct Source {
    text: Vec<u8>,
    current: usize,
}

impl Source {
    fn new(text: String) -> Self {
        Source {
            text: text.into_bytes(),
            current: 0,
        }
    }

    fn next(&mut self) -> Option<u8> {
        self.peek().map(|c| {
            self.current += 1;
            c
        })
    }

    fn peek(&self) -> Option<u8> {
        (self.current < self.text.len()).then(|| self.text[self.current])
    }

    fn peek_peek(&self) -> Option<u8> {
        (self.current + 1 < self.text.len())
            .then(|| self.text[self.current + 1])
    }

    fn skip_if_eq(&mut self, expected: u8) -> bool {
        self.skip_if(|c| c == expected)
    }

    fn skip_if<P>(&mut self, mut predicate: P) -> bool
    where
        P: FnMut(u8) -> bool,
    {
        self.peek().map_or(false, |c| {
            predicate(c) && {
                self.current += 1;
                true
            }
        })
    }

    fn skip_while<P>(&mut self, mut predicate: P)
    where
        P: FnMut(u8) -> bool,
    {
        while self.skip_if(&mut predicate) {}
    }
}

pub struct Scanner {
    source: Source,
    start: usize,
    line: u32,
}

impl Scanner {
    pub fn new(text: String) -> Self {
        Scanner {
            source: Source::new(text),
            start: 0,
            line: 1,
        }
    }

    pub fn token_text(&self, token: Token) -> &str {
        unsafe {
            str::from_utf8_unchecked(&self.source.text[token.start..token.end])
        }
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    fn is_digit(c: u8) -> bool {
        (b'0'..=b'9').contains(&c)
    }

    fn is_alpha(c: u8) -> bool {
        (b'a'..=b'z').contains(&c) || (b'A'..=b'Z').contains(&c) || c == b'_'
    }

    fn is_ident(c: u8) -> bool {
        Scanner::is_alpha(c) || Scanner::is_digit(c)
    }

    #[inline]
    pub fn scan_token(&mut self) -> Result<Token> {
        self.skip_whitespace();
        let c = match self.source.next() {
            None => return Ok(self.make_token(TokenType::Eof)),
            Some(ch) => ch,
        };

        let token = match c {
            _ if Scanner::is_digit(c) => self.number(),
            _ if Scanner::is_alpha(c) => self.alpha(c),
            b'(' => self.make_token(TokenType::LeftParen),
            b')' => self.make_token(TokenType::RightParen),
            b'{' => self.make_token(TokenType::LeftBrace),
            b'}' => self.make_token(TokenType::RightBrace),
            b';' => self.make_token(TokenType::Semicolon),
            b',' => self.make_token(TokenType::Comma),
            b'.' => self.make_token(TokenType::Dot),
            b'-' => self.make_token(TokenType::Minus),
            b'+' => self.make_token(TokenType::Plus),
            b'/' => self.make_token(TokenType::Slash),
            b'*' => self.make_token(TokenType::Star),
            b'!' => {
                if self.matches(b'=') {
                    self.make_token(TokenType::BangEqual)
                } else {
                    self.make_token(TokenType::Bang)
                }
            }
            b'=' => {
                if self.matches(b'=') {
                    self.make_token(TokenType::EqualEqual)
                } else {
                    self.make_token(TokenType::Equal)
                }
            }
            b'<' => {
                if self.matches(b'=') {
                    self.make_token(TokenType::LessEqual)
                } else {
                    self.make_token(TokenType::Less)
                }
            }
            b'>' => {
                if self.matches(b'=') {
                    self.make_token(TokenType::GreaterEqual)
                } else {
                    self.make_token(TokenType::Greater)
                }
            }
            b'"' => self.string()?,
            _ => bail!("unexpected character '{}'", c as char),
        };
        Ok(token)
    }

    fn skip_whitespace(&mut self) {
        loop {
            self.source.skip_while(|c| {
                matches!(c, b' ' | b'\r' | b'\t')
                    || (c == b'\n') && {
                        self.line += 1;
                        true
                    }
            });

            if self.source.peek() == Some(b'/')
                && self.source.peek_peek() == Some(b'/')
            {
                self.source.skip_while(|c| c != b'\n');
                // if let Some(b'\n') = self.source.peek() {
                //     self.line += 1;
                // }
                // self.source.next();
                continue;
            }
            break;
        }

        self.start = self.source.current;
    }

    fn make_token(&mut self, ty: TokenType) -> Token {
        Token {
            ty,
            start: self.start,
            end: self.source.current,
            line: self.line,
        }
    }

    fn matches(&mut self, expected: u8) -> bool {
        self.source.skip_if_eq(expected)
    }

    fn string(&mut self) -> Result<Token> {
        let line = self.line;
        self.source.skip_while(|c| {
            (c == b'\n') && {
                self.line += 1;
                true
            } || c != b'"'
        });
        if self.source.peek().is_none() {
            self.line = line;
            bail!("unterminated string");
        }
        self.source.next();
        Ok(self.make_token(TokenType::String))
    }

    fn number(&mut self) -> Token {
        self.source.skip_while(Scanner::is_digit);
        if self.source.peek() == Some(b'.')
            && self.source.peek_peek().map_or(false, Scanner::is_digit)
        {
            self.source.next();
            self.source.skip_while(Scanner::is_digit);
        }
        self.make_token(TokenType::Number)
    }

    fn alpha(&mut self, c: u8) -> Token {
        match c {
            b'a' => self.check_keyword(false, b"nd", TokenType::And),
            b'c' => self.check_keyword(false, b"lass", TokenType::Class),
            b'e' => self.check_keyword(false, b"lse", TokenType::Else),
            b'i' => self.check_keyword(false, b"f", TokenType::If),
            b'n' => self.check_keyword(false, b"il", TokenType::Nil),
            b'o' => self.check_keyword(false, b"r", TokenType::Or),
            b'p' => self.check_keyword(false, b"rint", TokenType::Print),
            b'r' => self.check_keyword(false, b"eturn", TokenType::Return),
            b's' => self.check_keyword(false, b"uper", TokenType::Super),
            b'v' => self.check_keyword(false, b"ar", TokenType::Var),
            b'w' => self.check_keyword(false, b"hile", TokenType::While),
            b'f' => match self.source.peek() {
                Some(b'a') => {
                    self.check_keyword(true, b"lse", TokenType::False)
                }
                Some(b'o') => self.check_keyword(true, b"r", TokenType::For),
                Some(b'u') => self.check_keyword(true, b"n", TokenType::Fun),
                Some(_) => self.get_ident(),
                None => self.make_token(TokenType::Identifier),
            },
            b't' => match self.source.peek() {
                Some(b'h') => self.check_keyword(true, b"is", TokenType::This),
                Some(b'r') => self.check_keyword(true, b"ue", TokenType::True),
                Some(_) => self.get_ident(),
                None => self.make_token(TokenType::Identifier),
            },
            _ => self.get_ident(),
        }
    }

    fn check_keyword(
        &mut self,
        skip: bool,
        suffix: &[u8],
        ty: TokenType,
    ) -> Token {
        if skip {
            self.source.next();
        }
        let idx = self.source.current;
        let mut iter = suffix.iter();
        self.source.skip_while(|c| iter.next() == Some(&c));
        if self.source.current - idx == suffix.len() {
            let c = self.source.peek();
            if c.map(|ch| !Scanner::is_ident(ch)).unwrap_or(true) {
                return self.make_token(ty);
            }
        }

        self.get_ident()
    }

    fn get_ident(&mut self) -> Token {
        self.source.skip_while(Scanner::is_ident);
        self.make_token(TokenType::Identifier)
    }
}

#[cfg(test)]
mod test;
