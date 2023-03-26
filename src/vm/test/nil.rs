use super::interpret;

#[test]
fn literal() {
    let source = r#"
    print nil; // expect nil
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "nil\n");
    assert_eq!(stderr, "");
}
