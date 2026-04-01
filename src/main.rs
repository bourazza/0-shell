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

fn read_line_with_prompt(prompt: &str) -> io::Result<Option<String>> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    let bytes_read = io::stdin().read_line(&mut input)?;

    if bytes_read == 0 {
        Ok(None)
    } else {
        input = input
            .replace("\u{1b}[A", "")
            .replace("\u{1b}[B", "")
            .replace("\u{1b}[C", "")
            .replace("\u{1b}[D", "")
            .replace("^[[A", "")
            .replace("^[[B", "")
            .replace("^[[C", "")
            .replace("^[[D", "");
        Ok(Some(input))
    }
}

fn continuation_prompt(state: parser::ContinuationState) -> &'static str {
    match state {
        parser::ContinuationState::DoubleQuote => "close double quote> ",
        parser::ContinuationState::SingleQuote => "close single quote> ",
        parser::ContinuationState::Backslash => "> ",
    }
}

fn read_command() -> io::Result<Option<String>> {
    let display = env::var("PWD").unwrap_or_else(|_| {
        env::current_dir()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| "?".to_string())
    });
    let main_prompt = format!("\x1b[1;36m{}\x1b[0m \x1b[1;37m$\x1b[0m ", display);

    let Some(mut input) = read_line_with_prompt(&main_prompt)? else {
        return Ok(None);
    };

    while let Some(state) = parser::continuation_state(input.trim_end_matches(['\n', '\r'])) {
        match state {
            parser::ContinuationState::Backslash => {
                while matches!(input.chars().last(), Some('\n' | '\r')) {
                    input.pop();
                }
                if input.ends_with('\\') {
                    input.pop();
                }
            }
            _ => {
                if !input.ends_with('\n') {
                    input.push('\n');
                }
            }
        }

        let Some(next_line) = read_line_with_prompt(continuation_prompt(state))? else {
            return Ok(None);
        };
        input.push_str(&next_line);
    }

    Ok(Some(input))
}

fn main() {
    println!("\x1b[1;32m0-shell\x1b[0m — type \x1b[1mhelp\x1b[0m for available commands\n");

    let mut shell = Shell::new();
    welcom::welcom();
    if let Ok(path) = env::current_dir() {
        env::set_var("PWD", path.display().to_string());
    }

    ctrlc::set_handler(|| {
        INTERRUPTED.store(true, Ordering::SeqCst);
    })
    .expect("failed to install Ctrl+C handler");

    loop {
        if INTERRUPTED.swap(false, Ordering::SeqCst) {
            println!("\n");
        }

        let Some(input) = read_command().unwrap() else {
            println!("\nExiting shell. Bye!");
            break;
        };

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
