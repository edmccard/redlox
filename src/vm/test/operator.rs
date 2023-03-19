use super::interpret;

#[test]
fn add_bool_nil() {
    let source = r#"
    true + nil; // expect runtime error: Operands must be two numbers or two strings.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers or strings\n");
}

#[test]
fn add_bool_num() {
    let source = r#"
    true + 123; // expect runtime error: Operands must be two numbers or two strings.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers or strings\n");
}

#[test]
fn add_bool_string() {
    let source = r#"
    true + "s"; // expect runtime error: Operands must be two numbers or two strings.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers or strings\n");
}

#[test]
fn add() {
    let source = r#"
    print 123 + 456; // expect: 579
    print "str" + "ing"; // expect: string
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "579\nstring\n");
    assert_eq!(stderr, "");
}

#[test]
fn add_nil_nil() {
    let source = r#"
    nil + nil; // expect runtime error: Operands must be two numbers or two strings.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers or strings\n");
}

#[test]
fn add_num_nil() {
    let source = r#"
    1 + nil; // expect runtime error: Operands must be two numbers or two strings.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers or strings\n");
}

#[test]
fn add_string_nil() {
    let source = r#"
    "s" + nil; // expect runtime error: Operands must be two numbers or two strings.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers or strings\n");
}

#[test]
fn comparison() {
    let source = r#"
    print 1 < 2;    // expect: true
    print 2 < 2;    // expect: false
    print 2 < 1;    // expect: false

    print 1 <= 2;    // expect: true
    print 2 <= 2;    // expect: true
    print 2 <= 1;    // expect: false

    print 1 > 2;    // expect: false
    print 2 > 2;    // expect: false
    print 2 > 1;    // expect: true

    print 1 >= 2;    // expect: false
    print 2 >= 2;    // expect: true
    print 2 >= 1;    // expect: true

    // Zero and negative zero compare the same.
    print 0 < -0; // expect: false
    print -0 < 0; // expect: false
    print 0 > -0; // expect: false
    print -0 > 0; // expect: false
    print 0 <= -0; // expect: true
    print -0 <= 0; // expect: true
    print 0 >= -0; // expect: true
    print -0 >= 0; // expect: true
    "#;

    let expected = [
        "true", "false", "false", "true", "true", "false", "false", "false",
        "true", "false", "true", "true", "false", "false", "false", "false",
        "true", "true", "true", "true", "",
    ];

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, expected.join("\n"));
    assert_eq!(stderr, "");
}

#[test]
fn divide() {
    let source = r#"
    print 8 / 2;         // expect: 4
    print 12.34 / 12.34;  // expect: 1
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "4\n1\n");
    assert_eq!(stderr, "");
}

#[test]
fn divide_nonnum_num() {
    let source = r#"
    "1" / 1; // expect runtime error: Operands must be numbers.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers\n");
}

fn divide_num_nonnum() {
    let source = r#"
    1 / "1"; // expect runtime error: Operands must be numbers.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers\n");
}

#[test]
fn equals_class() {
    panic!();
}

#[test]
fn equals() {
    let source = r#"
    print nil == nil; // expect: true

    print true == true; // expect: true
    print true == false; // expect: false

    print 1 == 1; // expect: true
    print 1 == 2; // expect: false

    print "str" == "str"; // expect: true
    print "str" == "ing"; // expect: false

    print nil == false; // expect: false
    print false == 0; // expect: false
    print 0 == "0"; // expect: false
    "#;

    let expected = [
        "true", "true", "false", "true", "false", "true", "false", "false",
        "false", "false", "",
    ];

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, expected.join("\n"));
    assert_eq!(stderr, "");
}

#[test]
fn equals_method() {
    panic!();
}

#[test]
fn greater_nonnum_num() {
    let source = r#"
    "1" > 1; // expect runtime error: Operands must be numbers.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers\n");
}

#[test]
fn greater_num_nonnum() {
    let source = r#"
    1 > "1"; // expect runtime error: Operands must be numbers.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers\n");
}

#[test]
fn greater_or_equal_nonnum_num() {
    let source = r#"
    "1" >= 1; // expect runtime error: Operands must be numbers.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers\n");
}

#[test]
fn greater_or_equal_num_nonnum() {
    let source = r#"
    1 >= "1"; // expect runtime error: Operands must be numbers.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers\n");
}

#[test]
fn less_nonnum_num() {
    let source = r#"
    "1" < 1; // expect runtime error: Operands must be numbers.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers\n");
}

#[test]
fn less_num_nonnum() {
    let source = r#"
    1 < "1"; // expect runtime error: Operands must be numbers.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers\n");
}

#[test]
fn less_or_equal_nonnum_num() {
    let source = r#"
    "1" <= 1; // expect runtime error: Operands must be numbers.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers\n");
}

#[test]
fn less_or_equal_num_nonnum() {
    let source = r#"
    1 <= "1"; // expect runtime error: Operands must be numbers.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers\n");
}

#[test]
fn multiply() {
    let source = r#"
    print 5 * 3; // expect: 15
    print 12.34 * 0.3; // expect: 3.702
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "15\n3.702\n");
    assert_eq!(stderr, "");
}

#[test]
fn multiply_nonnum_num() {
    let source = r#"
    "1" * 1; // expect runtime error: Operands must be numbers.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers\n");
}

#[test]
fn multiply_num_nonnum() {
    let source = r#"
    1 * "1"; // expect runtime error: Operands must be numbers.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers\n");
}

#[test]
fn negate() {
    let source = r#"
    print -(3); // expect: -3
    print --(3); // expect: 3
    print ---(3); // expect: -3
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "-3\n3\n-3\n");
    assert_eq!(stderr, "");
}

#[test]
fn negate_nonnum() {
    let source = r#"
    -"s"; // expect runtime error: Operand must be a number.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operand must be a number\n");
}

#[test]
fn not_class() {
    panic!();
}

#[test]
fn not_equals() {
    let source = r#"
    print nil != nil; // expect: false

    print true != true; // expect: false
    print true != false; // expect: true

    print 1 != 1; // expect: false
    print 1 != 2; // expect: true

    print "str" != "str"; // expect: false
    print "str" != "ing"; // expect: true

    print nil != false; // expect: true
    print false != 0; // expect: true
    print 0 != "0"; // expect: true
    "#;

    let expected = [
        "false", "false", "true", "false", "true", "false", "true", "true",
        "true", "true", "",
    ];

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, expected.join("\n"));
    assert_eq!(stderr, "");
}

#[test]
fn not() {
    panic!();
}

#[test]
fn subtract() {
    let source = r#"
    print 4 - 3; // expect: 1
    print 1.2 - 1.2; // expect: 0
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "1\n0\n");
    assert_eq!(stderr, "");
}

#[test]
fn subtract_nonnum_num() {
    let source = r#"
    "1" - 1; // expect runtime error: Operands must be numbers.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers\n");
}

#[test]
fn subtract_num_nonnum() {
    let source = r#"
    1 - "1"; // expect runtime error: Operands must be numbers.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] operands must be numbers\n");
}
