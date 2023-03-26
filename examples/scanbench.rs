use std::process::exit;

use anyhow::Result;

use redlox::bench_scanner;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: scanbench [path]");
        exit(1);
    }
    let text = std::fs::read_to_string(&args[1])?;
    bench_scanner(text)?;

    Ok(())
}
