use std::io::Write;
use std::{cell::RefCell, rc::Rc};

use crate::Vm;

fn interpret(source: &str) -> (String, String) {
    let stdout = Rc::new(RefCell::new(Vec::<u8>::new()));
    let stderr = Rc::new(RefCell::new(Vec::<u8>::new()));
    let mut vm = Vm::new(stdout.clone(), stderr.clone());
    if let Err(e) = vm.interpret(source.to_string()) {
        let _ = writeln!(stderr.borrow_mut(), "{}", e);
    }
    let ret = (
        String::from_utf8(stdout.borrow().to_vec()).unwrap(),
        String::from_utf8(stderr.borrow().to_vec()).unwrap(),
    );
    ret
}

#[test]
fn empty_file() {
    let source = r#""#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "");
}

#[test]
fn precedence() {
    let source = r#"
    // * has higher precedence than +.
    print 2 + 3 * 4; // expect: 14

    // * has higher precedence than -.
    print 20 - 3 * 4; // expect: 8

    // / has higher precedence than +.
    print 2 + 6 / 3; // expect: 4

    // / has higher precedence than -.
    print 2 - 6 / 3; // expect: 0

    // < has higher precedence than ==.
    print false == 2 < 1; // expect: true

    // > has higher precedence than ==.
    print false == 1 > 2; // expect: true

    // <= has higher precedence than ==.
    print false == 2 <= 1; // expect: true

    // >= has higher precedence than ==.
    print false == 1 >= 2; // expect: true

    // 1 - 1 is not space-sensitive.
    print 1 - 1; // expect: 0
    print 1 -1;  // expect: 0
    print 1- 1;  // expect: 0
    print 1-1;   // expect: 0

    // Using () for grouping.
    print (2 * (6 - (2 + 2))); // expect: 4
    "#;

    let expected = [
        "14", "8", "4", "0", "true", "true", "true", "true", "0", "0", "0",
        "0", "4", "",
    ];

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, expected.join("\n"));
    assert_eq!(stderr, "");
}

mod assignment;
mod block;
mod bool;
mod comments;
mod for_;
mod logical_operator;
mod numbers;
mod operator;
mod print;
mod string;
mod variable;
mod while_;
