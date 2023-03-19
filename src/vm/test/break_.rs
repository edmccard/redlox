use super::interpret;

#[test]
fn in_for() {
    let source = r#"
    for (;;) break;

    for (var a = 0; a < 10; a = a + 1) {
        if (a == 2) break;
        print a;
    }

    for (var a = 0; a < 3; a = a + 1) {
        for (;;) break;
        print a;
    }
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "0\n1\n0\n1\n2\n");
    assert_eq!(stderr, "");
}

#[test]
fn in_while() {
    let source = r#"
    while (true) break;

    var a = 0;
    while (a < 3) {
        if (a == 3) break;
        if (a == 2) break;
        print a;
        a = a + 1;
    }

    a = 0;
    var b;
    while (a < 3) {
        b = a;
        while (b < 3) break;
        print b;
        a = a + 1;
    }
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "0\n1\n0\n1\n2\n");
    assert_eq!(stderr, "");
}

#[test]
fn outside_loop() {
    let source = r#"
    if (true) break;
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] Error at ';': 'break' outside of loop\n");
}

// #[test]
// fn template() {
//     let source = r#"

//     "#;

//     let (stdout, stderr) = interpret(source);
//     assert_eq!(stdout, "");
//     assert_eq!(stderr, "");
// }
