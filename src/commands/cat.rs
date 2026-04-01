use crate::INTERRUPTED;
use std::fs;
use std::io::{self, BufRead, Write};
use std::sync::atomic::Ordering;

fn print_stdin(number_lines: bool) -> Result<(), String> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut line = String::new();
    let mut line_number = 1;

    loop {
        if INTERRUPTED.load(Ordering::SeqCst) {
            break;
        }

        line.clear();
        match handle.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                if number_lines {
                    print!("{:>6}\t{}", line_number, line);
                    line_number += 1;
                } else {
                    print!("{}", line);
                }
                io::stdout().flush().ok();
            }
            Err(err) if err.kind() == io::ErrorKind::Interrupted => break,
            Err(err) => return Err(format!("cat: {}", err)),
        }
    }

    Ok(())
}

pub fn run(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return print_stdin(false);
    }

    let mut number_lines = false;
    let mut files: Vec<&str> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-n" => number_lines = true,
            other => files.push(other),
        }
    }

    for filename in files {
        if filename == "-" {
            print_stdin(number_lines)?;
            continue;
        }

        let contents =
            fs::read_to_string(filename).map_err(|e| format!("cat: {}: {}", filename, e))?;

        if number_lines {
            for (i, line) in contents.lines().enumerate() {
                println!("{:>6}\t{}", i + 1, line);
            }
        } else {
            print!("{}", contents);
        }
    }

    io::stdout().flush().ok();
    Ok(())
}
