use anyhow::Result;

use super::{Scanner, TokenType};

#[test]
fn identifiers() -> Result<()> {
    let source = r#"
    andy formless fo _ _123 _abc ab123
    abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_
    "#;
    let mut scanner = Scanner::new(source.into());

    assert_eq!((TokenType::Identifier, "andy"), tok(&mut scanner)?);
    assert_eq!((TokenType::Identifier, "formless"), tok(&mut scanner)?);
    assert_eq!((TokenType::Identifier, "fo"), tok(&mut scanner)?);
    assert_eq!((TokenType::Identifier, "_"), tok(&mut scanner)?);
    assert_eq!((TokenType::Identifier, "_123"), tok(&mut scanner)?);
    assert_eq!((TokenType::Identifier, "_abc"), tok(&mut scanner)?);
    assert_eq!((TokenType::Identifier, "ab123"), tok(&mut scanner)?);
    assert_eq!(
        (
            TokenType::Identifier,
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_"
        ),
        tok(&mut scanner)?
    );
    assert_eq!(TokenType::Eof, tok(&mut scanner)?.0);

    Ok(())
}

#[test]
fn keywords() -> Result<()> {
    let source = r#"
    and class else false for fun if nil or return super this true var while
    "#;
    let mut scanner = Scanner::new(source.into());

    assert_eq!((TokenType::And, "and"), tok(&mut scanner)?);
    assert_eq!((TokenType::Class, "class"), tok(&mut scanner)?);
    assert_eq!((TokenType::Else, "else"), tok(&mut scanner)?);
    assert_eq!((TokenType::False, "false"), tok(&mut scanner)?);
    assert_eq!((TokenType::For, "for"), tok(&mut scanner)?);
    assert_eq!((TokenType::Fun, "fun"), tok(&mut scanner)?);
    assert_eq!((TokenType::If, "if"), tok(&mut scanner)?);
    assert_eq!((TokenType::Nil, "nil"), tok(&mut scanner)?);
    assert_eq!((TokenType::Or, "or"), tok(&mut scanner)?);
    assert_eq!((TokenType::Return, "return"), tok(&mut scanner)?);
    assert_eq!((TokenType::Super, "super"), tok(&mut scanner)?);
    assert_eq!((TokenType::This, "this"), tok(&mut scanner)?);
    assert_eq!((TokenType::True, "true"), tok(&mut scanner)?);
    assert_eq!((TokenType::Var, "var"), tok(&mut scanner)?);
    assert_eq!((TokenType::While, "while"), tok(&mut scanner)?);
    assert_eq!(TokenType::Eof, tok(&mut scanner)?.0);

    Ok(())
}

#[test]
fn numbers() -> Result<()> {
    let source = r#"
    123
    123.456
    .456
    123.
    "#;
    let mut scanner = Scanner::new(source.into());

    assert_eq!((TokenType::Number, "123"), tok(&mut scanner)?);
    assert_eq!((TokenType::Number, "123.456"), tok(&mut scanner)?);
    assert_eq!((TokenType::Dot, "."), tok(&mut scanner)?);
    assert_eq!((TokenType::Number, "456"), tok(&mut scanner)?);
    assert_eq!((TokenType::Number, "123"), tok(&mut scanner)?);
    assert_eq!((TokenType::Dot, "."), tok(&mut scanner)?);
    assert_eq!(TokenType::Eof, tok(&mut scanner)?.0);

    Ok(())
}

#[test]
fn punctuators() -> Result<()> {
    let source = r#"(){};,+-*!===<=>=!=<>/."#;
    let mut scanner = Scanner::new(source.into());

    assert_eq!((TokenType::LeftParen, "("), tok(&mut scanner)?);
    assert_eq!((TokenType::RightParen, ")"), tok(&mut scanner)?);
    assert_eq!((TokenType::LeftBrace, "{"), tok(&mut scanner)?);
    assert_eq!((TokenType::RightBrace, "}"), tok(&mut scanner)?);
    assert_eq!((TokenType::Semicolon, ";"), tok(&mut scanner)?);
    assert_eq!((TokenType::Comma, ","), tok(&mut scanner)?);
    assert_eq!((TokenType::Plus, "+"), tok(&mut scanner)?);
    assert_eq!((TokenType::Minus, "-"), tok(&mut scanner)?);
    assert_eq!((TokenType::Star, "*"), tok(&mut scanner)?);
    assert_eq!((TokenType::BangEqual, "!="), tok(&mut scanner)?);
    assert_eq!((TokenType::EqualEqual, "=="), tok(&mut scanner)?);
    assert_eq!((TokenType::LessEqual, "<="), tok(&mut scanner)?);
    assert_eq!((TokenType::GreaterEqual, ">="), tok(&mut scanner)?);
    assert_eq!((TokenType::BangEqual, "!="), tok(&mut scanner)?);
    assert_eq!((TokenType::Less, "<"), tok(&mut scanner)?);
    assert_eq!((TokenType::Greater, ">"), tok(&mut scanner)?);
    assert_eq!((TokenType::Slash, "/"), tok(&mut scanner)?);
    assert_eq!((TokenType::Dot, "."), tok(&mut scanner)?);
    assert_eq!(TokenType::Eof, tok(&mut scanner)?.0);

    Ok(())
}

#[test]
fn strings() -> Result<()> {
    let source = r#"
    ""
    "string"
    "#;
    let mut scanner = Scanner::new(source.into());

    assert_eq!((TokenType::String, r#""""#), tok(&mut scanner)?);
    assert_eq!((TokenType::String, r#""string""#), tok(&mut scanner)?);
    assert_eq!(TokenType::Eof, tok(&mut scanner)?.0);

    Ok(())
}

fn whitespace() -> Result<()> {
    let source = r#"
    space    tabs				newlines

    // a comment


    end
    "#;
    let mut scanner = Scanner::new(source.into());

    assert_eq!((TokenType::Identifier, "space"), tok(&mut scanner)?);
    assert_eq!((TokenType::Identifier, "tabs"), tok(&mut scanner)?);
    assert_eq!((TokenType::Identifier, "newlines"), tok(&mut scanner)?);
    assert_eq!((TokenType::Identifier, "end"), tok(&mut scanner)?);
    assert_eq!(TokenType::Eof, tok(&mut scanner)?.0);

    Ok(())
}

fn tok(scanner: &mut Scanner) -> Result<(TokenType, &str)> {
    let token = scanner.scan_token()?;
    Ok((token.ty(), scanner.token_text(token)))
}
