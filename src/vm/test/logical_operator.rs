use super::interpret;

#[test]
fn and() {
    let source = r#"
    // Note: These tests implicitly depend on ints being truthy.

    // Return the first non-true argument.
    print false and 1; // expect: false
    print true and 1; // expect: 1
    print 1 and 2 and false; // expect: false

    // Return the last argument if all are true.
    print 1 and true; // expect: true
    print 1 and 2 and 3; // expect: 3

    // Short-circuit at the first false argument.
    var a = "before";
    var b = "before";
    (a = true) and
        (b = false) and
        (a = "bad");
    print a; // expect: true
    print b; // expect: false
    "#;

    let expected = ["false", "1", "false", "true", "3", "true", "false", ""];

    let (stdout, stderr) = interpret(source);
    //assert_eq!(stdout, expected.join("\n"));
    assert_eq!(stderr, "");
}

#[test]
fn and_truth() {
    let source = r#"
    // False and nil are false.
    print false and "bad"; // expect: false
    print nil and "bad"; // expect: nil
    
    // Everything else is true.
    print true and "ok"; // expect: ok
    print 0 and "ok"; // expect: ok
    print "" and "ok"; // expect: ok
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "false\nnil\nok\nok\nok\n");
    assert_eq!(stderr, "");
}

#[test]
fn or() {
    let source = r#"
    // Note: These tests implicitly depend on ints being truthy.

    // Return the first true argument.
    print 1 or true; // expect: 1
    print false or 1; // expect: 1
    print false or false or true; // expect: true
    
    // Return the last argument if all are false.
    print false or false; // expect: false
    print false or false or false; // expect: false
    
    // Short-circuit at the first true argument.
    var a = "before";
    var b = "before";
    (a = false) or
        (b = true) or
        (a = "bad");
    print a; // expect: false
    print b; // expect: true
    "#;

    let expected = ["1", "1", "true", "false", "false", "false", "true", ""];

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, expected.join("\n"));
    assert_eq!(stderr, "");
}

#[test]
fn or_truth() {
    let source = r#"
    // False and nil are false.
    print false or "ok"; // expect: ok
    print nil or "ok"; // expect: ok
    
    // Everything else is true.
    print true or "ok"; // expect: true
    print 0 or "ok"; // expect: 0
    print "s" or "ok"; // expect: s
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "ok\nok\ntrue\n0\ns\n");
    assert_eq!(stderr, "");
}
