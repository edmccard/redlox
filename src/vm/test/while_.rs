use super::interpret;

#[test]
fn class_in_body() {
    panic!();
}

#[test]
fn closure_in_body() {
    panic!();
}

#[test]
fn fun_in_body() {
    let source = r#"
    // [line 2] Error at 'fun': Expect expression.
    while (true) fun foo() {}
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 3] Error at 'fun': expect expression\n");
}

#[test]
fn return_closure() {
    panic!();
}

#[test]
fn return_inside() {
    let source = r#"
    fun f() {
        while (true) {
          var i = "i";
          return i;
        }
      }
      
    print f();
    // expect: i
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "i\n");
    assert_eq!(stderr, "");
}

#[test]
fn syntax() {
    let source = r#"
    // Single-expression body.
    var c = 0;
    while (c < 3) print c = c + 1;
    // expect: 1
    // expect: 2
    // expect: 3
    
    // Block body.
    var a = 0;
    while (a < 3) {
      print a;
      a = a + 1;
    }
    // expect: 0
    // expect: 1
    // expect: 2
    
    // Statement bodies.
    while (false) if (true) 1; else 2;
    while (false) while (true) 1;
    while (false) for (;;) 1;
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "1\n2\n3\n0\n1\n2\n");
    assert_eq!(stderr, "");
}

#[test]
fn var_in_body() {
    let source = r#"
    // [line 2] Error at 'var': Expect expression.
    while (true) var foo;
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 3] Error at 'var': expect expression\n");
}
