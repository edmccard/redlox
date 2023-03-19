use std::cell::RefCell;
use std::io::{stdin, stdout, BufRead, Write};
use std::process::exit;
use std::rc::Rc;
use std::{env, io};

use rlox::{Parser, Result};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 1 {
        eprintln!("Usage: scandemo");
        exit(1);
    }

    let mut lines = stdin().lock().lines();
    let mut line_no = 1;
    let mut source: Vec<String> = Vec::new();
    loop {
        print!("{:4}> ", line_no);
        stdout().flush()?;
        let mut line = match lines.next() {
            None => break,
            Some(line) => line?,
        };
        line_no += 1;
        if line.ends_with('\\') {
            line.pop();
            source.push(line);
            continue;
        } else {
            source.push(line);
            let mut parser = Parser::new(
                source.join("\n"),
                Rc::new(RefCell::new(io::stderr())),
            );
            parser.show_tokens();
            source.clear();
        }
    }

    Ok(())
}
