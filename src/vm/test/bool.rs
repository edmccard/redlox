use super::interpret;

#[test]
fn equality() {
    let source = r#"
    print true == true;    // expect: true
    print true == false;   // expect: false
    print false == true;   // expect: false
    print false == false;  // expect: true

    // Not equal to other types.
    print true == 1;        // expect: false
    print false == 0;       // expect: false
    print true == "true";   // expect: false
    print false == "false"; // expect: false
    print false == "";      // expect: false

    print true != true;    // expect: false
    print true != false;   // expect: true
    print false != true;   // expect: true
    print false != false;  // expect: false

    // Not equal to other types.
    print true != 1;        // expect: true
    print false != 0;       // expect: true
    print true != "true";   // expect: true
    print false != "false"; // expect: true
    print false != "";      // expect: true
    "#;

    let expected = [
        "true", "false", "false", "true", "false", "false", "false", "false",
        "false", "false", "true", "true", "false", "true", "true", "true",
        "true", "true", "",
    ];

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, expected.join("\n"));
    assert_eq!(stderr, "");
}

#[test]
fn not() {
    let source = r#"
    print !true;    // expect: false
    print !false;   // expect: true
    print !!true;   // expect: true
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "false\ntrue\ntrue\n");
    assert_eq!(stderr, "");
}
