use std::{cell::RefCell, rc::Rc};

use super::interpret;

type Stdout = Rc<RefCell<Vec<u8>>>;
type Stderr = Rc<RefCell<Vec<u8>>>;

#[test]
fn associativity() {
    let source = r#"
    var a = "a";
    var b = "b";
    var c = "c";
    a = b = c;
    print a; // expect: c
    print b; // expect: c
    print c; // expect: c
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "c\nc\nc\n");
    assert_eq!(stderr, "");
}

#[test]
fn global() {
    let source = r#"
    var a = "before";
    print a; // expect: before

    a = "after";
    print a; // expect: after

    print a = "arg"; // expect: arg
    print a; // expect: arg
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "before\nafter\narg\narg\n");
    assert_eq!(stderr, "");
}

#[test]
fn grouping() {
    let source = r#"
    var a = "a";
    (a) = "value"; // Error at '=': Invalid assignment target.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 3] Error at '=': invalid assignment target\n");
}

#[test]
fn infix_operator() {
    let source = r#"
    var a = "a";
    var b = "b";
    a + b = "value"; // Error at '=': Invalid assignment target.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 4] Error at '=': invalid assignment target\n");
}

#[test]
fn local() {
    let source = r#"
    {
        var a = "before";
        print a; // expect: before
      
        a = "after";
        print a; // expect: after
      
        print a = "arg"; // expect: arg
        print a; // expect: arg
    }
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "before\nafter\narg\narg\n");
    assert_eq!(stderr, "");
}

#[test]
fn prefix_operator() {
    let source = r#"
    var a = "a";
    !a = "value"; // Error at '=': Invalid assignment target
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 3] Error at '=': invalid assignment target\n");
}

#[test]
fn syntax() {
    let source = r#"
    // Assignment on RHS of variable.
    var a = "before";
    var c = a = "var";
    print a; // expect: var
    print c; // expect: var
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "var\nvar\n");
    assert_eq!(stderr, "");
}

#[test]
fn to_this() {
    panic!();
}

#[test]
fn undefined() {
    let source = r#"
    unknown = "what"; // expect runtime error: Undefined variable 'unknown'.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] undefined variable 'unknown'\n");
}
