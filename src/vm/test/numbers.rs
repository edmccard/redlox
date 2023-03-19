use super::interpret;

#[test]
fn decimal_point_at_end() {
    panic!();
}

#[test]
fn leading_dot() {
    let source = r#"
    // [line 2] Error at '.': Expect expression.
    .123;
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 3] Error at '.': expect expression\n");
}

#[test]
fn literals() {
    let source = r#"
    print 123;     // expect: 123
    print 987654;  // expect: 987654
    print 0;       // expect: 0
    print -0;      // expect: -0

    print 123.456; // expect: 123.456
    print -0.001;  // expect: -0.001
    "#;

    let expected = ["123", "987654", "0", "-0", "123.456", "-0.001", ""];

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, expected.join("\n"));
    assert_eq!(stderr, "");
}

#[test]
fn nan_equality() {
    let source = r#"
    var nan = 0/0;

    print nan == 0; // expect: false
    print nan != 1; // expect: true

    // NaN is not equal to self.
    print nan == nan; // expect: false
    print nan != nan; // expect: true
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "false\ntrue\nfalse\ntrue\n");
    assert_eq!(stderr, "");
}

#[test]
fn trailing_dot() {
    panic!();
}
