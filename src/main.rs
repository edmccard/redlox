use std::cell::RefCell;
use std::io::{stdin, stdout, BufRead, Write};
use std::process::exit;
use std::rc::Rc;
use std::{env, io};

use anyhow::Result;

use redlox::Vm;

fn main() -> Result<()> {
    let stdout = Rc::new(RefCell::new(io::stdout()));
    let stderr = Rc::new(RefCell::new(io::stderr()));
    let mut vm = Vm::new(stdout, stderr);
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => repl(&mut vm)?,
        2 => {
            let source = std::fs::read_to_string(&args[1])?;
            vm.interpret(source)?;
        }
        _ => {
            eprintln!("Usage: rlox [path]");
            exit(1);
        }
    }
    Ok(())
}

fn repl(vm: &mut Vm) -> Result<()> {
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
            if let Err(e) = vm.interpret(source.join("\n")) {
                eprintln!("{}", e)
            }
            source.clear();
        }
    }
    Ok(())
}
