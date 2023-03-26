use std::{cell::RefCell, io, rc::Rc};

use anyhow::Error;

use crate::{
    code::{Chunk, Op, Opcode},
    vm::Vm,
    Stderr, Value,
};
use scanner::{Scanner, Token, TokenType};
use Prec::Precedence;

pub(super) mod scanner;

#[allow(non_upper_case_globals)]
#[allow(non_snake_case)]
mod Prec {
    use super::scanner::TokenType;

    pub const None: u32 = 0;
    pub const Assignment: u32 = 1;
    pub const Or: u32 = 2;
    pub const And: u32 = 3;
    pub const Equality: u32 = 4;
    pub const Comparison: u32 = 5;
    pub const Term: u32 = 6;
    pub const Factor: u32 = 7;
    pub const Unary: u32 = 8;
    pub const Call: u32 = 9;
    pub const Primary: u32 = 10;

    pub(crate) fn for_op_type(ty: TokenType) -> Precedence {
        match ty {
            TokenType::Minus | TokenType::Plus => Term,
            TokenType::Slash | TokenType::Star => Factor,
            TokenType::BangEqual | TokenType::EqualEqual => Equality,
            TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => Comparison,
            TokenType::And => And,
            TokenType::Or => Or,
            _ => None,
        }
    }

    pub type Precedence = u32;
}

struct Local {
    sym: u32,
    depth: i32,
}

struct Locals {
    depth: i32,
    locals: Vec<Local>,
}

#[derive(Copy, Clone)]
struct LoopInfo {
    depth: i32,
    loop_start: usize,
    exit_jump: usize,
}

pub(crate) struct Parser {
    scanner: Scanner,
    stderr: Stderr,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
    locals: Locals,
    chunks: Vec<Chunk>,
}

pub fn print_tokens(source: String) {
    let mut parser = Parser::new(source, Rc::new(RefCell::new(io::stderr())));
    let mut line: u32 = 0;
    loop {
        parser.advance();
        if parser.had_error {
            break;
        }
        let token = parser.current;
        if token.line() != line {
            line = token.line();
            print!("{:4} ", line);
        } else {
            print!("   | ");
        }
        println!("{:12} {}", token.ty(), parser.scanner.token_text(token));
        if token.ty() == TokenType::Eof {
            break;
        }
    }
}

impl Locals {
    fn new() -> Self {
        Locals {
            depth: 0,
            locals: Vec::new(),
        }
    }

    fn add(&mut self, sym: u32) -> bool {
        for local in self.locals.iter().rev() {
            if local.depth != -1 && local.depth < self.depth {
                break;
            }
            if local.sym == sym {
                return false;
            }
        }
        self.locals.push(Local { sym, depth: -1 });
        true
    }

    fn begin_scope(&mut self) {
        self.depth += 1;
    }

    fn count_to_depth(&self, depth: i32) -> usize {
        let mut count = 0usize;
        while (count + 1) <= self.locals.len()
            && self.locals[self.locals.len() - (count + 1)].depth > depth
        {
            count += 1;
        }
        count
    }

    fn end_scope(&mut self) -> usize {
        self.depth -= 1;
        let mut count = 0usize;
        while !self.locals.is_empty()
            && self.locals[self.locals.len() - 1].depth > self.depth
        {
            count += 1;
            self.locals.pop();
        }
        count
    }

    fn inject(&mut self) -> usize {
        self.locals.push(Local {
            sym: u32::MAX,
            depth: self.depth,
        });
        self.locals.len() - 1
    }

    fn mark_initialized(&mut self) {
        let idx = self.locals.len() - 1;
        self.locals[idx].depth = self.depth;
    }

    fn resolve(&self, sym: u32) -> Option<(usize, bool)> {
        let idx = self.locals.iter().rev().position(|local| local.sym == sym);
        idx.map(|i| {
            let slot = self.locals.len() - i - 1;
            (slot, self.locals[slot].depth != -1)
        })
    }

    fn top_level(&self) -> bool {
        self.depth == 0
    }
}

impl Default for Locals {
    fn default() -> Self {
        Locals::new()
    }
}

impl Parser {
    pub(crate) fn new(source: String, stderr: Stderr) -> Parser {
        Parser {
            scanner: Scanner::new(source),
            stderr,
            current: Token::default(),
            previous: Token::default(),
            had_error: false,
            panic_mode: false,
            locals: Locals::new(),
            chunks: Vec::new(),
        }
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

    fn and(&mut self, vm: &mut Vm) {
        let end_jump = self.emit_jump(Op::JumpIfFalse);
        self.emit_op(Op::Pop);
        self.parse_precedence(Prec::And, vm);
        self.patch_jump(end_jump);
    }

    fn begin_scope(&mut self) {
        self.locals.begin_scope();
    }

    fn binary(&mut self, vm: &mut Vm) {
        let operator_type = self.previous.ty();
        self.parse_precedence(Prec::for_op_type(operator_type) + 1, vm);

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

    fn block(&mut self, vm: &mut Vm, loop_: Option<LoopInfo>) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof)
        {
            self.declaration(vm, loop_);
        }
        self.consume(TokenType::RightBrace, "expect '}' after block");
    }

    fn break_statement(&mut self, loop_: Option<LoopInfo>) {
        self.consume(TokenType::Semicolon, "expect ';' after 'break'");
        if loop_.is_none() {
            self.error("'break' outside of loop");
            return;
        }

        let loop_ = loop_.unwrap();
        let n = self.locals.count_to_depth(loop_.depth);
        if n > 0 {
            self.emit_op_arg(Op::PopN, n as u32);
        }
        self.emit_op(Op::False);
        self.emit_loop(loop_.exit_jump);
    }

    fn check(&self, ty: TokenType) -> bool {
        self.current.ty() == ty
    }

    fn chunk(&mut self) -> &mut Chunk {
        let idx = self.chunks.len() - 1;
        &mut self.chunks[idx]
    }

    fn consume(&mut self, ty: TokenType, msg: &str) {
        if self.current.ty() == ty {
            self.advance();
        } else {
            self.error_at(self.current, msg)
        }
    }

    fn continue_statement(&mut self, loop_: Option<LoopInfo>) {
        self.consume(TokenType::Semicolon, "expect ';' after 'continue'");
        if loop_.is_none() {
            self.error("'continue' outside of loop");
            return;
        }

        let loop_ = loop_.unwrap();
        let n = self.locals.count_to_depth(loop_.depth);
        if n > 0 {
            self.emit_op_arg(Op::PopN, n as u32);
        }
        self.emit_loop(loop_.loop_start);
    }

    fn declaration(&mut self, vm: &mut Vm, loop_: Option<LoopInfo>) {
        if self.matches(TokenType::Var) {
            self.var_declaration(vm);
        } else {
            self.statement(vm, loop_);
        }

        if self.panic_mode {
            self.synchronize();
        }
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

    fn emit_jump(&mut self, op: Opcode) -> usize {
        self.chunk().write_jump(op)
    }

    fn emit_loop(&mut self, dest: usize) {
        let mut delta = self.chunk().len() - dest + 1;
        if delta > 0xff {
            delta += 1;
        }
        if delta > 0xffff {
            self.error("loop body too large");
        }
        self.emit_op_arg(Op::Loop, delta as u32);
    }

    fn emit_op(&mut self, op: Opcode) {
        self.chunk().write_op(op);
    }

    fn emit_op_arg(&mut self, op: Opcode, arg: u32) {
        self.chunk().write_op_arg(op, arg);
    }

    fn end_scope(&mut self) {
        let n = self.locals.end_scope() as u32;
        if n == 1 {
            self.emit_op(Op::Pop);
        }
        if n > 1 {
            self.emit_op_arg(Op::PopN, n);
        }
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

    fn expression(&mut self, vm: &mut Vm) {
        self.parse_precedence(Prec::Assignment, vm);
    }

    fn expression_statement(&mut self, vm: &mut Vm) {
        self.expression(vm);
        self.consume(TokenType::Semicolon, "expect ';' after expression");
        self.emit_op(Op::Pop);
    }

    fn for_statement(&mut self, vm: &mut Vm) {
        self.begin_scope();

        self.consume(TokenType::LeftParen, "expect '(' after for");
        if self.matches(TokenType::Semicolon) {
            // no initializer
        } else if self.matches(TokenType::Var) {
            self.var_declaration(vm);
        } else {
            self.expression_statement(vm);
        }

        let mut loop_start = self.chunk().len();
        if self.matches(TokenType::Semicolon) {
            // no condition
            self.emit_op(Op::True);
        } else {
            self.expression(vm);
            self.consume(
                TokenType::Semicolon,
                "expect ';' after loop condition",
            );
        }
        let exit_jump = self.emit_jump(Op::JumpIfFalse);
        self.emit_op(Op::Pop);

        if !self.matches(TokenType::RightParen) {
            let body_jump = self.emit_jump(Op::Jump);
            let increment_start = self.chunk().len();
            self.expression(vm);
            self.emit_op(Op::Pop);
            self.consume(
                TokenType::RightParen,
                "expect ')' after loop clauses",
            );
            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        let loop_ = Some(LoopInfo {
            depth: self.locals.depth,
            loop_start,
            exit_jump,
        });
        self.statement(vm, loop_);
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_op(Op::Pop);

        self.end_scope();
    }

    fn grouping(&mut self, vm: &mut Vm) {
        self.expression(vm);
        self.consume(TokenType::RightParen, "expect ')' after expression");
    }

    fn if_statement(&mut self, vm: &mut Vm, loop_: Option<LoopInfo>) {
        self.consume(TokenType::LeftParen, "expect '(' after 'if'");
        self.expression(vm);
        self.consume(TokenType::RightParen, "expect ')' after condition");

        let then_jump = self.emit_jump(Op::JumpIfFalse);
        self.emit_op(Op::Pop);
        self.statement(vm, loop_);
        let else_jump = self.emit_jump(Op::Jump);
        self.patch_jump(then_jump);
        self.emit_op(Op::Pop);

        if self.matches(TokenType::Else) {
            self.statement(vm, loop_);
        }
        self.patch_jump(else_jump);
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

    fn matches(&mut self, ty: TokenType) -> bool {
        if self.check(ty) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn number(&mut self) {
        let value = self.token_text().parse::<f64>().unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn or(&mut self, vm: &mut Vm) {
        let else_jump = self.emit_jump(Op::JumpIfFalse);
        let end_jump = self.emit_jump(Op::Jump);
        self.patch_jump(else_jump);
        self.emit_op(Op::Pop);
        self.parse_precedence(Prec::Or, vm);
        self.patch_jump(end_jump);
    }

    pub(crate) fn parse(&mut self, vm: &mut Vm) -> Option<Chunk> {
        self.chunks.push(Chunk::default());

        self.advance();

        while !(self.matches(TokenType::Eof)) {
            self.declaration(vm, None);
        }

        self.emit_op(Op::Return);

        #[cfg(feature = "print_code")]
        if !self.had_error {
            self.chunk().disassemble("<script>", vm.get_sym_names());
        }

        let chunk = self.chunks.pop().unwrap();
        (!self.had_error).then_some(chunk)
    }

    fn parse_precedence(&mut self, precedence: Precedence, vm: &mut Vm) {
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
                | TokenType::BangEqual
                | TokenType::Greater
                | TokenType::GreaterEqual
                | TokenType::Less
                | TokenType::LessEqual => self.binary(vm),
                TokenType::And => self.and(vm),
                TokenType::Or => self.or(vm),
                _ => unreachable!(),
            }
        }

        if can_assign && self.matches(TokenType::Equal) {
            self.error("invalid assignment target");
        }
    }

    fn patch_jump(&mut self, origin: usize) {
        // Forward jumps are always 2 ops
        let delta = self.chunk().len() - origin - 2;
        if delta > 0xffff {
            self.error("too much code to jump over");
        }
        self.chunk().patch_jump(origin, delta as u16);
    }

    fn print_statement(&mut self, vm: &mut Vm) {
        self.expression(vm);
        self.consume(TokenType::Semicolon, "expect ';' after value");
        self.emit_op(Op::Print);
    }

    fn report_error(&mut self, line: u32, msg: String) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        self.had_error = true;
        let _ =
            writeln!(self.stderr.borrow_mut(), "[line {}] Error{}", line, msg);
    }

    fn scan_error(&mut self, err: Error) {
        self.report_error(self.scanner.line(), format!(": {}", err));
    }

    fn show_tokens(&mut self) {
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

    fn statement(&mut self, vm: &mut Vm, loop_: Option<LoopInfo>) {
        if self.matches(TokenType::Print) {
            self.print_statement(vm);
        } else if self.matches(TokenType::For) {
            self.for_statement(vm);
        } else if self.matches(TokenType::If) {
            self.if_statement(vm, loop_);
        } else if self.matches(TokenType::While) {
            self.while_statement(vm);
        } else if self.matches(TokenType::Break) {
            self.break_statement(loop_);
        } else if self.matches(TokenType::Continue) {
            self.continue_statement(loop_);
        } else if self.matches(TokenType::Switch) {
            self.switch_statement(vm, loop_);
        } else if self.matches(TokenType::LeftBrace) {
            self.begin_scope();
            self.block(vm, loop_);
            self.end_scope();
        } else {
            self.expression_statement(vm);
        }
    }

    fn string(&mut self, vm: &mut Vm) {
        let raw = self.token_text();
        let value = vm.new_string(&raw[1..raw.len() - 1]);
        self.emit_constant(value);
    }

    fn switch_case(&mut self, vm: &mut Vm, loop_: Option<LoopInfo>) {
        // TODO: begin scope to keep local count down?
        while !self.check(TokenType::Semicolon) && !self.check(TokenType::Eof) {
            self.statement(vm, loop_);
            if self.previous.ty() == TokenType::Semicolon {
                return;
            }
        }
        self.consume(TokenType::Semicolon, "expect ';' after switch case")
    }

    fn switch_statement(&mut self, vm: &mut Vm, loop_: Option<LoopInfo>) {
        // To contain the synthesized local
        self.begin_scope();

        self.consume(TokenType::LeftParen, "expect '(' after 'switch'");
        let test_slot = self.locals.inject();
        self.expression(vm);
        self.consume(
            TokenType::RightParen,
            "expect ')' after switch expression",
        );

        self.consume(TokenType::LeftBrace, "expect '{' before switch body");
        let mut patch_false: Option<usize> = None;
        let mut patch_true: Vec<usize> = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof)
        {
            if let Some(jump) = patch_false {
                self.patch_jump(jump);
                self.emit_op(Op::Pop);
            }
            if self.matches(TokenType::Case) || self.matches(TokenType::Default)
            {
                let default = self.previous.ty() == TokenType::Default;
                if !default {
                    self.emit_op_arg(Op::GetLocal, test_slot as u32);
                    self.expression(vm);
                    self.emit_op(Op::Equal);
                    patch_false = Some(self.emit_jump(Op::JumpIfFalse));
                    self.emit_op(Op::Pop);
                }
                self.consume(
                    TokenType::Colon,
                    "expect ':' after switch expression",
                );
                self.switch_case(vm, loop_);
                if default {
                    if !self.check(TokenType::RightBrace) {
                        self.error("default case must be the last case");
                    }
                    break;
                } else if !self.check(TokenType::RightBrace) {
                    patch_true.push(self.emit_jump(Op::Jump));
                }
            } else {
                self.error("expect switch case");
                break;
            }
        }
        self.consume(TokenType::RightBrace, "expect '}' after switch body");
        for origin in patch_true {
            self.patch_jump(origin);
        }

        self.end_scope();
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;
        while self.current.ty() != TokenType::Eof {
            if self.previous.ty() == TokenType::Semicolon {
                return;
            }
            // Ignore continue/break -- will synchronize on the following ';'
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

    fn token_text(&self) -> &str {
        self.scanner.token_text(self.previous)
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

    fn var_declaration(&mut self, vm: &mut Vm) {
        self.consume(TokenType::Identifier, "expect variable name");

        let sym = vm.get_symbol(self.token_text());

        if !self.locals.top_level() && !self.locals.add(sym) {
            self.error("already a variable with this name in this scope");
        }

        if self.matches(TokenType::Equal) {
            self.expression(vm);
        } else {
            self.emit_op(Op::Nil);
        }
        self.consume(
            TokenType::Semicolon,
            "expect ';' after variable declaration",
        );

        if self.locals.top_level() {
            self.emit_op_arg(Op::DefineGlobal, sym);
        } else {
            self.locals.mark_initialized();
        }
    }

    fn variable(&mut self, vm: &mut Vm, can_assign: bool) {
        let sym = vm.get_symbol(self.token_text());
        let (op_set, op_get, arg) = match self.locals.resolve(sym) {
            None => (Op::SetGlobal, Op::GetGlobal, sym),
            Some((slot, is_initialized)) => {
                if !is_initialized {
                    self.error(
                        "can't read local variable in its own initializer",
                    );
                }
                (Op::SetLocal, Op::GetLocal, slot as u32)
            }
        };

        if can_assign && self.matches(TokenType::Equal) {
            self.expression(vm);
            self.emit_op_arg(op_set, arg);
        } else {
            self.emit_op_arg(op_get, arg);
        }
    }

    fn while_statement(&mut self, vm: &mut Vm) {
        let loop_start = self.chunk().len();
        self.consume(TokenType::LeftParen, "expect '(' after 'while'");
        self.expression(vm);
        self.consume(TokenType::RightParen, "expect ')' after condition");

        let exit_jump = self.emit_jump(Op::JumpIfFalse);
        self.emit_op(Op::Pop);

        let loop_ = Some(LoopInfo {
            depth: self.locals.depth,
            loop_start,
            exit_jump,
        });
        self.statement(vm, loop_);

        self.emit_loop(loop_start);
        self.patch_jump(exit_jump);
        self.emit_op(Op::Pop);
    }
}
