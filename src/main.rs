use std::io::{self, BufRead, Write};

use clap::Parser;

#[derive(Parser)]
#[command(name = "cczr", about = "Fast log colorizer — Rust port of CCZE")]
struct Cli {
    /// Disable ANSI color output (plain text pass-through)
    #[arg(short = 'n', long = "no-color")]
    no_color: bool,

    /// List available plugins and exit
    #[arg(long = "list-plugins")]
    list_plugins: bool,
}

fn main() {
    let cli = Cli::parse();

    let plugins = cczr::default_plugins();

    if cli.list_plugins {
        for p in &plugins {
            println!("{:10}  {:?}", p.name(), p.kind());
        }
        return;
    }

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if cli.no_color {
            writeln!(out, "{}", line).ok();
        } else {
            writeln!(out, "{}", cczr::colorize_line(&line, &plugins)).ok();
        }
    }
}
