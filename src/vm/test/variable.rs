use super::interpret;

#[test]
fn collide_with_parameter() {
    let source = r#"
    fun foo(a) {
        var a; // Error at 'a': Already a variable with this name in this scope.
      }
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 3] Error at 'a': already a variable with this name in this scope\n");
}

#[test]
fn duplicate_local() {
    let source = r#"
    {
        var a = "value";
        var a = "other"; // Error at 'a': Already a variable with this name in this scope.
    }
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 4] Error at 'a': already a variable with this name in this scope\n");
}

#[test]
fn duplicate_parameter() {
    let source = r#"
    fun foo(arg,
            arg) { // Error at 'arg': Already a variable with this name in this scope.
        "body";
    }
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 3] Error at 'arg': already a parameter with this name in this scope\n");
}

#[test]
fn early_bound() {
    let source = r#"
    var a = "outer";
    {
      fun foo() {
        print a;
      }
    
      foo(); // expect: outer
      var a = "inner";
      foo(); // expect: outer
    }
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "outer\nouter\n");
    assert_eq!(stderr, "");
}

#[test]
fn in_middle_of_block() {
    let source = r#"
    {
        var a = "a";
        print a; // expect: a
        var b = a + " b";
        print b; // expect: a b
        var c = a + " c";
        print c; // expect: a c
        var d = b + " d";
        print d; // expect: a b d
    }
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "a\na b\na c\na b d\n");
    assert_eq!(stderr, "");
}

#[test]
fn in_nested_block() {
    let source = r#"
    {
        var a = "outer";
        {
          print a; // expect: outer
        }
    }
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "outer\n");
    assert_eq!(stderr, "");
}

#[test]
fn local_from_method() {
    panic!();
}

#[test]
fn redeclare_global() {
    let source = r#"
    var a = "1";
    var a;
    print a; // expect: nil
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "nil\n");
    assert_eq!(stderr, "");
}

#[test]
fn redefine_global() {
    let source = r#"
    var a = "1";
    var a = "2";
    print a; // expect: 2
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "2\n");
    assert_eq!(stderr, "");
}

#[test]
fn scope_reuse_in_different_blocks() {
    let source = r#"
    {
        var a = "first";
        print a; // expect: first
      }
      
      {
        var a = "second";
        print a; // expect: second
    }
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "first\nsecond\n");
    assert_eq!(stderr, "");
}

#[test]
fn shadow_and_local() {
    let source = r#"
    {
        var a = "outer";
        {
          print a; // expect: outer
          var a = "inner";
          print a; // expect: inner
        }
    }
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "outer\ninner\n");
    assert_eq!(stderr, "");
}

#[test]
fn shadow_global() {
    let source = r#"
    var a = "global";
    {
        var a = "shadow";
        print a; // expect: shadow
    }
    print a; // expect: global
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "shadow\nglobal\n");
    assert_eq!(stderr, "");
}

#[test]
fn shadow_local() {
    let source = r#"
    {
        var a = "local";
        {
          var a = "shadow";
          print a; // expect: shadow
        }
        print a; // expect: local
    }
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "shadow\nlocal\n");
    assert_eq!(stderr, "");
}

#[test]
fn undefined_global() {
    let source = r#"
    print notDefined;  // expect runtime error: Undefined variable 'notDefined'.
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 2] undefined variable 'notDefined'\n");
}

#[test]
fn undefined_local() {
    let source = r#"
    {
        print notDefined;  // expect runtime error: Undefined variable 'notDefined'.
    }
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 3] undefined variable 'notDefined'\n");
}

#[test]
fn uninitialized() {
    let source = r#"
    var a;
    print a; // expect: nil
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "nil\n");
    assert_eq!(stderr, "");
}

#[test]
fn unreached_undefined() {
    let source = r#"
    if (false) {
        print notDefined;
    }
      
    print "ok"; // expect: ok
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "ok\n");
    assert_eq!(stderr, "");
}

#[test]
fn use_false_as_var() {
    let source = r#"
    // [line 2] Error at 'false': Expect variable name.
    var false = "value";
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 3] Error at 'false': expect variable name\n");
}

#[test]
fn use_global_in_initializer() {
    let source = r#"
    var a = "value";
    var a = a;
    print a; // expect: value
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "value\n");
    assert_eq!(stderr, "");
}

#[test]
fn use_local_in_initializer() {
    let source = r#"
    var a = "outer";
    {
      var a = a; // Error at 'a': Can't read local variable in its own initializer.
    }
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 4] Error at 'a': can't read local variable in its own initializer\n");
}

#[test]
fn use_nil_as_var() {
    let source = r#"
    // [line 2] Error at 'nil': Expect variable name.
    var nil = "value";
    "#;

    let (stdout, stderr) = interpret(source);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[line 3] Error at 'nil': expect variable name\n");
}

#[test]
fn use_this_as_var() {
    panic!();
}
