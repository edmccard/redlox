use std::{
    cell::RefCell,
    fmt::{self, Display},
    io::Write,
    rc::Rc,
};

pub use parser::print_tokens;
pub use parser::scanner::bench_scanner;
use vm::Obj;
pub use vm::Vm;

mod code;
mod parser;
mod vm;

pub type Stdout = Rc<RefCell<dyn Write>>;
pub type Stderr = Rc<RefCell<dyn Write>>;

#[derive(Clone, PartialEq)]
enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    Object(Obj),
}

impl Value {
    const TRUE: Value = Value::Boolean(true);
    const FALSE: Value = Value::Boolean(false);
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(v) => write!(f, "{}", v),
            Value::Number(v) => write!(f, "{}", v),
            Value::Object(v) => write!(f, "{}", v.borrow()),
        }
    }
}

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        !matches!(value, Value::Nil | Value::Boolean(false))
    }
}
