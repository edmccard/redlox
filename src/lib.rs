use std::{
    cell::RefCell,
    fmt::{self, Display},
    io::Write,
    ops::Deref,
    rc::Rc,
};

use vm::{LoxFunction, LoxString};

pub use parser::print_tokens;
pub use parser::scanner::bench_scanner;
pub use vm::Vm;

mod code;
mod parser;
mod vm;

struct Obj<T>(Rc<RefCell<T>>);

#[derive(Clone, PartialEq)]
enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(Obj<LoxString>),
    Function(Obj<LoxFunction>),
}

pub type Stdout = Rc<RefCell<dyn Write>>;
pub type Stderr = Rc<RefCell<dyn Write>>;

impl<T> Clone for Obj<T> {
    fn clone(&self) -> Self {
        Obj(self.0.clone())
    }
}

impl<T> Deref for Obj<T> {
    type Target = Rc<RefCell<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<LoxFunction> for Obj<LoxFunction> {
    fn from(value: LoxFunction) -> Self {
        Obj(Rc::new(RefCell::new(value)))
    }
}

impl From<LoxString> for Obj<LoxString> {
    fn from(value: LoxString) -> Self {
        Obj(Rc::new(RefCell::new(value)))
    }
}

impl PartialEq for Obj<LoxFunction> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(self, other)
    }
}

impl PartialEq for Obj<LoxString> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
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
            Value::Function(v) => v.borrow().fmt(f),
        }
    }
}

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        !matches!(value, Value::Nil | Value::Boolean(false))
    }
}
