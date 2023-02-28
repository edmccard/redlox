use std::process::exit;

use rlox::{Parser, Result};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: scanbench [path]");
        exit(1);
    }
    let source = std::fs::read_to_string(&args[1])?;
    let mut compiler = Parser::new(source);
    compiler.bench()?;
    Ok(())
}
