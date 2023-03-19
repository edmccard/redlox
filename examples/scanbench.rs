use std::cell::RefCell;
use std::io;
use std::process::exit;
use std::rc::Rc;

use rlox::{Parser, Result};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: scanbench [path]");
        exit(1);
    }
    let source = std::fs::read_to_string(&args[1])?;
    let mut parser = Parser::new(source, Rc::new(RefCell::new(io::stderr())));
    #[cfg(feature = "bench_mode")]
    parser.bench()?;
    Ok(())
}
