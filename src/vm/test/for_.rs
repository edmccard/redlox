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
    panic!();
}

#[test]
fn return_closure() {
    panic!();
}

#[test]
fn return_inside() {
    panic!();
}

#[test]
fn scope() {
    let source = r#"
    {
        var i = "before";
      
        // New variable is in inner scope.
        for (var i = 0; i < 1; i = i + 1) {
          print i; // expect: 0
      
          // Loop body is in second inner scope.
          var i = -1;
          print i; // expect: -1
        }
      }
      
      {
        // New variable shadows outer variable.
        for (var i = 0; i > 0; i = i + 1) {}
      
        // Goes out of scope after loop.
        var i = "after";
        print i; // expect: after
      
        // Can reuse an existing variable.
        for (i = 0; i < 1; i = i + 1) {
          print i; // expect: 0
        }
      }      
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "0\n-1\nafter\n0\n");
    assert_eq!(stderr, "");
}

#[test]
fn statement_condition() {
    let source = r#"
    // [line 3] Error at '{': Expect expression.
    // [line 3] Error at ')': Expect ';' after expression.
    for (var a = 1; {}; a = a + 1) {}
    "#;

    let expected = [
        "[line 4] Error at '{': expect expression",
        "[line 4] Error at ')': expect ';' after expression",
        "",
    ];

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, expected.join("\n"));
}

#[test]
fn statement_increment() {
    let source = r#"
    // [line 2] Error at '{': Expect expression.
    for (var a = 1; a < 2; {}) {}
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 3] Error at '{': expect expression\n");
}

#[test]
fn statement_initializer() {
    let source = r#"
    // [line 3] Error at '{': Expect expression.
    // [line 3] Error at ')': Expect ';' after expression.
    for ({}; a < 2; a = a + 1) {}
    "#;

    let expected = [
        "[line 4] Error at '{': expect expression",
        "[line 4] Error at ')': expect ';' after expression",
        "",
    ];

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, expected.join("\n"));
}

#[test]
fn syntax() {
    let source = r#"
    // Single-expression body.
    for (var c = 0; c < 3;) print c = c + 1;
    // expect: 1
    // expect: 2
    // expect: 3
    
    // Block body.
    for (var a = 0; a < 3; a = a + 1) {
      print a;
    }
    // expect: 0
    // expect: 1
    // expect: 2
    
    // No clauses.
    //fun foo() {
    //  for (;;) return "done";
    //}
    //print foo(); // expect: done
    
    // No variable.
    var i = 0;
    for (; i < 2; i = i + 1) print i;
    // expect: 0
    // expect: 1
    
    // No condition.
    //fun bar() {
    //  for (var i = 0;; i = i + 1) {
    //    print i;
    //    if (i >= 2) return;
    //  }
    //}
    //bar();
    // expect: 0
    // expect: 1
    // expect: 2
    
    // No increment.
    for (var i = 0; i < 2;) {
      print i;
      i = i + 1;
    }
    // expect: 0
    // expect: 1
    
    // Statement bodies.
    for (; false;) if (true) 1; else 2;
    for (; false;) while (true) 1;
    for (; false;) for (;;) 1;    
    "#;

    let expected = ["1", "2", "3", "0", "1", "2", "0", "1", "0", "1", ""];

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, expected.join("\n"));
    assert_eq!(stderr, "");
}

#[test]
fn var_in_body() {
    let source = r#"
    // [line 2] Error at 'var': Expect expression.
    for (;;) var foo;
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 3] Error at 'var': expect expression\n");
}
