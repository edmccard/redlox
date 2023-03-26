use std::{
    cell::RefCell,
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

#[cfg(test)]
mod test;

pub(crate) struct LoxFunction {}

#[derive(Clone, PartialEq)]
pub(crate) struct LoxString {
    text: Box<str>,
}

#[derive(Debug, thiserror::Error)]
#[error("{}", .msg)]
pub struct RuntimeError {
    msg: String,
}

struct SymTable {
    symbols: HashMap<Rc<str>, u32>,
    names: Vec<Rc<str>>,
}

pub struct Vm {
    stdout: Stdout,
    stderr: Stderr,
    stack: Vec<Value>,
    globals: HashMap<u32, Value>,
    symbols: SymTable,
}

type Result<T> = std::result::Result<T, RuntimeError>;

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
        Vm {
            stdout,
            stderr,
            stack: Vec::new(),
            globals: HashMap::new(),
            symbols: SymTable::new(),
        }
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

    pub(crate) fn get_sym_names(&self) -> &Vec<Rc<str>> {
        &self.symbols.names
    }

    pub(crate) fn get_symbol(&mut self, ident: &str) -> u32 {
        self.symbols.intern(ident)
    }

    pub fn interpret(&mut self, source: String) -> Result<()> {
        let mut parser = Parser::new(source, self.stderr.clone());
        match parser.parse(self) {
            Some(chunk) => self.run(&chunk),
            None => Ok(()),
        }
    }

    pub(crate) fn new_string(&mut self, text: &str) -> Value {
        let string = LoxString {
            text: Box::from(text),
        };
        let obj = Rc::new(RefCell::new(string));
        Value::String(Obj(obj))
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

    fn run(&mut self, chunk: &Chunk) -> Result<()> {
        let mut ip = chunk.instructions(0);
        let base = 0usize;
        while let Some(inst) = ip.next() {
            #[cfg(feature = "trace_execution")]
            {
                self.trace_stack();
                chunk.disassemble_instruction(inst, ip.offset - inst.len());
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
                            let value = self.new_string(
                                &[a.borrow().as_ref(), b.borrow().as_ref()]
                                    .concat(),
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

        Ok(())
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
