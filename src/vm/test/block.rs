use super::interpret;

#[test]
fn block_empty() {
    let source = r#"
    {} // By itself.

    // In a statement.
    if (true) {}
    if (false) {} else {}

    print "ok"; // expect: ok
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "ok\n");
    assert_eq!(stderr, "");
}

#[test]
fn scope() {
    let source = r#"
    var a = "outer";

    {
        var a = "inner";
        print a; // expect: inner
    }

    print a; // expect: outer
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "inner\nouter\n");
    assert_eq!(stderr, "");
}
