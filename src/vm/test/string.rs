use super::interpret;

#[test]
fn error_after_multiline() {
    let source = r#"
    // Tests that we correctly track the line info across multiline strings.
    var a = "1
    2
    3
    ";
    
    err; // // expect runtime error: Undefined variable 'err'.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 8] undefined variable 'err'\n");
}

#[test]
fn literals() {
    let source = r#"
    print "(" + "" + ")";   // expect: ()
    print "a string"; // expect: a string

    // Non-ASCII.
    print "A~¶Þॐஃ"; // expect: A~¶Þ
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "()\na string\nA~¶Þॐஃ\n");
    assert_eq!(stderr, "");
}

#[test]
fn multiline() {
    let source = r#"
var a = "1
2
3";
    print a;
    // expect: 1
    // expect: 2
    // expect: 3
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "1\n2\n3\n");
    assert_eq!(stderr, "");
}

#[test]
fn unterminated() {
    let source = r#"
    // [line 2] Error: Unterminated string.
    "this string has no close quote
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 3] Error: unterminated string\n");
}
