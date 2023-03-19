use super::interpret;

#[test]
fn line_at_eof() {
    let source = r#"
    print "ok"; // expect: ok
    // comment
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "ok\n");
    assert_eq!(stderr, "");
}

#[test]
fn only_line_comment_and_line() {
    let source = r#"// comment
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "");
}

#[test]
fn only_line_comment() {
    let source = r#"// comment"#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "");
}

#[test]
fn unicode() {
    let source = r#"
    // Unicode characters are allowed in comments.
    //
    // Latin 1 Supplement: £§¶ÜÞ
    // Latin Extended-A: ĐĦŋœ
    // Latin Extended-B: ƂƢƩǁ
    // Other stuff: ឃᢆ᯽₪ℜ↩⊗┺░
    // Emoji: ☃☺♣

    print "ok"; // expect: ok
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "ok\n");
    assert_eq!(stderr, "");
}
