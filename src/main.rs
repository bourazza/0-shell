mod commands;
mod parser;
mod shell;
mod utils;

use parser::Command;
use shell::Shell;
use std::env;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use utils::*;

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

fn main() {
    println!("\x1b[1;32m0-shell\x1b[0m — type \x1b[1mhelp\x1b[0m for available commands\n");

    let mut shell = Shell::new();
    welcom::welcom();

    ctrlc::set_handler(|| {
        INTERRUPTED.store(true, Ordering::SeqCst);
    })
    .expect("failed to install Ctrl+C handler");

    loop {
        if INTERRUPTED.swap(false, Ordering::SeqCst) {
            println!("\n");
        }

        let path = env::current_dir().unwrap_or_default();
        let display = path.display().to_string();

        // Colorized prompt: show path in cyan, $ in bold white
        print!("\x1b[1;36m{}\x1b[0m \x1b[1;37m$\x1b[0m ", display);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        let bytes_read = io::stdin().read_line(&mut input).unwrap();

        if bytes_read == 0 {
            println!("\nExiting shell. Bye!");
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Support command chaining with `;`
        let segments: Vec<&str> = input.split(';').collect();

        for segment in segments {
            let segment = segment.trim();
            if segment.is_empty() {
                continue;
            }

            let (cmd, args) = match parser::parsing(segment) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("\x1b[31mError: {}\x1b[0m", e);
                    continue;
                }
            };

            match cmd {
                Command::Exit => {
                    println!("Exiting shell. Bye!");
                    std::process::exit(0);
                }
                Command::Unknown(name) => {
                    eprintln!("\x1b[31mCommand '{}' not found\x1b[0m", name);
                }
                _ => {
                    if let Err(e) = shell.execute(cmd, args) {
                        eprintln!("\x1b[31m{}\x1b[0m", e);
                    }
                }
            }
        }
    }
}
