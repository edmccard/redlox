use std::fmt::Display;
use std::str::from_utf8_unchecked;

use anyhow::{bail, Result};

#[cfg(test)]
mod test;

#[derive(Copy, Clone)]
pub(super) struct Token {
    ty: TokenType,
    start: usize,
    end: usize,
    line: u32,
}

pub(super) struct Scanner {
    source: Source,
    current: usize,
    line: u32,
}

struct Source {
    text: Vec<u8>,
    current: usize,
}

pub fn bench_scanner(text: String) -> Result<()> {
    let mut b1 = 0usize;
    let mut b2 = 0usize;
    let mut b3 = 0usize;
    let mut b4 = 0usize;
    let mut scanner = Scanner::new(text);
    loop {
        let token = scanner.scan_token()?;
        b1 += token.ty as u8 as usize;
        b2 += token.start;
        b3 += token.end;
        b4 += token.line as usize;
        if token.ty == TokenType::Eof {
            break;
        }
    }
    println!("{} {} {} {}", b1, b2, b3, b4);
    Ok(())
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{self:?}").to_ascii_uppercase())
    }
}

impl Token {
    fn new() -> Self {
        Token {
            ty: TokenType::default(),
            start: 0,
            end: 0,
            line: 1,
        }
    }

    pub(super) fn ty(&self) -> TokenType {
        self.ty
    }

    pub(super) fn line(&self) -> u32 {
        self.line
    }
}

impl Default for Token {
    fn default() -> Self {
        Token::new()
    }
}

impl Scanner {
    pub(super) fn new(text: String) -> Self {
        Scanner {
            source: Source::new(text),
            current: 0,
            line: 1,
        }
    }

    fn alpha(&mut self, c: u8) -> Token {
        match c {
            b'a' => self.check_keyword(false, b"nd", TokenType::And),
            b'b' => self.check_keyword(false, b"reak", TokenType::Break),
            b'c' => match self.source.peek() {
                Some(b'a') => self.check_keyword(true, b"se", TokenType::Case),
                Some(b'l') => {
                    self.check_keyword(true, b"ass", TokenType::Class)
                }
                Some(b'o') => {
                    self.check_keyword(true, b"ntinue", TokenType::Continue)
                }
                Some(_) => self.get_ident(),
                None => self.make_token(TokenType::Identifier),
            },
            b'd' => self.check_keyword(false, b"efault", TokenType::Default),
            b'e' => self.check_keyword(false, b"lse", TokenType::Else),
            b'i' => self.check_keyword(false, b"f", TokenType::If),
            b'n' => self.check_keyword(false, b"il", TokenType::Nil),
            b'o' => self.check_keyword(false, b"r", TokenType::Or),
            b'p' => self.check_keyword(false, b"rint", TokenType::Print),
            b'r' => self.check_keyword(false, b"eturn", TokenType::Return),
            b's' => match self.source.peek() {
                Some(b'u') => {
                    self.check_keyword(true, b"per", TokenType::Super)
                }
                Some(b'w') => {
                    self.check_keyword(true, b"itch", TokenType::Switch)
                }
                Some(_) => self.get_ident(),
                None => self.make_token(TokenType::Identifier),
            },
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

    fn is_alpha(c: u8) -> bool {
        c.is_ascii_lowercase() || c.is_ascii_uppercase() || c == b'_'
    }

    fn is_digit(c: u8) -> bool {
        c.is_ascii_digit()
    }

    fn is_ident(c: u8) -> bool {
        Scanner::is_alpha(c) || Scanner::is_digit(c)
    }

    pub(super) fn line(&self) -> u32 {
        self.line
    }

    fn make_token(&mut self, ty: TokenType) -> Token {
        Token {
            ty,
            start: self.current,
            end: self.source.current,
            line: self.line,
        }
    }

    fn matches(&mut self, expected: u8) -> bool {
        self.source.skip_if_eq(expected)
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

    #[inline]
    pub(super) fn scan_token(&mut self) -> Result<Token> {
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
            b':' => self.make_token(TokenType::Colon),
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
            _ => {
                let ch = self.skip_unexpected();
                bail!("unexpected character '{}'", ch);
            }
        };
        Ok(token)
    }

    fn skip_unexpected(&mut self) -> char {
        let text = unsafe {
            from_utf8_unchecked(&self.source.text[(self.source.current - 1)..])
        };
        let c = text.chars().next().unwrap();
        self.source.current += c.len_utf8() - 1;
        self.current = self.source.current;
        c
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
                continue;
            }
            break;
        }

        self.current = self.source.current;
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

    pub(super) fn token_text(&self, token: Token) -> &str {
        unsafe {
            from_utf8_unchecked(&self.source.text[token.start..token.end])
        }
    }
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
        self.text.get(self.current).copied()
    }

    fn peek_peek(&self) -> Option<u8> {
        self.text.get(self.current + 1).copied()
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

    fn skip_if_eq(&mut self, expected: u8) -> bool {
        self.skip_if(|c| c == expected)
    }

    fn skip_while<P>(&mut self, mut predicate: P)
    where
        P: FnMut(u8) -> bool,
    {
        while self.skip_if(&mut predicate) {}
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub(crate) enum TokenType {
    #[default]
    Eof,
    // Punctuation
    Colon,
    Comma,
    LeftBrace,
    LeftParen,
    RightBrace,
    RightParen,
    Semicolon,
    // Operators
    Bang,
    BangEqual,
    Dot,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Minus,
    Plus,
    Slash,
    Star,
    // Values
    Identifier,
    Number,
    String,
    // Keywords
    And,
    Break,
    Case,
    Class,
    Continue,
    Default,
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
    Switch,
    This,
    True,
    Var,
    While,
}
