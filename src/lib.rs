use std::{
    cell::RefCell,
    fmt::{self, Display},
    io::Write,
    ops::Deref,
    rc::Rc,
};

use vm::LoxString;

pub use parser::print_tokens;
pub use parser::scanner::bench_scanner;
pub use vm::Vm;

mod code;
mod parser;
mod vm;

#[derive(Clone)]
struct Obj<T>(Rc<RefCell<T>>);

#[derive(Clone, PartialEq)]
enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(Obj<LoxString>),
}

pub type Stdout = Rc<RefCell<dyn Write>>;
pub type Stderr = Rc<RefCell<dyn Write>>;

impl<T> Deref for Obj<T> {
    type Target = Rc<RefCell<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> PartialEq for Obj<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(self, other)
    }
}

impl Value {
    const TRUE: Value = Value::Boolean(true);
    const FALSE: Value = Value::Boolean(false);
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(v) => v.fmt(f),
            Value::Number(v) => v.fmt(f),
            Value::String(v) => v.borrow().fmt(f),
        }
    }
}

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        !matches!(value, Value::Nil | Value::Boolean(false))
    }
}
