use super::interpret;

#[test]
fn missing_argument() {
    let source = r#"
    print;
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] Error at ';': expect expression\n");
}
