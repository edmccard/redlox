use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Display,
    ops::Deref,
    rc::Rc,
};

use crate::{
    code::{Chunk, Op},
    parser::Parser,
    Obj, Stderr, Stdout, Value,
};

mod native;

#[cfg(test)]
mod test;

struct Frame {
    func: Obj<LoxFunction>,
    offset: usize,
    base: usize,
}

#[derive(Default)]
pub(crate) struct LoxFunction {
    name: String,
    pub(crate) arity: usize,
    pub(crate) chunk: Chunk,
}

#[derive(PartialEq)]
pub(crate) struct LoxString {
    text: Box<str>,
}

#[derive(Debug, thiserror::Error)]
#[error("{}", .msg)]
pub struct RuntimeError {
    msg: String,
}

#[derive(Clone)]
pub(crate) struct RustFunction {
    name: String,
    arity: usize,
    func: NativeFn,
}

struct SymTable {
    symbols: HashMap<Rc<str>, u32>,
    names: Vec<Rc<str>>,
}

pub struct Vm {
    stdout: Stdout,
    stderr: Stderr,
    frames: Vec<Frame>,
    stack: Vec<Value>,
    globals: HashMap<u32, Value>,
    symbols: SymTable,
}

type Result<T> = std::result::Result<T, RuntimeError>;
type NativeFn = fn(usize, vm: &mut Vm) -> Result<Value>;

impl LoxFunction {
    pub(crate) fn new(name: &str) -> Self {
        LoxFunction {
            name: name.to_string(),
            arity: 0,
            chunk: Chunk::default(),
        }
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl LoxString {
    pub(crate) fn new(text: &str) -> Self {
        LoxString {
            text: Box::from(text),
        }
    }
}

impl Deref for LoxString {
    type Target = Box<str>;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl Display for LoxString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.text.fmt(f)
    }
}

impl RuntimeError {
    fn new(msg: String) -> Self {
        RuntimeError { msg }
    }

    fn with_line(&self, line: u32) -> Self {
        RuntimeError {
            msg: format!("[line {}] {}", line, self.msg),
        }
    }
}

impl Display for RustFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for RustFunction {
    fn eq(&self, other: &Self) -> bool {
        self.func as usize == other.func as usize
    }
}

impl SymTable {
    fn new() -> Self {
        SymTable {
            symbols: HashMap::new(),
            names: Vec::new(),
        }
    }

    fn intern(&mut self, ident: &str) -> u32 {
        if let Some(&sym) = self.symbols.get(ident) {
            return sym;
        }
        let idx = self.names.len() as u32;
        let name: Rc<str> = ident.into();
        self.symbols.insert(name.clone(), idx);
        self.names.push(name);
        idx
    }

    fn lookup(&self, sym: u32) -> Rc<str> {
        self.names[sym as usize].clone()
    }
}

impl Vm {
    const MAX_STACK: usize = 65536;

    pub fn new(stdout: Stdout, stderr: Stderr) -> Self {
        let mut vm = Vm {
            stdout,
            stderr,
            frames: Vec::new(),
            stack: Vec::new(),
            globals: HashMap::new(),
            symbols: SymTable::new(),
        };
        vm.add_native("clock", 0, native::clock);
        vm
    }

    fn add_native(&mut self, name: &str, arity: usize, func: NativeFn) {
        let native_fn = RustFunction {
            name: name.to_string(),
            arity,
            func,
        };
        let sym = self.get_symbol(name);
        self.globals.insert(sym, Value::Builtin(native_fn.into()));
    }

    fn arithmetic_args(&mut self) -> Result<(f64, f64)> {
        let b = self.pop();
        let a = self.peek(0);
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => Ok((a, b)),
            _ => {
                self.pop();
                Err(RuntimeError::new("operands must be numbers".to_string()))
            }
        }
    }

    fn error(msg: &str) -> Result<()> {
        Err(RuntimeError::new(msg.to_string()))
    }

    pub(crate) fn get_sym_name(&self, sym: u32) -> Rc<str> {
        self.symbols.lookup(sym)
    }

    pub(crate) fn get_sym_names(&self) -> &Vec<Rc<str>> {
        &self.symbols.names
    }

    pub(crate) fn get_symbol(&mut self, ident: &str) -> u32 {
        self.symbols.intern(ident)
    }

    pub fn interpret(&mut self, source: String) -> Result<()> {
        let mut parser = Parser::new(source, self.stderr.clone());
        match parser.parse(self, "<script>") {
            Some(func) => self.run(func),
            None => Ok(()),
        }
    }

    fn peek(&self, count: usize) -> Value {
        let idx = self.stack.len() - (count + 1);
        self.stack[idx].clone()
    }

    fn poke(&mut self, count: usize, val: Value) -> Result<()> {
        let idx = self.stack.len() - (count + 1);
        self.stack[idx] = val;
        Ok(())
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn push(&mut self, val: Value) -> Result<()> {
        if self.stack.len() < Vm::MAX_STACK {
            self.stack.push(val);
            Ok(())
        } else {
            Vm::error("stack overflow")
        }
    }

    fn run(&mut self, script: LoxFunction) -> Result<()> {
        self.frames.push(Frame {
            func: script.into(),
            base: 0,
            offset: 0,
        });
        self.push(Value::Nil).unwrap();
        let mut current = 0;
        loop {
            match self.run_frame(current) {
                Ok(None) => {
                    let frame = self.frames.pop().unwrap();
                    let result = self.pop();
                    self.stack.truncate(frame.base);
                    if current == 0 {
                        break;
                    }
                    self.push(result).unwrap();
                    current -= 1;
                }
                Ok(Some(frame)) => {
                    self.frames.push(frame);
                    current += 1;
                }
                // TODO: stack traces
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn run_frame(&mut self, current: usize) -> Result<Option<Frame>> {
        let func = self.frames[current].func.clone();
        let chunk = &func.borrow().chunk;
        let offset = self.frames[current].offset;
        let mut ip = chunk.instructions(offset);
        let base = self.frames[current].base;

        while let Some(inst) = ip.next() {
            #[cfg(feature = "trace_execution")]
            {
                self.trace_stack();
                chunk.disassemble_instruction(
                    inst,
                    ip.offset - inst.len(),
                    self.get_sym_names(),
                );
            }

            let result = match inst.opcode() {
                Op::Nil => self.push(Value::Nil),
                Op::True => self.push(Value::TRUE),
                Op::False => self.push(Value::FALSE),
                Op::Pop => {
                    self.pop();
                    Ok(())
                }
                Op::Print => {
                    let val = self.pop();
                    let _ = writeln!(self.stdout.borrow_mut(), "{}", val);
                    Ok(())
                }
                Op::Return => {
                    break;
                }
                Op::Not => {
                    let arg = bool::from(self.peek(0));
                    self.poke(0, Value::Boolean(!arg))
                }
                Op::Negate => {
                    let arg = self.peek(0);
                    match arg {
                        Value::Number(v) => self.poke(0, Value::Number(-v)),
                        _ => {
                            self.pop();
                            Vm::error("operand must be a number")
                        }
                    }
                }
                Op::Equal => {
                    let b = self.pop();
                    let a = self.peek(0);
                    self.poke(0, Value::Boolean(a == b))
                }
                Op::Greater => self
                    .arithmetic_args()
                    .and_then(|(a, b)| self.poke(0, Value::Boolean(a > b))),
                Op::Less => self
                    .arithmetic_args()
                    .and_then(|(a, b)| self.poke(0, Value::Boolean(a < b))),
                Op::Add => {
                    let b = self.pop();
                    let a = self.peek(0);
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.poke(0, Value::Number(a + b))
                        }
                        (Value::String(a), Value::String(b)) => {
                            let value = Value::String(
                                LoxString::new(
                                    &[a.borrow().as_ref(), b.borrow().as_ref()]
                                        .concat(),
                                )
                                .into(),
                            );
                            self.poke(0, value)
                        }
                        _ => {
                            self.pop();
                            Vm::error("operands must be numbers or strings")
                        }
                    }
                }
                Op::Subtract => self
                    .arithmetic_args()
                    .and_then(|(a, b)| self.poke(0, Value::Number(a - b))),
                Op::Multiply => self
                    .arithmetic_args()
                    .and_then(|(a, b)| self.poke(0, Value::Number(a * b))),
                Op::Divide => self
                    .arithmetic_args()
                    .and_then(|(a, b)| self.poke(0, Value::Number(a / b))),
                Op::Constant => {
                    let constant = chunk.get_constant(inst.operand());
                    self.push(constant)
                }
                Op::Call => {
                    let arg_count = inst.operand() as usize;
                    match self.peek(arg_count) {
                        Value::Function(f) => {
                            let arity = f.borrow().arity;
                            if arity != arg_count {
                                Vm::error(&format!(
                                    "expected {} arguments but got {}",
                                    arity, arg_count
                                ))
                            } else {
                                self.frames[current].offset = ip.offset;
                                return Ok(Some(Frame {
                                    func: f,
                                    base: self.stack.len() - arg_count - 1,
                                    offset: 0,
                                }));
                            }
                        }
                        Value::Builtin(f) => {
                            let arity = f.borrow().arity;
                            if arity != arg_count {
                                Vm::error(&format!(
                                    "expected {} arguments but got {}",
                                    arity, arg_count
                                ))
                            } else {
                                let func = f.borrow().func;
                                match func(arg_count, self) {
                                    Ok(v) => {
                                        self.stack.truncate(
                                            self.stack.len() - arg_count + 1,
                                        );
                                        self.push(v)
                                    }
                                    Err(e) => Err(e),
                                }
                            }
                        }
                        _ => Vm::error("can only call functions or classes"),
                    }
                }
                Op::PopN => {
                    let new_len = self.stack.len() - inst.operand() as usize;
                    self.stack.truncate(new_len);
                    Ok(())
                }
                Op::DefineGlobal => {
                    let global = self.pop();
                    self.globals.insert(inst.operand(), global);
                    Ok(())
                }
                Op::GetGlobal => match self.globals.get(&inst.operand()) {
                    None => Vm::error(&format!(
                        "undefined variable '{}'",
                        self.symbols.names[inst.operand() as usize]
                    )),
                    Some(val) => self.push(val.clone()),
                },
                Op::SetGlobal => {
                    let val = self.peek(0);
                    match self.globals.entry(inst.operand()) {
                        Entry::Occupied(mut entry) => {
                            entry.insert(val);
                            Ok(())
                        }
                        Entry::Vacant(_) => Err(RuntimeError::new(format!(
                            "undefined variable '{}'",
                            self.symbols.names[inst.operand() as usize]
                        ))),
                    }
                }
                Op::GetLocal => {
                    let slot = inst.operand() as usize + base;
                    let local = self.stack[slot].clone();
                    self.push(local)
                }
                Op::SetLocal => {
                    let val = self.peek(0);
                    let slot = inst.operand() as usize + base;
                    self.stack[slot] = val.clone();
                    Ok(())
                }
                Op::JumpIfFalse => {
                    if !bool::from(self.peek(0)) {
                        ip.offset += inst.operand() as usize;
                    }
                    Ok(())
                }
                Op::Jump => {
                    ip.offset += inst.operand() as usize;
                    Ok(())
                }
                Op::Loop => {
                    ip.offset -= inst.operand() as usize;
                    Ok(())
                }
                Op::Nop => Ok(()),
                _ => Vm::error(&format!("unknown opcode {}", inst.opcode())),
            };
            result.map_err(|e| {
                let offset = ip.offset - inst.len();
                let line = chunk.get_line(offset);
                self.stack.clear();
                e.with_line(line)
            })?;
        }

        Ok(None)
    }

    #[cfg(feature = "trace_execution")]
    fn trace_stack(&self) {
        print!("          ");
        for elem in &self.stack {
            print!("[ {} ]", elem);
        }
        println!();
    }
}
