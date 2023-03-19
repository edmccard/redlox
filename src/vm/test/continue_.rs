use super::interpret;

#[test]
fn outside_loop() {
    let source = r#"
    if (true) continue;
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(
        stderr,
        "[line 2] Error at ';': 'continue' outside of loop\n"
    );
}

#[test]
fn in_while() {
    let source = r#"
    var a = 0;
    while (a < 3) {
        if (a == 1) {
             a = 3;
             continue;
        }
        print a;
        a = a + 1;
    } // expect 0
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "0\n");
    assert_eq!(stderr, "");
}

#[test]
fn in_for() {
    let source = r#"
    for (var a = 0; a < 4; a = a + 1) continue;

    for (var a = 0; a < 4; a = a + 1) {
        if (a == 2) continue;
        print a;
    } // expect 0 1 3

    for (var a = 0; a < 4; a = a + 1) {
        print a;
        for (var b = 0; b < 10; b = b + 1) continue;
    }
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "0\n1\n3\n0\n1\n2\n3\n");
    assert_eq!(stderr, "");
}
