use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;
use std::rc::{Rc, Weak};

use crate::code::{Chunk, Op};
use crate::parser::Parser;
use crate::Value;

pub(crate) type Obj = Rc<RefCell<Object>>;
type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(PartialEq, PartialOrd)]
pub(crate) struct Object {
    payload: Payload,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.payload.fmt(f)
    }
}

#[derive(PartialEq, PartialOrd)]
enum Payload {
    String(Box<str>),
}

impl fmt::Display for Payload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Payload::String(v) => write!(f, "\"{}\"", v),
        }
    }
}

// TODO: faster global access
pub struct Vm {
    stack: Vec<Value>,
    heap: Vec<Weak<RefCell<Object>>>,
    symbols: SymTable,
    globals: HashMap<u32, Value>,
}

impl Vm {
    const MAX_STACK: usize = 65536;

    pub fn init() -> Self {
        Vm {
            stack: Vec::new(),
            heap: Vec::new(),
            symbols: SymTable::new(),
            globals: HashMap::new(),
        }
    }

    fn error(msg: &str) -> Result<()> {
        Err(RuntimeError::new(msg.to_string()))
    }

    pub fn interpret(&mut self, source: String) -> Result<()> {
        let mut parser = Parser::new(source);
        match parser.parse(self) {
            Some(chunk) => self.run(&chunk),
            None => Ok(()),
        }
    }

    fn run(&mut self, chunk: &Chunk) -> Result<()> {
        let mut ip = chunk.instructions();
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
                    println!("{}", self.pop());
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
                        (Value::Object(a), Value::Object(b)) => {
                            self.add_objects(a, b)
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
                    let local = self.stack[inst.operand() as usize].clone();
                    self.push(local)
                }
                Op::SetLocal => {
                    let val = self.peek(0);
                    self.stack[inst.operand() as usize] = val.clone();
                    Ok(())
                }
                _ => Vm::error("unknown opcode"),
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

    pub(crate) fn new_string(&mut self, text: &str) -> Value {
        let object = Object {
            payload: Payload::String(Box::from(text)),
        };
        let obj = Rc::new(RefCell::new(object));
        self.heap.push(Rc::downgrade(&obj));
        Value::Object(obj)
    }

    pub(crate) fn get_symbol(&mut self, ident: &str) -> u32 {
        self.symbols.intern(ident)
    }

    pub(crate) fn get_sym_names(&self) -> &Vec<Rc<str>> {
        &self.symbols.names
    }

    fn add_objects(&mut self, a: Obj, b: Obj) -> Result<()> {
        match (&a.borrow().payload, &b.borrow().payload) {
            (Payload::String(a), Payload::String(b)) => {
                let value = self.new_string(&[a.as_ref(), b.as_ref()].concat());
                self.poke(0, value)
            }
            _ => {
                self.pop();
                Err(RuntimeError::new(
                    "operands must be numbers or strings".to_string(),
                ))
            }
        }
    }

    fn push(&mut self, val: Value) -> Result<()> {
        if self.stack.len() < Vm::MAX_STACK {
            self.stack.push(val);
            Ok(())
        } else {
            Vm::error("stack overflow")
        }
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
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

    #[cfg(feature = "trace_execution")]
    fn trace_stack(&self) {
        print!("          ");
        for elem in &self.stack {
            print!("[ {} ]", elem);
        }
        println!();
    }
}

struct SymTable {
    symbols: HashMap<Rc<str>, u32>,
    names: Vec<Rc<str>>,
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

#[derive(Debug, thiserror::Error)]
#[error("{}", .msg)]
pub struct RuntimeError {
    msg: String,
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
