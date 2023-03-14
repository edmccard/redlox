use anyhow::Error;
use num_enum::UnsafeFromPrimitive;

use crate::code::{Chunk, Op};
use crate::scanner::{Scanner, Token, TokenType};
use crate::{Value, Vm};

#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    UnsafeFromPrimitive
)]
#[repr(u32)]
enum Prec {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Prec {
    fn next(self) -> Self {
        unsafe { Prec::from_unchecked(self as u32 + 1) }
    }

    fn for_op_type(ty: TokenType) -> Self {
        match ty {
            TokenType::Minus | TokenType::Plus => Prec::Term,
            TokenType::Slash | TokenType::Star => Prec::Factor,
            TokenType::BangEqual | TokenType::EqualEqual => Prec::Equality,
            TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => Prec::Comparison,
            _ => Prec::None,
        }
    }
}

pub struct Parser {
    scanner: Scanner,
    code: Vec<Chunk>,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
}

impl Parser {
    pub fn new(source: String) -> Parser {
        Parser {
            scanner: Scanner::new(source),
            code: Vec::new(),
            current: Token::default(),
            previous: Token::default(),
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn parse(&mut self, vm: &mut Vm) -> Option<Chunk> {
        self.code.push(Chunk::new());

        self.advance();

        while !(self.matches(TokenType::Eof)) {
            self.declaration(vm);
        }

        self.emit_op(Op::Return);

        #[cfg(feature = "print_code")]
        if !self.had_error {
            self.chunk().disassemble("<script>", vm.get_sym_names());
        }

        let chunk = self.code.pop().unwrap();
        (!self.had_error).then_some(chunk)
    }

    #[cfg(debug_assertions)]
    pub fn show_tokens(&mut self) {
        let mut line: u32 = 0;
        loop {
            self.advance();
            if self.had_error {
                break;
            }
            let token = self.current;
            if token.line() != line {
                line = token.line();
                print!("{:4} ", line);
            } else {
                print!("   | ");
            }
            println!("{:12} {}", token.ty(), self.scanner.token_text(token));
            if token.ty() == TokenType::Eof {
                break;
            }
        }
    }

    #[cfg(feature = "bench_mode")]
    pub fn bench(&mut self) -> Result<()> {
        let mut b1 = 0usize;
        let mut b2 = 0usize;
        let mut b3 = 0usize;
        let mut b4 = 0usize;
        loop {
            let token = self.scanner.scan_token()?;
            b1 += token.ty() as u8 as usize;
            b2 += token.start();
            b3 += token.end();
            b4 += token.line() as usize;

            if token.ty() == TokenType::Eof {
                break;
            }
        }
        println!("{} {} {} {}", b1, b2, b3, b4);

        Ok(())
    }

    fn chunk(&mut self) -> &mut Chunk {
        &mut self.code[0]
    }

    fn advance(&mut self) {
        self.previous = self.current;
        loop {
            match self.scanner.scan_token() {
                Ok(token) => {
                    self.current = token;
                    let line = self.current.line();
                    if line != self.previous.line() {
                        self.chunk().new_line(line);
                    }
                    break;
                }
                Err(e) => self.scan_error(e),
            }
        }
    }

    fn consume(&mut self, ty: TokenType, msg: &str) {
        if self.current.ty() == ty {
            self.advance();
        } else {
            self.error_at(self.current, msg)
        }
    }

    fn matches(&mut self, ty: TokenType) -> bool {
        if self.check(ty) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, ty: TokenType) -> bool {
        self.current.ty() == ty
    }

    fn token_text(&self) -> &str {
        self.scanner.token_text(self.previous)
    }

    fn parse_precedence(&mut self, precedence: Prec, vm: &mut Vm) {
        self.advance();

        let can_assign = precedence <= Prec::Assignment;
        match self.previous.ty() {
            TokenType::LeftParen => self.grouping(vm),
            TokenType::Minus | TokenType::Bang => self.unary(vm),
            TokenType::Number => self.number(),
            TokenType::Identifier => self.variable(vm, can_assign),
            TokenType::String => self.string(vm),
            TokenType::Nil | TokenType::True | TokenType::False => {
                self.literal()
            }
            _ => {
                self.error("expect expression");
                return;
            }
        }

        while precedence <= Prec::for_op_type(self.current.ty()) {
            self.advance();
            match self.previous.ty() {
                TokenType::Minus
                | TokenType::Plus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::EqualEqual
                | TokenType::Greater
                | TokenType::GreaterEqual
                | TokenType::Less
                | TokenType::LessEqual => self.binary(vm),
                _ => unreachable!(),
            }
        }

        if can_assign && self.matches(TokenType::Equal) {
            self.error("invalid assignment target");
        }
    }

    fn number(&mut self) {
        let value = self.token_text().parse::<f64>().unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn literal(&mut self) {
        let op = match self.previous.ty() {
            TokenType::Nil => Op::Nil,
            TokenType::True => Op::True,
            TokenType::False => Op::False,
            _ => unreachable!(),
        };
        self.emit_op(op);
    }

    fn string(&mut self, vm: &mut Vm) {
        let raw = self.token_text();
        let value = vm.new_string(&raw[1..raw.len() - 1]);
        self.emit_constant(value);
    }

    fn variable(&mut self, vm: &mut Vm, can_assign: bool) {
        let sym = vm.get_symbol(self.token_text());

        if can_assign && self.matches(TokenType::Equal) {
            self.expression(vm);
            self.emit_op_arg(Op::SetGlobal, sym);
        } else {
            self.emit_op_arg(Op::GetGlobal, sym)
        }
    }

    fn declaration(&mut self, vm: &mut Vm) {
        if self.matches(TokenType::Var) {
            self.var_declaration(vm);
        } else {
            self.statement(vm);
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn var_declaration(&mut self, vm: &mut Vm) {
        self.consume(TokenType::Identifier, "expect variable name");
        let sym = vm.get_symbol(self.token_text());
        if self.matches(TokenType::Equal) {
            self.expression(vm);
        } else {
            self.emit_op(Op::Nil);
        }
        self.consume(
            TokenType::Semicolon,
            "expect ';' after variable declaration",
        );
        self.emit_op_arg(Op::DefineGlobal, sym);
    }

    fn statement(&mut self, vm: &mut Vm) {
        if self.matches(TokenType::Print) {
            self.print_statement(vm);
        } else {
            self.expression_statement(vm);
        }
    }

    fn print_statement(&mut self, vm: &mut Vm) {
        self.expression(vm);
        self.consume(TokenType::Semicolon, "expect ';' after value");
        self.emit_op(Op::Print);
    }

    fn expression_statement(&mut self, vm: &mut Vm) {
        self.expression(vm);
        self.consume(TokenType::Semicolon, "expect ';' after expression");
        self.emit_op(Op::Pop);
    }

    fn expression(&mut self, vm: &mut Vm) {
        self.parse_precedence(Prec::Assignment, vm);
    }

    fn grouping(&mut self, vm: &mut Vm) {
        self.expression(vm);
        self.consume(TokenType::RightParen, "expect ')' after expression");
    }

    fn unary(&mut self, vm: &mut Vm) {
        let operator_type = self.previous.ty();

        self.parse_precedence(Prec::Unary, vm);

        match operator_type {
            TokenType::Minus => self.emit_op(Op::Negate),
            TokenType::Bang => self.emit_op(Op::Not),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self, vm: &mut Vm) {
        let operator_type = self.previous.ty();
        self.parse_precedence(Prec::for_op_type(operator_type).next(), vm);

        match operator_type {
            TokenType::Plus => self.emit_op(Op::Add),
            TokenType::Minus => self.emit_op(Op::Subtract),
            TokenType::Star => self.emit_op(Op::Multiply),
            TokenType::Slash => self.emit_op(Op::Divide),
            TokenType::EqualEqual => self.emit_op(Op::Equal),
            TokenType::Less => self.emit_op(Op::Less),
            TokenType::Greater => self.emit_op(Op::Greater),
            TokenType::BangEqual => {
                self.emit_op(Op::Equal);
                self.emit_op(Op::Not);
            }
            TokenType::GreaterEqual => {
                self.emit_op(Op::Less);
                self.emit_op(Op::Not);
            }
            TokenType::LessEqual => {
                self.emit_op(Op::Greater);
                self.emit_op(Op::Not);
            }
            _ => unreachable!(),
        }
    }

    fn emit_op(&mut self, op: Op) {
        self.chunk().write_op(op);
    }

    fn emit_op_arg(&mut self, op: Op, arg: u32) {
        self.chunk().write_op_arg(op, arg);
    }

    fn emit_constant(&mut self, value: Value) {
        let chunk = self.chunk();
        let arg = match chunk.add_constant(value) {
            Ok(idx) => idx,
            Err(e) => {
                self.error(&e.to_string());
                return;
            }
        };
        chunk.write_op_arg(Op::Constant, arg);
    }

    fn scan_error(&mut self, err: Error) {
        self.report_error(self.previous.line(), format!(": {}", err));
    }

    fn error(&mut self, msg: &str) {
        self.error_at(self.previous, msg);
    }

    fn error_at(&mut self, token: Token, msg: &str) {
        let msg = match token.ty() {
            TokenType::Eof => format!(" at end: {}", msg),
            _ => format!(" at '{}': {}", self.scanner.token_text(token), msg),
        };
        self.report_error(token.line(), msg);
    }

    fn report_error(&mut self, line: u32, msg: String) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        self.had_error = true;
        eprintln!("[line {}] Error{}", line, msg);
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;
        while self.current.ty() != TokenType::Eof {
            if self.previous.ty() == TokenType::Semicolon {
                return;
            }
            match self.current.ty() {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => self.advance(),
            }
        }
    }
}
