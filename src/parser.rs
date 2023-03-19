use anyhow::Error;
use num_enum::UnsafeFromPrimitive;

use crate::code::{Chunk, Op};
use crate::scanner::{Scanner, Token, TokenType};
use crate::{Stderr, Value, Vm};

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
            TokenType::And => Prec::And,
            TokenType::Or => Prec::Or,
            _ => Prec::None,
        }
    }
}

pub struct Parser {
    stderr: Stderr,
    scanner: Scanner,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
    locals: Locals,
    code: Vec<Chunk>,
}

impl Parser {
    pub fn new(source: String, stderr: Stderr) -> Parser {
        Parser {
            stderr,
            scanner: Scanner::new(source),
            current: Token::default(),
            previous: Token::default(),
            had_error: false,
            panic_mode: false,
            locals: Locals::new(),
            code: Vec::new(),
        }
    }

    pub fn parse(&mut self, vm: &mut Vm) -> Option<Chunk> {
        self.code.push(Chunk::new());

        self.advance();

        while !(self.matches(TokenType::Eof)) {
            self.declaration(vm, None);
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
    pub fn bench(&mut self) -> crate::Result<()> {
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

    // TODO: ternary operator
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
        } else if self.matches(TokenType::LeftBrace) {
            self.begin_scope();
            self.block(vm, loop_);
            self.end_scope();
        } else {
            self.expression_statement(vm);
        }
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

    fn print_statement(&mut self, vm: &mut Vm) {
        self.expression(vm);
        self.consume(TokenType::Semicolon, "expect ';' after value");
        self.emit_op(Op::Print);
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

    fn block(&mut self, vm: &mut Vm, loop_: Option<LoopInfo>) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof)
        {
            self.declaration(vm, loop_);
        }
        self.consume(TokenType::RightBrace, "expect '}' after block");
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

    fn and(&mut self, vm: &mut Vm) {
        let end_jump = self.emit_jump(Op::JumpIfFalse);
        self.emit_op(Op::Pop);
        self.parse_precedence(Prec::And, vm);
        self.patch_jump(end_jump);
    }

    fn or(&mut self, vm: &mut Vm) {
        let else_jump = self.emit_jump(Op::JumpIfFalse);
        let end_jump = self.emit_jump(Op::Jump);
        self.patch_jump(else_jump);
        self.emit_op(Op::Pop);
        self.parse_precedence(Prec::Or, vm);
        self.patch_jump(end_jump);
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

    fn emit_jump(&mut self, op: Op) -> usize {
        self.chunk().write_jump(op)
    }

    fn patch_jump(&mut self, origin: usize) {
        // Forward jumps are always 2 ops
        let delta = self.chunk().len() - origin - 2;
        if delta > 0xffff {
            self.error("too much code to jump over");
        }
        self.chunk().patch_jump(origin, delta as u16);
    }

    fn begin_scope(&mut self) {
        self.locals.begin_scope();
    }

    fn end_scope(&mut self) {
        let n = self.locals.end_scope() as u32;
        if n > 0 {
            self.emit_op_arg(Op::PopN, n);
        }
    }

    fn scan_error(&mut self, err: Error) {
        self.report_error(self.scanner.line(), format!(": {}", err));
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
        let _ =
            writeln!(self.stderr.borrow_mut(), "[line {}] Error{}", line, msg);
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
}

#[derive(Copy, Clone)]
struct LoopInfo {
    depth: i32,
    loop_start: usize,
    exit_jump: usize,
}

struct Locals {
    depth: i32,
    locals: Vec<Local>,
}

struct Local {
    sym: u32,
    depth: i32,
}

impl Locals {
    fn new() -> Self {
        Locals {
            depth: 0,
            locals: Vec::new(),
        }
    }

    fn top_level(&self) -> bool {
        self.depth == 0
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

    fn resolve(&self, sym: u32) -> Option<(usize, bool)> {
        let idx = self.locals.iter().rev().position(|local| local.sym == sym);
        idx.map(|i| {
            let slot = self.locals.len() - i - 1;
            (slot, self.locals[slot].depth != -1)
        })
    }

    fn mark_initialized(&mut self) {
        let idx = self.locals.len() - 1;
        self.locals[idx].depth = self.depth;
    }

    fn begin_scope(&mut self) {
        self.depth += 1;
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

    fn count_to_depth(&self, depth: i32) -> usize {
        let mut count = 0usize;
        while (count + 1) <= self.locals.len()
            && self.locals[self.locals.len() - (count + 1)].depth > depth
        {
            count += 1;
        }
        count
    }
}
